use nes_core::cpu::{Cpu, CpuBus, B, C, D, I, N, U, V, Z};

#[derive(Debug, PartialEq, Eq)]
enum Access {
    Read(u16),
    Write(u16, u8),
}
struct Bus {
    mem: Box<[u8; 65536]>,
    accesses: Vec<Access>,
    nmi: bool,
    irq: bool,
}
impl Bus {
    fn new(program: &[u8], pc: u16) -> Self {
        let mut mem = Box::new([0; 65536]);
        for (i, v) in program.iter().enumerate() {
            mem[usize::from(pc.wrapping_add(i as u16))] = *v;
        }
        Self {
            mem,
            accesses: Vec::new(),
            nmi: false,
            irq: false,
        }
    }
}
impl CpuBus for Bus {
    fn read(&mut self, addr: u16) -> u8 {
        self.accesses.push(Access::Read(addr));
        self.mem[usize::from(addr)]
    }
    fn write(&mut self, addr: u16, v: u8) {
        self.accesses.push(Access::Write(addr, v));
        self.mem[usize::from(addr)] = v;
    }
    fn take_nmi(&mut self) -> bool {
        std::mem::take(&mut self.nmi)
    }
    fn irq_pending(&self) -> bool {
        self.irq
    }
}
fn setup(program: &[u8]) -> (Cpu, Bus) {
    let mut cpu = Cpu::new();
    cpu.pc = 0x8000;
    cpu.sp = 0xfd;
    (cpu, Bus::new(program, 0x8000))
}

