# nes-core

Rust stable / edition 2021 library with no external dependencies. Initial scope
is NTSC and mapper 0 (NROM); `nes-desktop` is a separate placeholder binary.

Ownership: core owns `cartridge`, `mapper`, `cpu`, `bus`, `console`, `input`.
The PPU/APU developer owns `src/ppu/` and `src/apu/`; these currently contain
compiling interface stubs, replaced by their implementation in a separate task.

Frontend API: `Cartridge::from_bytes(&[u8]) -> Result<Cartridge, CartridgeError>`,
`Console::new(cartridge, sample_rate)`, `step_frame()`, `reset()`,
`framebuffer() -> &[u8]` (256×240 RGBA8), `drain_audio(&mut Vec<f32>)`
(mono [-1, 1], appended to the destination), `set_buttons(port, state)`.
Button bits 0..7 are A/B/Select/Start/Up/Down/Left/Right; ports 0 and 1.

`Mapper` supplies mutable CPU/PPU reads and writes, mirroring and optional
IRQ/CPU clock hooks. `PpuMemory` is supplied by the bus and routes pattern
tables through the mapper, nametables through mirrored bus-owned VRAM,
and palette memory through the bus. The PPU's CPU register methods receive
addresses in $2000–$3FFF (the PPU masks register mirrors). `Ppu::tick` advances
one dot; `frame_ready` is a public bool cleared by the console; `take_nmi`
consumes a latched edge. OAM DMA calls `oam_dma_write` once per byte.

`Apu::tick` advances one CPU cycle. CPU register dispatch excludes $4014/$4016;
$4017 reads controllers and writes the APU frame counter. `dmc_request` consumes
one pending fetch address; the bus returns the sample with `dmc_provide` and
stalls CPU execution. `take_samples` appends and consumes queued f32 samples.

Timing: NTSC CPU 1,789,773 Hz, three PPU dots per CPU cycle, nominally
29,780.5 CPU cycles/frame. CPU bus accesses, including dummy accesses and DMA,
advance hardware. Decimal arithmetic is disabled on the 2A03.
