# Registro de Decisões de Arquitetura (ADR / Decisions)

## D-01: Serialização CSV Sem Alocação Dinâmica no Heap
Usar `heapless::String<320>` e formatação estática em tempo de compilação sem depender de `format!` da biblioteca padrão (`std`).

## D-02: Buffer Circular de 4 KB Alinhado ao Setor FAT32
Implementar `SD_BUFFER` estático de 4 KB (8 setores de 512 bytes) com flush assíncrono acionado a 85% de capacidade (3584 bytes) ou timeout de 10 s.

## D-03: Compartilhamento de Barramento SPI via `embassy_embedded_hal`
Utilizar `embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice` para isolar o CS (GPIO 10) e permitir o acesso seguro do `embedded-sdmmc` sem incluir a dependência redundante `embedded-hal-bus`.

## D-04: Alocação de Backlog de Reconexão na PSRAM (External RAM)
Alocar o buffer de backlog de reconexão (500 frames × 320 B ≈ 160 KB) na PSRAM de 8 MB do ESP32-S3-WROOM-1 (N16R8) usando a seção `#[link_section = ".ext_ram.bss"]`. Em caso de indisponibilidade da PSRAM em runtime, aplicar fallback para 50 frames (16 KB) na SRAM interna.

## D-05: Cliente MQTT em Ambiente `no_std` (`rust-mqtt` sobre `embassy-net`)
- **Contexto:** A crate `rumqttc` (v0.24) exige a biblioteca padrão (`std::net::TcpStream`) e o runtime `tokio`, tornando-a incompatível com o firmware `no_std` do ESP32-S3 sob o `esp-rtos`.
- **Decisão:** Substituir `rumqttc` pela crate **`rust-mqtt`** (v0.3.0).
- **Justificativa:** `rust-mqtt` é projetada nativamente para `no_std`, operando diretamente sobre sockets TCP assíncronos de `embassy_net::tcp::TcpSocket` sem qualquer alocação dinâmica no heap (`zero-copy`).
- **Consequência:** Garante total conformidade com o ecossistema Embassy e zero dependência de `std`.

## D-06: Monitoramento Automático de Performance sem Overhead de Processamento (AC-05)
- **Contexto:** Necessidade de extrair as métricas de uso de memória SRAM e uso de buffer SD, fundamentais para a validação do artigo final, sem depender de scripts locais ou análise manual do `esp-idf heap monitor`.
- **Decisão:** Realizar o probing direto ao allocator global `esp_alloc::HEAP.free()` e `.used()` a cada 5s de forma silenciosa, consolidando mínimos, médias e máximos localmente na memória RAM (*stack variables*) e injetando as estatísticas em uma linha especial `DIAG,HEARTBEAT` no próprio log CSV.
- **Justificativa:** Extrai de maneira auto-contida os números reais de performance sem introduzir alocação extra, sobrecarga atômica de escalonador ou aumento na taxa de overhead. Adicionalmente, as linhas do tipo DIAG são logicamente desconsideradas na contagem do dataset principal, preservando 100% da validade dos arquivos originados para análise.

## D-07: Chunking de Gravação SD (256 bytes)
- **Contexto**: O flush massivo (3.5 KB) do buffer de telemetria no SD bloqueava o executor `esp-hal` por longos períodos (10~15ms), impedindo o poll do driver TWAI.
- **Decisão**: Dividir o flush em pedaços (*chunks*). Em vez de 512 bytes, optou-se por **256 bytes**. A justificativa matemática: 512 bytes × 8 bits / 4 MHz + overhead ≈ 1.52 ms de bloqueio por chunk, enquanto a FIFO do TWAI a 500 Kbps transborda em ~1.08 ms (5 frames). Com 256 bytes, o bloqueio cai para **~0.76 ms**, permanecendo dentro da janela segura de *preemption* e garantindo a integridade dos pacotes.

## D-08: yield_now() restrito à BSW (Conformidade AUTOSAR)
- **Contexto**: Para aplicar a multitarefa cooperativa, o controle deve ser devolvido ao executor após cada chunk gravado.
- **Decisão**: Em conformidade com AUTOSAR Classic (Separação MCAL vs BSW), o `yield_now().await` pertence exclusivamente à camada BSW (`bsw_mem.rs`). A camada MCAL (`spi_sd.rs`) não pode importar `embassy_futures` nem conhecer o contexto de runtime/tasks. A função MCAL `write_sector_blocking` atua como uma interface síncrona pura. A decisão de ceder o controle pertence à BSW.

## D-09: Estratégia de recuperação de Bus-Off via pausa cooperativa (Signals) e reinicialização unsafe direta via PAC
- **Contexto**: Quando ocorre um Bus-Off, o controlador TWAI exige 128 ciclos recessivos e intervenção de software para limpar a flag de reset. Como o framework *Embassy* trabalha com *tasks* alocadas estaticamente no *boot*, não é possível utilizar funções de de-spawn e spawn para recriar o TWAI em runtime de forma segura quando a estrutura já foi dividida entre `TwaiTx` e `TwaiRx`.
- **Decisão**: Implementar pausas cooperativas nas *tasks* consumidoras de `TwaiTx` e `TwaiRx` usando o macro `select!()`, gerenciadas via Signals Globais (`BUS_OFF_SIGNAL` e `BUS_OFF_CLEAR`). A função de reestabelecimento efetivo na camada MCAL (`twai::twai_recover_unsafe()`) utiliza acessos diretamente aos registradores do SoC (via PAC) com `.modify(|_, w| w.reset_mode().set_bit())`, evadindo as limitações do crate `esp-hal`. Como é um bloco inteiramente síncrono que interage no metal da arquitetura Xtensa (`no_std`), usamos `core::sync::atomic::compiler_fence(Ordering::SeqCst)` para garantir o ordenamento serial estrito no pipeline e evitar que otimizações de compilador corrompam os ciclos de reset sem a necessidade de *loops* travantes (busy-wait) ou `await` na MCAL (que seria proibido pelo AUTOSAR).
- **Justificativa**: Preserva o ciclo de vida das *tasks*, garante *safety* com `embassy_sync`, atende a restrição de não poder encerrar *tasks*, e se mantém estritamente aderente ao AUTOSAR (MCAL pura lidando com registradores, sem saber sobre o assincronismo do framework, enquanto o BSW governa a lógica de pausa e janela de tempo da norma ISO-11898).