#[test]
fn reset_has_seven_reads_no_stack_writes() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new(&[], 0);
    bus.mem[0xfffc] = 0x34;
    bus.mem[0xfffd] = 0x12;
    cpu.reset(&mut bus);
    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cpu.sp, 0xfd);
    assert_eq!(cpu.status, I | U);
    assert_eq!(cpu.cycles, 7);
    assert_eq!(
        bus.accesses,
        vec![
            Access::Read(0),
            Access::Read(0),
            Access::Read(0x100),
            Access::Read(0x1ff),
            Access::Read(0x1fe),
            Access::Read(0xfffc),
            Access::Read(0xfffd)
        ]
    );
}
#[test]
fn all_official_opcodes_have_base_cycle_counts() {
    // Independent NMOS 6502 official-opcode base timing table. Branches are
    // configured not taken, indexes zero, and no page crossing in this fixture.
    let expected = [
        (0x00, 7),
        (0x01, 6),
        (0x05, 3),
        (0x06, 5),
        (0x08, 3),
        (0x09, 2),
        (0x0a, 2),
        (0x0d, 4),
        (0x0e, 6),
        (0x10, 2),
        (0x11, 5),
        (0x15, 4),
        (0x16, 6),
        (0x18, 2),
        (0x19, 4),
        (0x1d, 4),
        (0x1e, 7),
        (0x20, 6),
        (0x21, 6),
        (0x24, 3),
        (0x25, 3),
        (0x26, 5),
        (0x28, 4),
        (0x29, 2),
        (0x2a, 2),
        (0x2c, 4),
        (0x2d, 4),
        (0x2e, 6),
        (0x30, 2),
        (0x31, 5),
        (0x35, 4),
        (0x36, 6),
        (0x38, 2),
        (0x39, 4),
        (0x3d, 4),
        (0x3e, 7),
        (0x40, 6),
        (0x41, 6),
        (0x45, 3),
        (0x46, 5),
        (0x48, 3),
        (0x49, 2),
        (0x4a, 2),
        (0x4c, 3),
        (0x4d, 4),
        (0x4e, 6),
        (0x50, 2),
        (0x51, 5),
        (0x55, 4),
        (0x56, 6),
        (0x58, 2),
        (0x59, 4),
        (0x5d, 4),
        (0x5e, 7),
        (0x60, 6),
        (0x61, 6),
        (0x65, 3),
        (0x66, 5),
        (0x68, 4),
        (0x69, 2),
        (0x6a, 2),
        (0x6c, 5),
        (0x6d, 4),
        (0x6e, 6),
        (0x70, 2),
        (0x71, 5),
        (0x75, 4),
        (0x76, 6),
        (0x78, 2),
        (0x79, 4),
        (0x7d, 4),
        (0x7e, 7),
        (0x81, 6),
        (0x84, 3),
        (0x85, 3),
        (0x86, 3),
        (0x88, 2),
        (0x8a, 2),
        (0x8c, 4),
        (0x8d, 4),
        (0x8e, 4),
        (0x90, 2),
        (0x91, 6),
        (0x94, 4),
        (0x95, 4),
        (0x96, 4),
        (0x98, 2),
        (0x99, 5),
        (0x9a, 2),
        (0x9d, 5),
        (0xa0, 2),
        (0xa1, 6),
        (0xa2, 2),
        (0xa4, 3),
        (0xa5, 3),
        (0xa6, 3),
        (0xa8, 2),
        (0xa9, 2),
        (0xaa, 2),
        (0xac, 4),
        (0xad, 4),
        (0xae, 4),
        (0xb0, 2),
        (0xb1, 5),
        (0xb4, 4),
        (0xb5, 4),
        (0xb6, 4),
        (0xb8, 2),
        (0xb9, 4),
        (0xba, 2),
        (0xbc, 4),
        (0xbd, 4),
        (0xbe, 4),
        (0xc0, 2),
        (0xc1, 6),
        (0xc4, 3),
        (0xc5, 3),
        (0xc6, 5),
        (0xc8, 2),
        (0xc9, 2),
        (0xca, 2),
        (0xcc, 4),
        (0xcd, 4),
        (0xce, 6),
        (0xd0, 2),
        (0xd1, 5),
        (0xd5, 4),
        (0xd6, 6),
        (0xd8, 2),
        (0xd9, 4),
        (0xdd, 4),
        (0xde, 7),
        (0xe0, 2),
        (0xe1, 6),
        (0xe4, 3),
        (0xe5, 3),
        (0xe6, 5),
        (0xe8, 2),
        (0xe9, 2),
        (0xea, 2),
        (0xec, 4),
        (0xed, 4),
        (0xee, 6),
        (0xf0, 2),
        (0xf1, 5),
        (0xf5, 4),
        (0xf6, 6),
        (0xf8, 2),
        (0xf9, 4),
        (0xfd, 4),
        (0xfe, 7),
    ];
    assert_eq!(expected.len(), 151);
    for (opcode, cycles) in expected {
        let (mut cpu, mut bus) = setup(&[opcode, 0, 0]);
        cpu.status = U | match opcode {
            0x10 => N,
            0x50 => V,
            0x90 => C,
            0xd0 => Z,
            _ => 0,
        };
        assert_eq!(cpu.step(&mut bus).unwrap(), cycles, "opcode {opcode:02x}");
        assert_eq!(bus.accesses.len() as u64, cycles, "opcode {opcode:02x}");
    }
}
#[test]
fn indexed_read_page_cross_and_store_dummy_reads_are_visible() {
    let (mut cpu, mut bus) = setup(&[0xbd, 0xff, 0x20, 0x9d, 0xff, 0x20]);
    cpu.x = 1;
    bus.mem[0x2100] = 0x73;
    assert_eq!(cpu.step(&mut bus).unwrap(), 5);
    assert_eq!(cpu.a, 0x73);
    assert_eq!(
        bus.accesses,
        vec![
            Access::Read(0x8000),
            Access::Read(0x8001),
            Access::Read(0x8002),
            Access::Read(0x2000),
            Access::Read(0x2100)
        ]
    );
    bus.accesses.clear();
    cpu.a = 0x42;
    assert_eq!(cpu.step(&mut bus).unwrap(), 5);
    assert_eq!(
        bus.accesses,
        vec![
            Access::Read(0x8003),
            Access::Read(0x8004),
            Access::Read(0x8005),
            Access::Read(0x2000),
            Access::Write(0x2100, 0x42)
        ]
    );
    let (mut cpu, mut bus) = setup(&[0x9d, 0x00, 0x20]);
    cpu.x = 1;
    assert_eq!(cpu.step(&mut bus).unwrap(), 5);
    assert_eq!(bus.accesses[3], Access::Read(0x2001));
}
#[test]
fn indirect_addressing_wraps_zero_page_and_jmp_has_hardware_bug() {
    let (mut cpu, mut bus) = setup(&[0xa1, 0xfe, 0xb1, 0xff, 0x6c, 0xff, 0x12]);
    cpu.x = 1;
    cpu.y = 1;
    bus.mem[0xff] = 0xff;
    bus.mem[0] = 0x20;
    bus.mem[0x20ff] = 0x42;
    bus.mem[0x2100] = 0x56;
    assert_eq!(cpu.step(&mut bus).unwrap(), 6);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(bus.accesses[2], Access::Read(0xfe));
    assert_eq!(bus.accesses[3], Access::Read(0xff));
    assert_eq!(bus.accesses[4], Access::Read(0));
    bus.accesses.clear();
    assert_eq!(cpu.step(&mut bus).unwrap(), 6);
    assert_eq!(cpu.a, 0x56);
    assert_eq!(bus.accesses[4], Access::Read(0x2000));
    bus.mem[0x12ff] = 0x78;
    bus.mem[0x1200] = 0x56;
    bus.mem[0x1300] = 0xab;
    assert_eq!(cpu.step(&mut bus).unwrap(), 5);
    assert_eq!(cpu.pc, 0x5678);
}
#[test]
fn zero_page_index_wraps_and_reads_unindexed_address() {
    let (mut cpu, mut bus) = setup(&[0xb5, 0xff]);
    cpu.x = 2;
    bus.mem[1] = 0x80;
    assert_eq!(cpu.step(&mut bus).unwrap(), 4);
    assert_eq!(cpu.a, 0x80);
    assert_eq!(bus.accesses[2], Access::Read(0xff));
    assert_eq!(bus.accesses[3], Access::Read(1));
    assert_eq!(cpu.status & (N | Z), N);
}
#[test]
fn branch_penalties_and_wrong_page_read() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x80fd;
    let mut bus = Bus::new(&[0xd0, 2], 0x80fd);
    assert_eq!(cpu.step(&mut bus).unwrap(), 4);
    assert_eq!(cpu.pc, 0x8101);
    assert_eq!(
        bus.accesses,
        vec![
            Access::Read(0x80fd),
            Access::Read(0x80fe),
            Access::Read(0x80ff),
            Access::Read(0x8001)
        ]
    );
    let (mut cpu, mut bus) = setup(&[0xd0, 0xfe]);
    assert_eq!(cpu.step(&mut bus).unwrap(), 3);
    assert_eq!(cpu.pc, 0x8000);
    cpu.status |= Z;
    assert_eq!(cpu.step(&mut bus).unwrap(), 2);
    assert_eq!(cpu.pc, 0x8002);
}
#[test]
fn rmw_writes_old_then_new_value_even_when_indexed() {
    let (mut cpu, mut bus) = setup(&[0x1e, 0xff, 0x20]);
    cpu.x = 1;
    bus.mem[0x2100] = 0x81;
    assert_eq!(cpu.step(&mut bus).unwrap(), 7);
    assert_eq!(
        bus.accesses[3..],
        [
            Access::Read(0x2000),
            Access::Read(0x2100),
            Access::Write(0x2100, 0x81),
            Access::Write(0x2100, 2)
        ]
    );
    assert_eq!(cpu.status & (C | Z | N), C);
}
#[test]
fn jsr_rts_stack_order_and_return_address() {
    let (mut cpu, mut bus) = setup(&[0x20, 0x00, 0x90, 0xea]);
    bus.mem[0x9000] = 0x60;
    assert_eq!(cpu.step(&mut bus).unwrap(), 6);
    assert_eq!(cpu.pc, 0x9000);
    assert_eq!(cpu.sp, 0xfb);
    assert_eq!(
        bus.accesses,
        vec![
            Access::Read(0x8000),
            Access::Read(0x8001),
            Access::Read(0x1fd),
            Access::Write(0x1fd, 0x80),
            Access::Write(0x1fc, 2),
            Access::Read(0x8002)
        ]
    );
    bus.accesses.clear();
    assert_eq!(cpu.step(&mut bus).unwrap(), 6);
    assert_eq!(cpu.pc, 0x8003);
    assert_eq!(cpu.sp, 0xfd);
    assert_eq!(
        bus.accesses,
        vec![
            Access::Read(0x9000),
            Access::Read(0x9001),
            Access::Read(0x1fb),
            Access::Read(0x1fc),
            Access::Read(0x1fd),
            Access::Read(0x8002)
        ]
    );
}
#[test]
fn brk_pushes_pc_plus_two_and_break_only_in_stack_status_rti_restores() {
    let (mut cpu, mut bus) = setup(&[0, 0xea]);
    cpu.status = C | U;
    bus.mem[0xfffe] = 0;
    bus.mem[0xffff] = 0x90;
    bus.mem[0x9000] = 0x40;
    assert_eq!(cpu.step(&mut bus).unwrap(), 7);
    assert_eq!(cpu.pc, 0x9000);
    assert_eq!(bus.mem[0x1fd], 0x80);
    assert_eq!(bus.mem[0x1fc], 2);
    assert_eq!(bus.mem[0x1fb], U | B | C);
    assert_eq!(cpu.status, U | I | C);
    assert_eq!(cpu.step(&mut bus).unwrap(), 6);
    assert_eq!(cpu.pc, 0x8002);
    assert_eq!(cpu.status, U | C);
    assert_eq!(cpu.sp, 0xfd);
}
#[test]
fn nmi_precedes_irq_and_latched_edge_is_consumed_once() {
    let (mut cpu, mut bus) = setup(&[0xea]);
    bus.nmi = true;
    bus.irq = true;
    bus.mem[0xfffa] = 0;
    bus.mem[0xfffb] = 0x90;
    bus.mem[0x9000] = 0xea;
    assert_eq!(cpu.step(&mut bus).unwrap(), 7);
    assert_eq!(cpu.pc, 0x9000);
    assert_eq!(bus.mem[0x1fb] & B, 0);
    assert_eq!(bus.mem[0x1fc], 0);
    assert_eq!(cpu.step(&mut bus).unwrap(), 2);
    assert_eq!(cpu.pc, 0x9001);
}
#[test]
fn cli_delays_irq_one_instruction_and_sei_uses_old_mask() {
    let (mut cpu, mut bus) = setup(&[0x58, 0xea, 0xea]);
    bus.irq = true;
    bus.mem[0xfffe] = 0;
    bus.mem[0xffff] = 0x90;
    assert_eq!(cpu.step(&mut bus).unwrap(), 2);
    assert_eq!(cpu.status & I, 0);
    assert_eq!(cpu.step(&mut bus).unwrap(), 2);
    assert_eq!(cpu.pc, 0x8002);
    assert_eq!(cpu.step(&mut bus).unwrap(), 7);
    assert_eq!(cpu.pc, 0x9000);
    let (mut cpu, mut bus) = setup(&[0x58, 0xea, 0x78, 0xea]);
    cpu.step(&mut bus).unwrap();
    cpu.step(&mut bus).unwrap();
    cpu.step(&mut bus).unwrap();
    bus.irq = true;
    bus.mem[0xffff] = 0x90;
    assert_eq!(cpu.step(&mut bus).unwrap(), 7);
    assert_eq!(cpu.pc, 0x9000);
}
#[test]
fn stack_wraps_and_php_plp_do_not_store_break_flag_in_cpu() {
    let (mut cpu, mut bus) = setup(&[0x08, 0x28]);
    cpu.sp = 0;
    cpu.status = U | D | C;
    assert_eq!(cpu.step(&mut bus).unwrap(), 3);
    assert_eq!(cpu.sp, 255);
    assert_eq!(bus.mem[0x100], U | B | D | C);
    cpu.status = U;
    assert_eq!(cpu.step(&mut bus).unwrap(), 4);
    assert_eq!(cpu.sp, 0);
    assert_eq!(cpu.status, U | D | C);
}
#[test]
fn adc_sbc_exhaustive_binary_results_and_flags_even_with_decimal_set() {
    let (mut cpu, mut bus) = setup(&[0x69, 0]);
    for opcode in [0x69, 0xe9] {
        bus.mem[0x8000] = opcode;
        for a in 0..=255u8 {
            for b in 0..=255u8 {
                for carry in 0..=1u8 {
                    cpu.a = a;
                    cpu.pc = 0x8000;
                    cpu.status = U | D | carry;
                    bus.mem[0x8001] = b;
                    bus.accesses.clear();
                    cpu.step(&mut bus).unwrap();
                    let arithmetic = if opcode == 0x69 {
                        i16::from(a) + i16::from(b) + i16::from(carry)
                    } else {
                        i16::from(a) - i16::from(b) - i16::from(1 - carry)
                    };
                    let result = arithmetic as u8;
                    let overflow = if opcode == 0x69 {
                        (i16::from(a as i8) + i16::from(b as i8) + i16::from(carry)) < -128
                            || (i16::from(a as i8) + i16::from(b as i8) + i16::from(carry)) > 127
                    } else {
                        (i16::from(a as i8) - i16::from(b as i8) - i16::from(1 - carry)) < -128
                            || (i16::from(a as i8) - i16::from(b as i8) - i16::from(1 - carry))
                                > 127
                    };
                    let c = if opcode == 0x69 {
                        arithmetic > 255
                    } else {
                        arithmetic >= 0
                    };
                    let flags = (if c { C } else { 0 })
                        | (if result == 0 { Z } else { 0 })
                        | (result & N)
                        | (if overflow { V } else { 0 });
                    assert_eq!(cpu.a, result);
                    assert_eq!(
                        cpu.status & (C | Z | V | N),
                        flags,
                        "opcode={opcode:02x} a={a} b={b} c={carry}"
                    );
                    assert_ne!(cpu.status & D, 0);
                }
            }
        }
    }
}
#[test]
fn transfers_logic_compare_bit_and_register_wraparound() {
    let (mut cpu, mut bus) = setup(&[
        0xa9, 0x80, 0xaa, 0xe8, 0x8a, 0x49, 0xff, 0x29, 0x0f, 0x09, 0x80, 0xc9, 0x8e, 0xa0, 0,
        0x88, 0xc0, 0xff, 0x24, 0x10,
    ]);
    bus.mem[0x10] = 0x40;
    for _ in 0..12 {
        cpu.step(&mut bus).unwrap();
    }
    assert_eq!(cpu.a, 0x8e);
    assert_eq!(cpu.x, 0x81);
    assert_eq!(cpu.y, 0xff);
    assert_eq!(cpu.status & (C | Z | V | N), C | Z | V);
}
#[test]
fn unofficial_rmw_families_update_memory_accumulator_and_flags() {
    let cases = [
        (0x07, 0x81, 0x10, 0, 2, 0x12, C),        // SLO
        (0x47, 0x03, 0x80, 0, 1, 0x81, C | N),    // SRE
        (0x27, 0x80, 0xff, C, 1, 1, C),           // RLA
        (0x67, 0x02, 0x7f, 0, 1, 0x80, N | V),    // RRA uses rotate carry for ADC
        (0xc7, 0x11, 0x10, 0, 0x10, 0x10, C | Z), // DCP
        (0xe7, 0x0f, 0x10, C, 0x10, 0, C | Z),    // ISB
    ];
    for (opcode, value, a, flags, expected_mem, expected_a, expected_flags) in cases {
        let (mut cpu, mut bus) = setup(&[opcode, 0x10]);
        cpu.a = a;
        cpu.status = U | flags;
        bus.mem[0x10] = value;
        assert_eq!(cpu.step(&mut bus).unwrap(), 5);
        assert_eq!(bus.mem[0x10], expected_mem);
        assert_eq!(cpu.a, expected_a);
        assert_eq!(
            cpu.status & (C | Z | N | V),
            expected_flags,
            "opcode {opcode:02x}"
        );
        assert_eq!(bus.accesses[3], Access::Write(0x10, value));
        assert_eq!(bus.accesses[4], Access::Write(0x10, expected_mem));
    }
}
#[test]
fn stable_unofficial_addressing_modes_and_nops_have_expected_cycles() {
    for base in [0x03, 0x23, 0x43, 0x63, 0xc3, 0xe3] {
        for (delta, cycles) in [(0, 8), (4, 5), (12, 6), (16, 8), (20, 6), (24, 7), (28, 7)] {
            let (mut cpu, mut bus) = setup(&[base + delta, 0, 0]);
            assert_eq!(cpu.step(&mut bus).unwrap(), cycles);
        }
    }
    for (opcode, cycles) in [
        (0xa3, 6),
        (0xa7, 3),
        (0xaf, 4),
        (0xb3, 5),
        (0xb7, 4),
        (0xbf, 4),
        (0x83, 6),
        (0x87, 3),
        (0x8f, 4),
        (0x97, 4),
    ] {
        let (mut cpu, mut bus) = setup(&[opcode, 0, 0]);
        assert_eq!(cpu.step(&mut bus).unwrap(), cycles);
    }
    for opcode in [0x1a, 0x3a, 0x5a, 0x7a, 0xda, 0xfa] {
        let (mut cpu, mut bus) = setup(&[opcode]);
        assert_eq!(cpu.step(&mut bus).unwrap(), 2);
    }
    for opcode in [0x80, 0x82, 0x89, 0xc2, 0xe2] {
        let (mut cpu, mut bus) = setup(&[opcode, 0x42]);
        assert_eq!(cpu.step(&mut bus).unwrap(), 2);
        assert_eq!(cpu.pc, 0x8002);
    }
    for (opcodes, cycles) in [
        (vec![0x04, 0x44, 0x64], 3),
        (vec![0x14, 0x34, 0x54, 0x74, 0xd4, 0xf4], 4),
        (vec![0x0c], 4),
        (vec![0x1c, 0x3c, 0x5c, 0x7c, 0xdc, 0xfc], 4),
    ] {
        for opcode in opcodes {
            let (mut cpu, mut bus) = setup(&[opcode, 0, 0]);
            assert_eq!(cpu.step(&mut bus).unwrap(), cycles);
        }
    }
}
#[test]
fn lax_sax_and_unofficial_nop_read_operand() {
    let (mut cpu, mut bus) = setup(&[0xa7, 0x10, 0x87, 0x11, 0x1c, 0xff, 0x20]);
    bus.mem[0x10] = 0x81;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.a, 0x81);
    assert_eq!(cpu.x, 0x81);
    assert_eq!(cpu.status & (N | Z), N);
    cpu.a = 0x73;
    cpu.step(&mut bus).unwrap();
    assert_eq!(bus.mem[0x11], 1);
    bus.accesses.clear();
    assert_eq!(cpu.step(&mut bus).unwrap(), 5);
    assert_eq!(bus.accesses[3], Access::Read(0x2080));
    assert_eq!(bus.accesses[4], Access::Read(0x2180));
}
#[test]
fn unsupported_jam_is_explicit_error() {
    let (mut cpu, mut bus) = setup(&[0x02]);
    let error = cpu.step(&mut bus).unwrap_err();
    assert_eq!(error.opcode, 2);
    assert_eq!(error.pc, 0x8000);
    assert_eq!(bus.accesses.len(), 1);
}

