#![allow(dead_code)]

//! Camada MCAL — Driver TWAI (Two-Wire Automotive Interface) assíncrono.
//!
//! Este módulo encapsula as operações CAN do ESP32-S3 usando o modelo assíncrono
//! do esp-hal e esp-rtos.
//!
//! ## Princípio AUTOSAR (P1)
//! Nenhuma task de aplicação (APP) deve importar `esp_hal` diretamente. Todo acesso
//! ao barramento CAN é feito através deste módulo MCAL.

use embedded_can::Frame;
use esp_hal::twai::{EspTwaiFrame, StandardId, TwaiRx, TwaiTx};

use crate::types::{CanError, CanFrame};

// =============================================================================
// Transmissão e Recepção Assíncrona via MCAL
// =============================================================================

/// Envia um frame CAN pelo transmissor TWAI de forma assíncrona.
pub async fn twai_send(
    tx: &mut TwaiTx<'static, esp_hal::Async>,
    frame: &CanFrame,
) -> Result<(), CanError> {
    let esp_frame = can_frame_to_esp_frame(frame).ok_or(CanError::TxFailed)?;
    tx.transmit_async(&esp_frame)
        .await
        .map_err(|e| {
            log::error!("TWAI tx error: {:?}", e);
            CanError::TxFailed
        })
}

/// Recebe um frame CAN pelo receptor TWAI de forma assíncrona.
pub async fn twai_recv(
    rx: &mut TwaiRx<'static, esp_hal::Async>,
) -> Result<CanFrame, CanError> {
    let esp_frame = rx.receive_async().await.map_err(|e| {
        log::error!("TWAI rx error: {:?}", e);
        CanError::HalError
    })?;
    Ok(esp_frame_to_can_frame(&esp_frame))
}

// =============================================================================
// Recuperação de Falhas (Bus-Off) - AC-07
// =============================================================================

/// Recupera o periférico TWAI do estado de Bus-Off utilizando o Peripheral Access Crate (PAC).
///
/// **INVARIANTE DE SEGURANÇA (UNSAFE):**
/// Esta função acessa e modifica os registradores de hardware do TWAI diretamente.
/// Ela é `safe` SOMENTE se invocada pela camada BSW (ex: `bsw_diag`) com a garantia estrita
/// de que TODAS as *tasks* que detêm instâncias de `TwaiRx` e `TwaiTx` estão pausadas
/// e aguardando cooperativamente pelo `BUS_OFF_CLEAR` signal.
/// Modificar o hardware enquanto as tasks estão ativamente aguardando interrupções
/// pode causar corrupção de estado no Waker do executor.
pub unsafe fn twai_recover_unsafe() {
    let twai = unsafe { &*esp_hal::peripherals::TWAI0::PTR };
    
    // Entrar no modo de reset
    twai.mode().modify(|_, w| w.reset_mode().set_bit());
    
    // Pequeno delay via compiler fence para hardware estabilizar
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    
    // Sair do modo de reset
    twai.mode().modify(|_, w| w.reset_mode().clear_bit());
}

// =============================================================================
// Conversores
// =============================================================================

/// Converte um `EspTwaiFrame` do esp-hal para o nosso `CanFrame` interno.
pub fn esp_frame_to_can_frame(esp_frame: &EspTwaiFrame) -> CanFrame {
    let id = match esp_frame.id() {
        embedded_can::Id::Standard(sid) => sid.as_raw() as u32,
        embedded_can::Id::Extended(eid) => eid.as_raw(),
    };
    let data = esp_frame.data();
    let mut frame_data = [0u8; 8];
    let len = data.len().min(8);
    frame_data[..len].copy_from_slice(&data[..len]);
    CanFrame {
        id,
        data: frame_data,
        dlc: esp_frame.dlc() as u8,
    }
}

/// Constrói um `EspTwaiFrame` a partir de um `CanFrame` de solicitação OBD-II.
pub fn can_frame_to_esp_frame(frame: &CanFrame) -> Option<EspTwaiFrame> {
    let std_id = StandardId::new(frame.id as u16)?;
    let data_slice = &frame.data[..frame.dlc as usize];
    EspTwaiFrame::new(std_id, data_slice)
}
