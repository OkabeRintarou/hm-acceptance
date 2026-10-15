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

Implemented: all 151 official opcodes and stable unofficial LAX/SAX,
DCP/ISB (ISC), SLO/SRE/RLA/RRA and NOP forms. Also ANC/ALR/ARR/AXS and the
unofficial immediate SBC alias. `cpu::Cpu::step(&mut dyn CpuBus)` returns
instruction cycles or a typed `CpuError`; `reset` performs seven bus reads.
`Cpu::cycles` counts instruction cycles, while `SystemBus::cycles` includes
DMA/DMC stalls. RAM mirrors every 2 KiB; indexed and branch page-cross reads,
RMW old-value writes, stack dummy reads and the indirect-JMP page-wrap bug
are modeled. OAM DMA starts on the next CPU read and stalls 513/514 cycles.
Controller reads retain open-bus high bits and return 1 after eight buttons.

`Cartridge` exposes format, mapper/submapper, trainer, mirroring, battery,
PRG/CHR RAM and NVRAM sizes, console type and timing. NES 2.0 linear/MSB and
exponent/multiplier ROM sizes are decoded with checked arithmetic. The parser
then validates mapper-0 layouts: 16/32 KiB PRG, 8 KiB CHR ROM or up to 8 KiB
CHR RAM, at most 8 KiB PRG RAM/NVRAM, and submapper 0. iNES byte-8 zero implies
8 KiB PRG RAM; absent CHR ROM implies 8 KiB CHR RAM. NES 2.0 RAM zero means
absent, so CHR-ROM-less images must declare CHR RAM. Trainers are preserved
and copied into $7000–$71FF. Trailing file data is tolerated; reserved iNES
bytes are ignored, but mapper bits in byte 7 are honored. Four-screen uses
an extra 2 KiB alongside console CIRAM; palette RAM mirrors $3F10/$14/$18/$1C.
`Nrom::prg_ram[_mut]` permits a host to save/load RAM; disk persistence is
outside the core. Smaller RAM declarations mirror within the board window.

Additional API: `Console::try_step_frame() -> Result<(), CpuError>` handles
unsupported/JAM instructions without a panic. The specified `step_frame()`
wraps it and panics on those errors. `Console::new` rejects PAL/Dendy and
nonstandard console types with an assertion; multi-region images select NTSC.
`Console::reset` retains CPU registers and memory, resets PPU/APU and consumes
the hardware CPU reset sequence. Public `cpu` and `bus` fields permit headless
debugging; invalid controller ports are ignored.

Known limitations: PPU/APU here are still owned interface placeholders (blank
framebuffer and no audio/NMI/APU IRQ/DMC requests). Their frame stub advances
262×341 dots, without odd-frame skipping. CPU interrupts are sampled between
instructions with CLI/SEI/PLP mask delay modeled; exact edge races, BRK/IRQ
NMI hijacking, reset line timing and branch interrupt-poll quirks are not
cycle-exact. DMC fetches stall four cycles at the next CPU read; precise
OAM/DMC arbitration is deferred. Open bus has no analog decay. Unstable
unofficial instructions and JAM return errors. No PAL, Dendy, VS, additional
mappers, save states, or file/device I/O are implemented.

Verification: `cargo build --workspace`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`. Tests generate their
own iNES/NES 2.0 fixtures and a small executable ROM; no external binary is
vendored or downloaded. Coverage includes all official base timings,
page-cross and dummy bus access traces, reset/BRK/IRQ/NMI/stack behavior,
unofficial families, exhaustive binary ADC/SBC flags, cartridge failures,
NROM ROM/RAM and mirroring, controllers, and DMA. nestest has not been run.
