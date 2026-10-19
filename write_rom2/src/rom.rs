//! Control signals, microcode, and control-ROM generation.

use bitbybit::{bitenum, bitfield};

use crate::isa::Op;

pub const ROM_ADDR_WIDTH: usize = 12;
pub const N_ROM_WORDS: usize = 1 << ROM_ADDR_WIDTH;

const END: Control = Control::DEFAULT.with_inst_step_rst(true);

static FETCH_STEPS: &[Control] = &[
    Control::DEFAULT
        .with_bus_source(BusSource::Pc)
        .with_mar_bus_we(true),
    Control::DEFAULT
        .with_bus_source(BusSource::Ram)
        .with_inst_msb_we(true),
    Control::DEFAULT.with_pc_inc(true),
    Control::DEFAULT
        .with_bus_source(BusSource::Pc)
        .with_mar_bus_we(true),
    Control::DEFAULT
        .with_bus_source(BusSource::Ram)
        .with_inst_lsb_we(true)
        .with_pc_inc(true),
];

static NOP_STEPS: &[Control] = &[END];

/// LDA 0x123
/// RAM 0x123: 0xAB
/// RAM 0x124: 0xCD
static LDA_STEPS: &[Control] = &[
    Control::DEFAULT
        .with_bus_source(BusSource::Inst)
        .with_mar_bus_we(true),
    Control::DEFAULT.with_mem_hi_we(true),
    Control::DEFAULT.with_mar_inc(true),
    Control::DEFAULT
        .with_bus_source(BusSource::Ram)
        .with_a_we(true),
    END,
];

static ADD_STEPS: &[Control] = &[
    Control::DEFAULT
        .with_bus_source(BusSource::Inst)
        .with_mar_bus_we(true),
    Control::DEFAULT.with_mem_hi_we(true),
    Control::DEFAULT.with_mar_inc(true),
    Control::DEFAULT
        .with_bus_source(BusSource::Ram)
        .with_b_we(true),
    Control::DEFAULT
        .with_bus_source(BusSource::Alu)
        .with_a_we(true),
    END,
];

/// This sequence also imposes big-endian since the first byte in memory is the MSB
static STA_STEPS: &[Control] = &[
    Control::DEFAULT
        .with_bus_source(BusSource::Inst)
        .with_mar_bus_we(true),
    Control::DEFAULT
        .with_bus_source(BusSource::A)
        .with_ram_msb_we(true),
    Control::DEFAULT.with_mar_inc(true),
    Control::DEFAULT
        .with_bus_source(BusSource::A)
        .with_ram_lsb_we(true),
    END,
];

static LDI_STEPS: &[Control] = &[
    Control::DEFAULT
        .with_bus_source(BusSource::Inst)
        .with_a_we(true),
    // Control::DEFAULT
    END,
];

static JMP_STEPS: &[Control] = &[
    Control::DEFAULT
        .with_bus_source(BusSource::Inst)
        .with_pc_jmp(true),
    // Control::DEFAULT
    END,
];

/// This sequence is the microinstructions to be performed provided the jump condition is already satisfied (ie ALU_OUT == 0)
static JZ_STEPS: &[Control] = &[
    Control::DEFAULT
        .with_bus_source(BusSource::Inst)
        .with_pc_jmp(true),
    // Control::DEFAULT
    END,
];

static HLT_STEPS: &[Control] = &[Control::DEFAULT.with_hlt(true)];

#[bitenum(u3, exhaustive = true)]
pub enum BusSource {
    A = 0,
    B = 1,
    C = 2,
    Alu = 3,
    Ram = 4,
    Pc = 5,
    Inst = 6,
    Man = 7,
}

#[bitfield(u32, default = 0, forbid_overlaps)]
pub struct Control {
    #[bit(0, rw)]
    a_we: bool,
    #[bit(1, rw)]
    b_we: bool,
    #[bit(2, rw)]
    c_we: bool,

    #[bit(3, rw)]
    mar_bus_we: bool,
    #[bit(4, rw)]
    mar_inc: bool,

    #[bit(5, rw)]
    ram_msb_we: bool,
    #[bit(6, rw)]
    ram_lsb_we: bool,

    #[bit(7, rw)]
    pc_inc: bool,
    #[bit(8, rw)]
    pc_jmp: bool,

    #[bit(9, rw)]
    inst_msb_we: bool,
    #[bit(10, rw)]
    inst_lsb_we: bool,

    #[bits(11..=13, rw)]
    bus_source: BusSource,
    #[bit(14, rw)]
    inst_step_rst: bool,
    #[bit(15, rw)]
    hlt: bool,
    #[bit(16, rw)]
    alu_sub: bool,

    #[bit(17, rw)]
    mem_hi_we: bool,
    #[bit(18, rw)]
    mem_hi_rst: bool,
}

fn execution_steps(op: Op) -> &'static [Control] {
    match op {
        Op::Nop => NOP_STEPS,
        Op::Lda => LDA_STEPS,
        Op::Add => ADD_STEPS,
        Op::Sub => NOP_STEPS, // DANGER DANGER
        Op::Sta => STA_STEPS,
        Op::Ldi => LDI_STEPS,
        Op::Jmp => JMP_STEPS,
        Op::Jz => JZ_STEPS,
        Op::Hlt => HLT_STEPS,
    }
}

/// Builds the control ROM indexed by opcode, microinstruction step, and flags.
pub fn create_rom() -> [u32; N_ROM_WORDS] {
    let n_fetch = FETCH_STEPS.len();
    let mut rom: [u32; N_ROM_WORDS] = [0; N_ROM_WORDS];
    for (addr, x) in rom.iter_mut().enumerate() {
        let opcode = addr >> 8;
        let step = (addr >> 4) & 0b1111;
        let control_flags = addr & 0b1111;
        let is_alu_zero = (control_flags & 1) != 0;
        if let Some(&fetch_control) = FETCH_STEPS.get(step) {
            *x = fetch_control.raw_value();
            continue;
        }

        if let Some(op) = Op::from_repr(opcode as u8) {
            if op == Op::Jz && !is_alu_zero {
                *x = END.raw_value();
                continue;
            }
            if let Some(&step_i) = execution_steps(op).get(step - n_fetch) {
                *x = step_i.raw_value()
            }
        }
    }

    rom
}
