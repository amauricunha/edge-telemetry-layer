//! Abstração de Hardware para o SD Card via barramento SPI2 (MCAL).
//!
//! Configura os pinos SPI2 físicos do ESP32-S3 (esp-hal 1.1.1):
//!   - GPIO10 = CS (OutputPin passado diretamente ao SdCard — gerenciado pelo sdmmc)
//!   - GPIO11 = MOSI (Master Out Slave In)
//!   - GPIO12 = CLK/SCK (Serial Clock)
//!   - GPIO13 = MISO (Master In Slave Out)
//!
//! `embedded-sdmmc 0.7` usa `SpiDevice<u8>` (embedded-hal 1.0) + CS separado.
//! O SPI bus sem CS é criado com `NoCs` do `embedded-hal-bus`.

extern crate alloc;

use alloc::boxed::Box;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use embedded_sdmmc::{RawDirectory, RawVolume, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::spi::Mode as SpiMode;
use esp_hal::time::Rate;

use portable_atomic::Ordering;

/// Timestamp fixo para o `TimeSource` do `embedded-sdmmc` (sem RTC real na Fase 1).
pub struct DummyTimeSource;

impl TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 56, // 2026
            zero_indexed_month: 6,
            zero_indexed_day: 28,
            hours: 20,
            minutes: 0,
            seconds: 0,
        }
    }
}

/// Dummy CS para o `ExclusiveDevice` — o CS real é gerenciado pelo `SdCard`.
/// O `SdCard` do `embedded-sdmmc` controla o CS pin diretamente via `OutputPin`.
pub struct NoCs;

impl embedded_hal::digital::OutputPin for NoCs {
    fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
}

impl embedded_hal::digital::ErrorType for NoCs {
    type Error = core::convert::Infallible;
}

type SpiBus = Spi<'static, esp_hal::Blocking>;
type CsPin = Output<'static>;
type SdCardDevice = SdCard<ExclusiveDevice<SpiBus, NoCs, NoDelay>, CsPin, Delay>;
type SdVolumeMgr = VolumeManager<SdCardDevice, DummyTimeSource>;

pub struct SdStorage {
    /// O VolumeManager é mantido em Box (heap) para evitar stack overflow.
    /// O embedded-sdmmc contém caches internos de setor (~4 KB) que são
    /// grandes demais para a pilha Xtensa quando combinados com esp-radio.
    pub volume_mgr: Box<SdVolumeMgr>,
    pub _raw_volume: RawVolume,
    pub root_dir: RawDirectory,
}

pub static SD_STORAGE: Mutex<CriticalSectionRawMutex, Option<SdStorage>> = Mutex::new(None);
pub static CURRENT_FILENAME: Mutex<CriticalSectionRawMutex, heapless::String<16>> = Mutex::new(heapless::String::new());

/// Inicializa o barramento SPI2 físico e detecta o cartão SD.
pub fn init_and_probe(
    spi2: esp_hal::peripherals::SPI2<'static>,
    gpio10: esp_hal::peripherals::GPIO10<'static>,
    gpio11: esp_hal::peripherals::GPIO11<'static>,
    gpio12: esp_hal::peripherals::GPIO12<'static>,
    gpio13: esp_hal::peripherals::GPIO13<'static>,
) {
    let spi_bus = match Spi::new(
        spi2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(15))
            .with_mode(SpiMode::_0),
    ) {
        Ok(spi) => spi
            .with_mosi(gpio11)
            .with_sck(gpio12)
            .with_miso(gpio13),
        Err(e) => {
            log::error!("[MCAL SPI] Falha ao inicializar barramento SPI2: {:?}", e);
            crate::bsw::bsw_mem::set_sd_present(false);
            return;
        }
    };

    let cs_for_sdmmc = Output::new(gpio10, Level::High, OutputConfig::default());
    let delayer = Delay::new();

    let spi_device = ExclusiveDevice::new_no_delay(spi_bus, NoCs)
        .expect("ExclusiveDevice sempre retorna Ok com NoCs");

    let mut opts = embedded_sdmmc::sdcard::AcquireOpts::default();
    opts.acquire_retries = 5;
    let sd_card = SdCard::new_with_options(spi_device, cs_for_sdmmc, delayer, opts);
    // Alocar VolumeManager diretamente no heap controlado (esp-alloc, 58 KB).
    // Isso evita o stack overflow causado pelos caches internos de setor do embedded-sdmmc.
    let mut volume_mgr = Box::new(VolumeManager::new(sd_card, DummyTimeSource));

    match volume_mgr.open_raw_volume(VolumeIdx(0)) {
        Ok(raw_volume) => {
            match volume_mgr.open_root_dir(raw_volume) {
                Ok(root_dir) => {
                    // Armazenar no static SD_STORAGE — volume_mgr já está no heap (Box)
                    if let Ok(mut guard) = SD_STORAGE.try_lock() {
                        *guard = Some(SdStorage {
                            volume_mgr,
                            _raw_volume: raw_volume,
                            root_dir,
                        });
                    }

                    crate::bsw::bsw_mem::set_sd_present(true);

                    // Inicializar a sessão ativa no SD Storage estático (sem ocupar stack local)
                    if let Some(session_idx) = rotate_session_file() {
                        log::info!("[MCAL SPI] Cartão SD detectado e montado! Sessão ativa: 'S_{:04}'.", session_idx);
                    } else {
                        log::warn!("[MCAL SPI] Cartão SD montado, mas falha ao inicializar arquivo de sessão.");
                    }
                }
                Err(e) => {
                    log::error!("[MCAL SPI] Erro ao abrir diretório raiz do SD Card: {:?}", e);
                    crate::bsw::bsw_mem::set_sd_present(false);
                }
            }
        }
        Err(e) => {
            log::error!(
                "[MCAL SPI] SD Card ERRO: {:?}. \
                 Verifique: (1) Cartão inserido? (2) Formatado como FAT32?",
                e
            );
            crate::bsw::bsw_mem::set_sd_present(false);
        }
    }
}