#[test]
fn irq_stack_vector_and_rti_immediately_restore_interrupt_mask() {
    let (mut cpu, mut bus) = setup(&[0x58, 0xea, 0xea]);
    cpu.step(&mut bus).unwrap();
    cpu.step(&mut bus).unwrap();
    bus.irq = true;
    bus.mem[0xfffe] = 0x34;
    bus.mem[0xffff] = 0x90;
    bus.mem[0x9034] = 0x40;
    bus.accesses.clear();
    assert_eq!(cpu.step(&mut bus).unwrap(), 7);
    assert_eq!(
        bus.accesses,
        vec![
            Access::Read(0x8002),
            Access::Read(0x8002),
            Access::Write(0x1fd, 0x80),
            Access::Write(0x1fc, 2),
            Access::Write(0x1fb, U),
            Access::Read(0xfffe),
            Access::Read(0xffff)
        ]
    );
    assert_eq!(cpu.step(&mut bus).unwrap(), 6);
    assert_eq!(cpu.pc, 0x8002);
    assert_eq!(cpu.step(&mut bus).unwrap(), 7);
    assert_eq!(cpu.pc, 0x9034);
}
#[test]
fn absolute_indexed_and_indirect_y_loads_all_apply_page_cross_penalty() {
    for opcode in [0xb9, 0xbd, 0xbc, 0xbe, 0xbf] {
        let (mut cpu, mut bus) = setup(&[opcode, 0xff, 0x20]);
        cpu.x = 1;
        cpu.y = 1;
        bus.mem[0x2100] = 0x42;
        assert_eq!(cpu.step(&mut bus).unwrap(), 5, "opcode {opcode:02x}");
        assert_eq!(bus.accesses[3], Access::Read(0x2000));
        assert_eq!(bus.accesses[4], Access::Read(0x2100));
    }
    for opcode in [0x11, 0x31, 0x51, 0x71, 0xb1, 0xb3, 0xd1, 0xf1] {
        let (mut cpu, mut bus) = setup(&[opcode, 0xff]);
        cpu.y = 1;
        bus.mem[0xff] = 0xff;
        bus.mem[0] = 0x20;
        assert_eq!(cpu.step(&mut bus).unwrap(), 6, "opcode {opcode:02x}");
        assert_eq!(bus.accesses[4], Access::Read(0x2000));
        assert_eq!(bus.accesses[5], Access::Read(0x2100));
    }
}
#[test]
fn accumulator_shifts_and_rotates_preserve_unrelated_flags() {
    for (opcode, value, carry, result, flags) in [
        (0x0a, 0x80, 0, 0, C | Z),
        (0x4a, 1, 0, 0, C | Z),
        (0x2a, 0x80, 1, 1, C),
        (0x6a, 1, 1, 0x80, C | N),
    ] {
        let (mut cpu, mut bus) = setup(&[opcode]);
        cpu.a = value;
        cpu.status = U | V | D | I | carry;
        assert_eq!(cpu.step(&mut bus).unwrap(), 2);
        assert_eq!(cpu.a, result);
        assert_eq!(cpu.status, U | V | D | I | flags);
        assert_eq!(
            bus.accesses,
            vec![Access::Read(0x8000), Access::Read(0x8001)]
        );
    }
}
