use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use heapless::Vec;
use portable_atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering};

/// Buffer circular estático de 4 KB (alinhado a 8 setores FAT32 de 512 B).
static SD_BUFFER: Mutex<CriticalSectionRawMutex, Vec<u8, 4096>> = Mutex::new(Vec::new());

/// Flag de sinalização de flush necessário (atingiu 85% de capacidade = 3584 B).
static FLUSH_NEEDED: AtomicBool = AtomicBool::new(false);

/// Signal para acordar imediatamente a task_sd_writer sem esperar o timeout.
pub static FLUSH_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// Status operacional do cartão SD (False quando o cartão não está presente no slot).
pub static SD_OK: AtomicBool = AtomicBool::new(false);

/// Identificador sequencial da sessão/viagem ativa (ex: 1 -> S0001, 2 -> S0002)
pub static SESSION_ID: AtomicU16 = AtomicU16::new(1);

/// Ponteiro de offset de escrita no SD Card
pub static SD_WRITE_OFFSET: AtomicU32 = AtomicU32::new(0);

/// Registra se o hardware do cartão SD foi montado com sucesso.
pub fn set_sd_present(present: bool) {
    SD_OK.store(present, Ordering::Relaxed);
    if present {
        log::info!("BSW Mem: Cartão SD detectado e montado em FAT32.");
    } else {
        log::warn!("BSW Mem: Nenhum cartão SD detectado no slot (Modo Fallback ativado).");
    }
}

/// Adiciona uma linha formatada ao buffer de memória do SD Card.
pub fn push_line(line: &str) -> Result<(), ()> {
    // Se o SD não estiver operacional, falhar imediatamente para ativar o fallback no logger
    if !SD_OK.load(Ordering::Relaxed) {
        return Err(());
    }

    let bytes = line.as_bytes();

    // Bloqueio rápido em seção crítica
    let mut guard = SD_BUFFER.try_lock().map_err(|_| ())?;

    // Verificar se há espaço suficiente
    if guard.len() + bytes.len() > 4096 {
        return Err(());
    }

    let _ = guard.extend_from_slice(bytes);

    // Se atingir 85% (3584 bytes), acionar a task de escrita imediatamente via Signal
    if guard.len() >= 3584 {
        FLUSH_NEEDED.store(true, Ordering::Relaxed);
        FLUSH_SIGNAL.signal(());
    }

    Ok(())
}

/// Limpa completamente o buffer em RAM (usado no RESET/WIPE para evitar recriar arquivos deletados).
pub fn clear_buffer() {
    if let Ok(mut guard) = SD_BUFFER.try_lock() {
        guard.clear();
        FLUSH_NEEDED.store(false, Ordering::Relaxed);
    }
}

/// Executa o flush síncrono e imediato de todo o conteúdo do buffer para o arquivo atual.
pub fn flush_sync() {
    if !SD_OK.load(Ordering::Relaxed) {
        return;
    }
    let mut bytes_to_flush = Vec::<u8, 4096>::new();
    if let Ok(mut guard) = SD_BUFFER.try_lock() {
        if !guard.is_empty() {
            let _ = bytes_to_flush.extend_from_slice(&guard);
            guard.clear();
            FLUSH_NEEDED.store(false, Ordering::Relaxed);
        }
    }
    if !bytes_to_flush.is_empty() {
        let len = bytes_to_flush.len();
        let mut write_success = true;
        for chunk in bytes_to_flush.chunks(256) {
            if crate::mcal::spi_sd::write_sector_blocking(chunk).is_err() {
                write_success = false;
                break;
            }
        }
        if write_success {
            SD_WRITE_OFFSET.fetch_add(len as u32, Ordering::Relaxed);
            log::info!("BSW Mem: Flush síncrono de {} bytes concluído no arquivo atual.", len);
        }
    }
}

/// Transição atômica de sessão: faz flush do arquivo antigo, abre o novo,
/// grava o cabeçalho e boot line como primeiras linhas e inicia a temporização.
pub fn flush_and_rotate_session(
    label: crate::types::SessionLabel,
    duration_min: u32,
    epoch_opt: Option<u64>,
) -> Option<u16> {
    // 1. Esvaziar buffer pendente da sessão anterior no arquivo antigo
    flush_sync();

    // 2. Rotacionar para o próximo arquivo S_XXXX.CSV
    let new_session_idx = crate::mcal::spi_sd::rotate_session_file()?;

    // 3. Atualizar o label ativo
    crate::types::ACTIVE_SESSION_LABEL.store(label.as_u8(), Ordering::Relaxed);

    // 4. Inserir cabeçalho e linha BOOT como PRIMEIRAS LINHAS do novo arquivo
    let header = crate::app::csv_writer::csv_header();
    let _ = push_line(header);
    let boot_line = crate::app::csv_writer::serialize_boot(0, &label);
    let _ = push_line(&boot_line);

    // 5. Iniciar o temporizador de sessão (se duration_min > 0) e sincronizar epoch
    crate::types::start_session_timer(duration_min, epoch_opt);

    log::info!(
        "BSW Mem: Nova sessão S_{:04} ({}) iniciada atomicamente (Duração: {} min).",
        new_session_idx, label.as_str(), duration_min
    );

    Some(new_session_idx)
}


/// Task Embassy de flush periódico para o SD Card (a cada 2 s ou imediatamente ao atingir 3.5 KB).
#[embassy_executor::task]
pub async fn task_sd_writer() {
    log::info!("BSW Mem: task_sd_writer iniciada (Flush a cada 2s ou imed. em 3.5 KB)");

    loop {
        embassy_futures::select::select(
            Timer::after(Duration::from_secs(2)),
            FLUSH_SIGNAL.wait(),
        )
        .await;

        if !SD_OK.load(Ordering::Relaxed) {
            continue;
        }

        let mut bytes_to_flush = Vec::<u8, 4096>::new();
        {
            let mut guard = SD_BUFFER.lock().await;
            if !guard.is_empty() {
                let _ = bytes_to_flush.extend_from_slice(&guard);
                guard.clear();
                FLUSH_NEEDED.store(false, Ordering::Relaxed);
            }
        }

        if !bytes_to_flush.is_empty() {
            let len = bytes_to_flush.len();
            let mut write_success = true;

            // D-06 e D-07: Chunking de 256 bytes com multitarefa cooperativa na BSW
            for chunk in bytes_to_flush.chunks(256) {
                if crate::mcal::spi_sd::write_sector_blocking(chunk).is_err() {
                    write_success = false;
                    break;
                }
                embassy_futures::yield_now().await;
            }

            if write_success {
                SD_WRITE_OFFSET.fetch_add(len as u32, Ordering::Relaxed);
                let fn_guard = crate::mcal::spi_sd::CURRENT_FILENAME.try_lock();
                let fn_str = match &fn_guard {
                    Ok(g) if !g.is_empty() => g.as_str(),
                    _ => "S_0001.CSV",
                };
                log::info!("BSW Mem: Flush de {} bytes GRAVADO COM SUCESSO no arquivo {} do SD Card!", len, fn_str);
            } else {
                log::error!("BSW Mem: Falha ao escrever {} bytes no SD Card!", len);
            }
        }
    }
}