/// Escreve um bloco de bytes (chunk) no arquivo de sessão ativo S_XXXX.CSV no cartão SD FAT32.
/// Por conformidade AUTOSAR (D-07), esta função é síncrona pura (MCAL).
pub fn write_sector_blocking(bytes: &[u8]) -> Result<(), ()> {
    let mut guard = SD_STORAGE.try_lock().map_err(|_| ())?;
    let storage = guard.as_mut().ok_or(())?;

    let fn_guard = CURRENT_FILENAME.try_lock().map_err(|_| ())?;
    let filename = if fn_guard.is_empty() { "S_0001.CSV" } else { fn_guard.as_str() };

    let file = storage.volume_mgr.open_file_in_dir(
        storage.root_dir,
        filename,
        embedded_sdmmc::Mode::ReadWriteCreateOrAppend,
    ).map_err(|_| ())?;

    let _ = storage.volume_mgr.write(file, bytes);
    let _ = storage.volume_mgr.close_file(file);

    Ok(())
}

/// Apaga todos os arquivos de sessão S_XXXX.CSV e reinicia de S_0001.CSV
pub fn wipe_all_sessions() {
    let mut guard = match SD_STORAGE.try_lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    
    if let Some(storage) = guard.as_mut() {
        log::warn!("[MCAL SPI] Iniciando WIPE do Cartão SD (Isso pode levar alguns segundos)...");
        let mut session_filename: heapless::String<16> = heapless::String::new();
        let mut deleted_count = 0;
        
        for id in 1..=9999 {
            session_filename.clear();
            let _ = core::fmt::write(&mut session_filename, format_args!("S_{:04}.CSV", id));
            
            // Verifica se o arquivo existe e deleta
            if let Ok(file) = storage.volume_mgr.open_file_in_dir(storage.root_dir, session_filename.as_str(), embedded_sdmmc::Mode::ReadOnly) {
                let _ = storage.volume_mgr.close_file(file);
                if storage.volume_mgr.delete_file_in_dir(storage.root_dir, session_filename.as_str()).is_ok() {
                    deleted_count += 1;
                }
            } else {
                // Como os arquivos são sequenciais, podemos parar no primeiro que não existe para economizar tempo
                break;
            }
        }
        
        log::warn!("[MCAL SPI] WIPE Concluído: {} arquivos removidos.", deleted_count);
        
        // Esvaziar buffers de memória pendentes para não recriar arquivos deletados
        crate::bsw::bsw_mem::clear_buffer();
        
        // Reiniciar contadores para S_0001
        crate::bsw::bsw_mem::SESSION_ID.store(1, Ordering::Relaxed);
        crate::bsw::bsw_mem::SD_WRITE_OFFSET.store(0, Ordering::Relaxed);
        
        if let Ok(mut fn_guard) = CURRENT_FILENAME.try_lock() {
            fn_guard.clear();
        }
        
        log::info!("[MCAL SPI] Reset completo: próxima sessão será iniciada em S_0001.CSV.");
    }
}

