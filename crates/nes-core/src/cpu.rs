//! Ricoh 2A03 execution. All memory operations, including dummy operations,
//! go through CpuBus exactly once per CPU cycle. Decimal mode is ignored.
use std::{error::Error, fmt};

#[path = "cpu/decode.rs"]
mod decode;
use decode::{Mode, Op, DECODE};

pub trait CpuBus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, v: u8);
    fn take_nmi(&mut self) -> bool {
        false
    }
    fn irq_pending(&self) -> bool {
        false
    }
}

pub const C: u8 = 0x01;
pub const Z: u8 = 0x02;
pub const I: u8 = 0x04;
pub const D: u8 = 0x08;
pub const B: u8 = 0x10;
pub const U: u8 = 0x20;
pub const V: u8 = 0x40;
pub const N: u8 = 0x80;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuError {
    pub opcode: u8,
    pub pc: u16,
}
impl fmt::Display for CpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unsupported / jam opcode ${:02X} at ${:04X}",
            self.opcode, self.pc
        )
    }
}
impl Error for CpuError {}

pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    /// B is not a physical flag; U is always set in this representation.
    pub status: u8,
    /// CPU instruction cycles; SystemBus also counts DMA/DMC stalls.
    pub cycles: u64,
    irq_mask: bool,
}
impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
impl Cpu {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            pc: 0,
            status: U | I,
            cycles: 0,
            irq_mask: true,
        }
    }
    fn read(&mut self, bus: &mut dyn CpuBus, addr: u16) -> u8 {
        self.cycles += 1;
        bus.read(addr)
    }
    fn write(&mut self, bus: &mut dyn CpuBus, addr: u16, v: u8) {
        self.cycles += 1;
        bus.write(addr, v);
    }
    fn fetch(&mut self, bus: &mut dyn CpuBus) -> u8 {
        let v = self.read(bus, self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }
    fn word(&mut self, bus: &mut dyn CpuBus) -> u16 {
        let lo = self.fetch(bus);
        u16::from(lo) | (u16::from(self.fetch(bus)) << 8)
    }
    fn push(&mut self, bus: &mut dyn CpuBus, v: u8) {
        self.write(bus, 0x100 | u16::from(self.sp), v);
        self.sp = self.sp.wrapping_sub(1);
    }
    fn pull(&mut self, bus: &mut dyn CpuBus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read(bus, 0x100 | u16::from(self.sp))
    }
    fn flag(&mut self, flag: u8, set: bool) {
        if set {
            self.status |= flag;
        } else {
            self.status &= !flag;
        }
    }
    fn nz(&mut self, value: u8) {
        self.flag(N, value & N != 0);
        self.flag(Z, value == 0);
    }
    fn adc(&mut self, value: u8) {
        let sum = u16::from(self.a) + u16::from(value) + u16::from(self.status & C);
        let result = sum as u8;
        self.flag(C, sum > 255);
        self.flag(V, (!(self.a ^ value) & (self.a ^ result) & 0x80) != 0);
        self.a = result;
        self.nz(result);
    }
    fn compare(&mut self, reg: u8, value: u8) {
        self.flag(C, reg >= value);
        self.nz(reg.wrapping_sub(value));
    }
    pub fn reset(&mut self, bus: &mut dyn CpuBus) {
        self.read(bus, self.pc);
        self.read(bus, self.pc);
        for _ in 0..3 {
            self.read(bus, 0x100 | u16::from(self.sp));
            self.sp = self.sp.wrapping_sub(1);
        }
        let lo = self.read(bus, 0xfffc);
        let hi = self.read(bus, 0xfffd);
        self.pc = u16::from_le_bytes([lo, hi]);
        self.status = (self.status | U | I) & !B;
        self.irq_mask = true;
    }
    fn interrupt(&mut self, bus: &mut dyn CpuBus, vector: u16) {
        self.read(bus, self.pc);
        self.read(bus, self.pc);
        self.push(bus, (self.pc >> 8) as u8);
        self.push(bus, self.pc as u8);
        self.push(bus, (self.status | U) & !B);
        self.status |= I;
        self.irq_mask = true;
        let lo = self.read(bus, vector);
        let hi = self.read(bus, vector + 1);
        self.pc = u16::from_le_bytes([lo, hi]);
    }
    /// Execute one instruction or interrupt sequence, returning its CPU cycles.
    /// Unsupported unstable opcodes and JAM return an error after opcode fetch.
    pub fn step(&mut self, bus: &mut dyn CpuBus) -> Result<u64, CpuError> {
        let start = self.cycles;
        if bus.take_nmi() {
            self.interrupt(bus, 0xfffa);
            return Ok(self.cycles - start);
        }
        if bus.irq_pending() && !self.irq_mask {
            self.interrupt(bus, 0xfffe);
            return Ok(self.cycles - start);
        }
        let pc = self.pc;
        let opcode = self.fetch(bus);
        let (op, mode) = DECODE[usize::from(opcode)];
        if op == Op::Jam {
            return Err(CpuError { opcode, pc });
        }
        let old_i = self.status & I != 0;
        use Op::*;
        match op {
            Brk => {
                self.fetch(bus);
                self.push(bus, (self.pc >> 8) as u8);
                self.push(bus, self.pc as u8);
                self.push(bus, self.status | B | U);
                self.status |= I;
                let lo = self.read(bus, 0xfffe);
                let hi = self.read(bus, 0xffff);
                self.pc = u16::from_le_bytes([lo, hi]);
            }
            Jsr => {
                let lo = self.fetch(bus);
                self.read(bus, 0x100 | u16::from(self.sp));
                self.push(bus, (self.pc >> 8) as u8);
                self.push(bus, self.pc as u8);
                let hi = self.fetch(bus);
                self.pc = u16::from_le_bytes([lo, hi]);
            }
            Jmp => {
                let addr = self.word(bus);
                self.pc = if mode == Mode::Ind {
                    let lo = self.read(bus, addr);
                    let hi = self.read(
                        bus,
                        (addr & 0xff00) | u16::from((addr as u8).wrapping_add(1)),
                    );
                    u16::from_le_bytes([lo, hi])
                } else {
                    addr
                };
            }
            Rts | Rti => {
                self.read(bus, self.pc);
                self.read(bus, 0x100 | u16::from(self.sp));
                if op == Rti {
                    self.status = (self.pull(bus) | U) & !B;
                }
                let lo = self.pull(bus);
                let hi = self.pull(bus);
                self.pc = u16::from_le_bytes([lo, hi]);
                if op == Rts {
                    self.read(bus, self.pc);
                    self.pc = self.pc.wrapping_add(1);
                }
            }
            Pha | Php => {
                self.read(bus, self.pc);
                self.push(
                    bus,
                    if op == Pha {
                        self.a
                    } else {
                        self.status | B | U
                    },
                );
            }
            Pla | Plp => {
                self.read(bus, self.pc);
                self.read(bus, 0x100 | u16::from(self.sp));
                let v = self.pull(bus);
                if op == Pla {
                    self.a = v;
                    self.nz(v);
                } else {
                    self.status = (v | U) & !B;
                }
            }
            Bpl | Bmi | Bvc | Bvs | Bcc | Bcs | Bne | Beq => {
                let offset = self.fetch(bus) as i8;
                let take = match op {
                    Bpl => self.status & N == 0,
                    Bmi => self.status & N != 0,
                    Bvc => self.status & V == 0,
                    Bvs => self.status & V != 0,
                    Bcc => self.status & C == 0,
                    Bcs => self.status & C != 0,
                    Bne => self.status & Z == 0,
                    _ => self.status & Z != 0,
                };
                if take {
                    self.read(bus, self.pc);
                    let target = self.pc.wrapping_add_signed(i16::from(offset));
                    if self.pc & 0xff00 != target & 0xff00 {
                        self.read(bus, (self.pc & 0xff00) | (target & 0xff));
                    }
                    self.pc = target;
                }
            }
            _ if mode == Mode::Imp => {
                self.read(bus, self.pc);
                match op {
                    Clc => self.status &= !C,
                    Sec => self.status |= C,
                    Cli => self.status &= !I,
                    Sei => self.status |= I,
                    Cld => self.status &= !D,
                    Sed => self.status |= D,
                    Clv => self.status &= !V,
                    Tax => {
                        self.x = self.a;
                        self.nz(self.x);
                    }
                    Tay => {
                        self.y = self.a;
                        self.nz(self.y);
                    }
                    Txa => {
                        self.a = self.x;
                        self.nz(self.a);
                    }
                    Tya => {
                        self.a = self.y;
                        self.nz(self.a);
                    }
                    Tsx => {
                        self.x = self.sp;
                        self.nz(self.x);
                    }
                    Txs => self.sp = self.x,
                    Dex => {
                        self.x = self.x.wrapping_sub(1);
                        self.nz(self.x);
                    }
                    Dey => {
                        self.y = self.y.wrapping_sub(1);
                        self.nz(self.y);
                    }
                    Inx => {
                        self.x = self.x.wrapping_add(1);
                        self.nz(self.x);
                    }
                    Iny => {
                        self.y = self.y.wrapping_add(1);
                        self.nz(self.y);
                    }
                    Nop => {}
                    _ => unreachable!(),
                }
            }
            _ if mode == Mode::Acc => {
                self.read(bus, self.pc);
                self.a = self.modify(op, self.a);
            }
            _ => {
                let write = matches!(op, Sta | Stx | Sty | Sax);
                let rmw = matches!(
                    op,
                    Asl | Lsr | Rol | Ror | Inc | Dec | Slo | Sre | Rla | Rra | Dcp | Isb
                );
                let addr = self.address(bus, mode, write || rmw);
                if write {
                    let v = match op {
                        Sta => self.a,
                        Stx => self.x,
                        Sty => self.y,
                        _ => self.a & self.x,
                    };
                    self.write(bus, addr, v);
                } else {
                    let value = self.read(bus, addr);
                    if rmw {
                        self.write(bus, addr, value);
                        let result = self.modify(op, value);
                        self.write(bus, addr, result);
                    } else {
                        self.apply(op, value);
                    }
                }
            }
        }
        // CLI/SEI/PLP change I after the interrupt polling point. RTI polls the restored I.
        self.irq_mask = if matches!(op, Cli | Sei | Plp) {
            old_i
        } else {
            self.status & I != 0
        };
        Ok(self.cycles - start)
    }
    fn address(&mut self, bus: &mut dyn CpuBus, mode: Mode, always_dummy: bool) -> u16 {
        use Mode::*;
        match mode {
            Imm => {
                let addr = self.pc;
                self.pc = self.pc.wrapping_add(1);
                addr
            }
            Zp => u16::from(self.fetch(bus)),
            Zpx | Zpy => {
                let base = self.fetch(bus);
                self.read(bus, u16::from(base));
                u16::from(base.wrapping_add(if mode == Zpx { self.x } else { self.y }))
            }
            Abs => self.word(bus),
            Abx | Aby => {
                let base = self.word(bus);
                self.indexed(
                    bus,
                    base,
                    if mode == Abx { self.x } else { self.y },
                    always_dummy,
                )
            }
            Izx => {
                let base = self.fetch(bus);
                self.read(bus, u16::from(base));
                let ptr = base.wrapping_add(self.x);
                let lo = self.read(bus, u16::from(ptr));
                let hi = self.read(bus, u16::from(ptr.wrapping_add(1)));
                u16::from_le_bytes([lo, hi])
            }
            Izy => {
                let ptr = self.fetch(bus);
                let lo = self.read(bus, u16::from(ptr));
                let hi = self.read(bus, u16::from(ptr.wrapping_add(1)));
                self.indexed(bus, u16::from_le_bytes([lo, hi]), self.y, always_dummy)
            }
            _ => unreachable!(),
        }
    }
    fn indexed(&mut self, bus: &mut dyn CpuBus, base: u16, index: u8, always_dummy: bool) -> u16 {
        let addr = base.wrapping_add(u16::from(index));
        if always_dummy || base & 0xff00 != addr & 0xff00 {
            self.read(bus, (base & 0xff00) | (addr & 0xff));
        }
        addr
    }
    fn modify(&mut self, op: Op, value: u8) -> u8 {
        use Op::*;
        let carry = self.status & C;
        let result = match op {
            Asl | Slo => {
                self.flag(C, value & 0x80 != 0);
                value << 1
            }
            Lsr | Sre => {
                self.flag(C, value & 1 != 0);
                value >> 1
            }
            Rol | Rla => {
                self.flag(C, value & 0x80 != 0);
                (value << 1) | carry
            }
            Ror | Rra => {
                self.flag(C, value & 1 != 0);
                (value >> 1) | (carry << 7)
            }
            Inc | Isb => value.wrapping_add(1),
            Dec | Dcp => value.wrapping_sub(1),
            _ => unreachable!(),
        };
        self.nz(result);
        match op {
            Slo => {
                self.a |= result;
                self.nz(self.a);
            }
            Sre => {
                self.a ^= result;
                self.nz(self.a);
            }
            Rla => {
                self.a &= result;
                self.nz(self.a);
            }
            Rra => self.adc(result),
            Dcp => self.compare(self.a, result),
            Isb => self.adc(!result),
            _ => {}
        }
        result
    }
    fn apply(&mut self, op: Op, value: u8) {
        use Op::*;
        match op {
            Ora => {
                self.a |= value;
                self.nz(self.a);
            }
            And => {
                self.a &= value;
                self.nz(self.a);
            }
            Eor => {
                self.a ^= value;
                self.nz(self.a);
            }
            Adc => self.adc(value),
            Sbc => self.adc(!value),
            Lda => {
                self.a = value;
                self.nz(value);
            }
            Ldx => {
                self.x = value;
                self.nz(value);
            }
            Ldy => {
                self.y = value;
                self.nz(value);
            }
            Lax => {
                self.a = value;
                self.x = value;
                self.nz(value);
            }
            Cmp => self.compare(self.a, value),
            Cpx => self.compare(self.x, value),
            Cpy => self.compare(self.y, value),
            Bit => {
                self.flag(Z, self.a & value == 0);
                self.flag(V, value & V != 0);
                self.flag(N, value & N != 0);
            }
            Anc => {
                self.a &= value;
                self.nz(self.a);
                self.flag(C, self.a & N != 0);
            }
            Alr => {
                self.a &= value;
                self.a = self.modify(Lsr, self.a);
            }
            Arr => {
                self.a = ((self.a & value) >> 1) | ((self.status & C) << 7);
                self.nz(self.a);
                self.flag(C, self.a & 0x40 != 0);
                self.flag(V, ((self.a >> 6) ^ (self.a >> 5)) & 1 != 0);
            }
            Axs => {
                let reg = self.a & self.x;
                self.x = reg.wrapping_sub(value);
                self.flag(C, reg >= value);
                self.nz(self.x);
            }
            Nop => {}
            _ => unreachable!(),
        }
    }
}
