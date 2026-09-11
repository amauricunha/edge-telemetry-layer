#![allow(dead_code)]

//! Task de polling ativo OBD-II.
//!
//! Implementa o ciclo de solicitação OBD-II conforme RF-03 da spec:
//! - Envia uma solicitação a cada 100 ms via `0x7DF`
//! - Circula pelos PIDs suportados em round-robin
//!
//! ## Arquitetura de Recepção
//! As respostas OBD-II (`0x7E8`) são recebidas pela `task_can_rx` no main.rs,
//! que é o único consumidor do periférico TWAI RX. O OBD poller é responsável
//! apenas pelo lado TX (envio das solicitações e comandos eventuais).
//!
//! Para cálculo de latência, este módulo registra o timestamp de envio nos
//! atomics globais `LAST_OBD_SEND_TICKS` e `LAST_OBD_PID`, que são lidos
//! pelo `task_can_rx` ao receber a resposta.

use embassy_time::{Duration, Instant};

use crate::config;
use crate::mcal::twai;
use crate::types::CanFrame;

/// Task Embassy de polling OBD-II ativo — lado TX.
///
/// Envia solicitações OBD-II em round-robin. As respostas são processadas
/// por `task_can_rx` no main.rs.
///
/// Registra timestamp e PID de cada envio para cálculo de latência.
#[embassy_executor::task]
pub async fn task_obd_poller(
    mut tx: esp_hal::twai::TwaiTx<'static, esp_hal::Async>,
) {
    let mut pid_index: usize = 0;

    log::info!(
        "OBD Poller: iniciado — {} PIDs, intervalo {}ms",
        config::OBD_PIDS.len(),
        config::OBD_POLL_INTERVAL_MS,
    );

    loop {
        match embassy_futures::select::select(
            async {
                let pid = config::OBD_PIDS[pid_index];

                // Montar frame OBD-II de solicitação: [0x02, 0x01, PID, 0, 0, 0, 0, 0]
                let request_data: [u8; 8] = [0x02, config::OBD_MODE_REQUEST, pid, 0, 0, 0, 0, 0];
                let request_frame = CanFrame::new(config::OBD_REQUEST_ID, &request_data);

                // Resetar flag de recebimento e registrar dados de envio
                crate::LAST_OBD_RESPONSE_RECEIVED.store(false, portable_atomic::Ordering::Relaxed);
                crate::LAST_OBD_SEND_TICKS.store(
                    Instant::now().as_ticks(),
                    portable_atomic::Ordering::Relaxed,
                );
                crate::LAST_OBD_PID.store(pid, portable_atomic::Ordering::Relaxed);

                // Enviar solicitação via MCAL com timeout de 10ms para evitar travamento em caso de erro físico no barramento
                match embassy_time::with_timeout(
                    Duration::from_millis(10),
                    twai::twai_send(&mut tx, &request_frame)
                ).await {
                    Ok(Ok(())) => {
                        log::debug!("OBD TX: PID={:#04x}", pid);
                    }
                    Ok(Err(e)) => {
                        log::error!("OBD Poller: Falha no twai_send: {:?}", e);
                    }
                    Err(_) => {
                        log::warn!("OBD Poller: Timeout enviando requisição para PID {:#04x}", pid);
                    }
                }

                // Aguardar o tempo limite do timeout (50 ms) ouvindo fila de comandos (multiplexação RTE)
                match embassy_time::with_timeout(
                    Duration::from_millis(config::OBD_TIMEOUT_MS),
                    crate::CAN_CMD_CHANNEL.receive()
                ).await {
                    Ok(cmd_frame) => {
                        process_can_cmd(&mut tx, cmd_frame).await;
                    }
                    Err(_) => {}
                }

                // Verificar se resposta foi recebida
                if !crate::LAST_OBD_RESPONSE_RECEIVED.load(portable_atomic::Ordering::Relaxed) {
                    log::warn!("OBD_TIMEOUT pid={:#04x}", pid);
                }

                // Aguardar o tempo restante para completar o intervalo total de 100 ms ouvindo fila de comandos
                let remaining_ms = config::OBD_POLL_INTERVAL_MS.saturating_sub(config::OBD_TIMEOUT_MS);
                if remaining_ms > 0 {
                    match embassy_time::with_timeout(
                        Duration::from_millis(remaining_ms),
                        crate::CAN_CMD_CHANNEL.receive()
                    ).await {
                        Ok(cmd_frame) => {
                            process_can_cmd(&mut tx, cmd_frame).await;
                        }
                        Err(_) => {}
                    }
                }

                // Avançar para o próximo PID
                pid_index = (pid_index + 1) % config::OBD_PIDS.len();
            },
            crate::BUS_OFF_SIGNAL.wait()
        ).await {
            embassy_futures::select::Either::First(_) => {}
            embassy_futures::select::Either::Second(_) => {
                log::warn!("OBD Poller pausado cooperativamente — aguardando Bus-Off recovery");
                crate::BUS_OFF_CLEAR.wait().await;
                log::info!("OBD Poller retomado após Bus-Off recovery");
            }
        }
    }
}

/// Helper para enviar o comando CAN lido do RTE.
async fn process_can_cmd(tx: &mut esp_hal::twai::TwaiTx<'static, esp_hal::Async>, cmd_frame: CanFrame) {
    log::info!("OBD Poller: Comando CAN recebido da fila RTE, enviando (ID: {:#x})", cmd_frame.id);
    match embassy_time::with_timeout(
        Duration::from_millis(10),
        twai::twai_send(tx, &cmd_frame)
    ).await {
        Ok(Ok(())) => {
            log::info!("OBD Poller: Comando CAN enviado com sucesso");
        }
        Ok(Err(e)) => {
            log::error!("OBD Poller: Falha ao enviar comando CAN: {:?}", e);
        }
        Err(_) => {
            log::warn!("OBD Poller: Timeout enviando comando CAN");
        }
    }
}