/// Rotaciona para o próximo arquivo de sessão disponível (S_XXXX.CSV)
pub fn rotate_session_file() -> Option<u16> {
    let mut guard = match SD_STORAGE.try_lock() {
        Ok(g) => g,
        Err(_) => return None,
    };
    
    if let Some(storage) = guard.as_mut() {
        let mut session_idx: u16 = 1;
        let mut session_filename: heapless::String<16> = heapless::String::new();

        for id in 1..=9999 {
            session_filename.clear();
            let _ = core::fmt::write(&mut session_filename, format_args!("S_{:04}.CSV", id));
            let is_free = match storage.volume_mgr.open_file_in_dir(storage.root_dir, session_filename.as_str(), embedded_sdmmc::Mode::ReadOnly) {
                Ok(f) => {
                    let _ = storage.volume_mgr.close_file(f);
                    false
                }
                Err(_) => true,
            };
            if is_free {
                session_idx = id;
                break;
            }
        }

        if let Ok(file) = storage.volume_mgr.open_file_in_dir(
            storage.root_dir,
            session_filename.as_str(),
            embedded_sdmmc::Mode::ReadWriteCreateOrAppend,
        ) {
            let _ = storage.volume_mgr.close_file(file);
            
            crate::bsw::bsw_mem::SESSION_ID.store(session_idx, Ordering::Relaxed);
            crate::bsw::bsw_mem::SD_WRITE_OFFSET.store(0, Ordering::Relaxed);

            if let Ok(mut fn_guard) = CURRENT_FILENAME.try_lock() {
                *fn_guard = session_filename.clone();
            }

            log::info!(
                "[MCAL SPI] Rotação de SD Card! Nova sessão 'S_{:04}' aberta no arquivo '{}'.",
                session_idx, session_filename
            );
            return Some(session_idx);
        }
    }
    None
}

/// Retorna a lista de nomes dos arquivos de sessão existentes no SD Card (ex: S_0001.CSV, S_0002.CSV).
pub fn list_session_files() -> heapless::Vec<heapless::String<16>, 32> {
    let mut list = heapless::Vec::<heapless::String<16>, 32>::new();
    let mut guard = match SD_STORAGE.try_lock() {
        Ok(g) => g,
        Err(_) => return list,
    };

    if let Some(storage) = guard.as_mut() {
        let mut session_filename: heapless::String<16> = heapless::String::new();
        for id in 1..=9999 {
            session_filename.clear();
            let _ = core::fmt::write(&mut session_filename, format_args!("S_{:04}.CSV", id));
            if let Ok(file) = storage.volume_mgr.open_file_in_dir(storage.root_dir, session_filename.as_str(), embedded_sdmmc::Mode::ReadOnly) {
                let _ = storage.volume_mgr.close_file(file);
                if list.push(session_filename.clone()).is_err() {
                    break;
                }
            } else {
                break;
            }
        }
    }
    list
}

use embedded_sdmmc::RawFile;

/// MCAL Stream Reader: abstrai a leitura sequencial em blocos do SD Card (AUTOSAR D-07).
pub struct SessionStreamReader {
    file: RawFile,
}

impl SessionStreamReader {
    /// Abre o arquivo de sessão para streaming sequencial no MCAL
    pub fn open(session_filename: &str) -> Option<Self> {
        let mut guard = SD_STORAGE.try_lock().ok()?;
        let storage = guard.as_mut()?;
        let file_size = match storage.volume_mgr.find_directory_entry(storage.root_dir, session_filename) {
            Ok(entry) => entry.size,
            Err(_) => 0,
        };
        match storage.volume_mgr.open_file_in_dir(
            storage.root_dir,
            session_filename,
            embedded_sdmmc::Mode::ReadOnly,
        ) {
            Ok(file) => {
                log::info!("[MCAL SPI] Arquivo '{}' aberto para streaming (Tamanho: {} bytes).", session_filename, file_size);
                Some(Self { file })
            }
            Err(e) => {
                log::error!("[MCAL SPI] Falha ao abrir '{}' para streaming: {:?}", session_filename, e);
                None
            }
        }
    }

    /// Lê o próximo bloco de bytes do arquivo mantendo o cursor FAT32
    pub fn read_chunk(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let mut guard = SD_STORAGE.try_lock().map_err(|_| ())?;
        let storage = guard.as_mut().ok_or(())?;
        storage.volume_mgr.read(self.file, buf).map_err(|_| ())
    }

    /// Fecha o arquivo de sessão no SD Card
    pub fn close(self) {
        if let Ok(mut guard) = SD_STORAGE.try_lock() {
            if let Some(storage) = guard.as_mut() {
                let _ = storage.volume_mgr.close_file(self.file);
                log::info!("[MCAL SPI] Arquivo de streaming fechado com sucesso.");
            }
        }
    }
}

