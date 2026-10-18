use std::{
    collections::HashMap,
    error::Error,
    fmt::LowerHex,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
    str::FromStr,
};
use strum::{EnumIter, EnumString, FromRepr, IntoEnumIterator};
use bitbybit::{bitenum, bitfield};

const ROM_LOOKUP_ADDR_WIDTH: usize = 12;
const PC_ADDR_WIDTH: usize = 12;

const OPCODE_WIDTH: usize = 4;
const OPCODE_COUNT: usize = 2_usize.pow(OPCODE_WIDTH as u32);

const STEPS_PER_OPCODE: usize = 16;

const ROM_WORDS: usize = 2_usize.pow(ROM_LOOKUP_ADDR_WIDTH as u32);
const RAM_WORDS: usize = 2_usize.pow(PC_ADDR_WIDTH as u32);

// Bit positions, counting from the least significant bit.
// const A_WE: usize = 0;
// const B_WE: usize = 1;
// const C_WE: usize = 2;

// const MAR_BUS_WE: usize = 3;
// const MAR_INC: usize = 4;

// const RAM_MSB_WE: usize = 5;
// const RAM_LSB_WE: usize = 6;

// const PC_INC: usize = 7;
// const PC_JMP: usize = 8;

// const INST_MSB_WE: usize = 9;
// const INST_LSB_WE: usize = 10;

// const BUS_SOURCE: usize = 11; // 1<<2 = 100 101 << 11
// const INST_STEP_RST: usize = 14;
// const HLT: usize = 15;

// const ALU_SUB: usize = 16;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumString, FromRepr)]
#[strum(ascii_case_insensitive)]
#[repr(u8)]
enum Op {
    /// No op, does nothing but reset the instruction step counter
    Nop = 0,
    /// Load A, (LDA ADDR) means put the contents of RAM[addr] -> A
    Lda = 1,
    /// (ADD addr) take contents of RAM[addr] -> B, set ALU to addition, and write ALU_OUT -> A
    Add = 2,
    /// same as (ADD addr), except sets ALU to subtraction
    Sub = 3,
    /// (STA addr), store the contents of A into ram[addr]
    Sta = 4,
    /// (LDI data) store the immediate data in the lower 12 bits of the instruction into register A
    Ldi = 5,
    /// (JMP addr) set the program counter to addr
    Jmp = 6,
    /// (JMP addr) conditionally jump the PC to addr if the alu output is zero
    Jz = 7,
    ///
    Hlt = 15,
}

impl Op {
    fn requires_operand(self) -> bool {
        // !matches!(self, Self::Nop | Self::Hlt)
        match self {
            Self::Nop | Self::Hlt => false,

            Self::Lda | Self::Add | Self::Sub | Self::Sta | Self::Ldi | Self::Jmp | Self::Jz => {
                true
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Instruction {
    op: Op,
    operand: u8,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, FromRepr)]
#[repr(u8)]
enum BusSource {
    #[default]
    A = 0,
    B = 1,
    C = 2,
    Alu = 3,
    Ram = 4,
    Pc = 5,
    Inst = 6,
}

/// One complete microinstruction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Control {
    a_we: bool,
    b_we: bool,
    c_we: bool,

    mar_bus_we: bool,
    mar_inc: bool,
    
    ram_msb_we: bool,
    ram_lsb_we: bool,
    
    pc_inc: bool,
    pc_jmp: bool,
    
    inst_msb_we: bool,
    inst_lsb_we: bool,
    
    bus_source: BusSource,
    inst_step_rst: bool,
    hlt: bool,
    alu_sub: bool,
}

fn encode(c: &Control) -> u16 {
    (u16::from(c.a_we) << A_WE)
        | (u16::from(c.b_we) << B_WE)
        | (u16::from(c.c_we) << C_WE)
        | (u16::from(c.mar_bus_we) << MAR_BUS_WE)
        | (u16::from(c.mar_inc) << MAR_INC)
        | (u16::from(c.ram_msb_we) << RAM_MSB_WE)
        | (u16::from(c.ram_lsb_we) << RAM_LSB_WE)
        | (u16::from(c.pc_inc) << PC_INC)
        | (u16::from(c.pc_jmp) << PC_JMP)
        | (u16::from(c.inst_msb_we) << INST_MSB_WE)
        | ((c.bus_source as u16) << BUS_SOURCE)
        | (u16::from(c.inst_step_rst) << INST_STEP_RST)
        | (u16::from(c.hlt) << HLT)
        | (u16::from(c.alu_sub) << ALU_SUB)
}

fn decode(word: u16) -> Option<Control> {
    let bit = |position| word & (1u16 << position) != 0;
    let source = ((word >> BUS_SOURCE) & 0b111) as u8;

    // Bits 0..=2 are unused and ignored.
    Some(Control {
        a_we: bit(A_WE),
        b_we: bit(B_WE),
        c_we: bit(C_WE),
        mar_we: bit(MAR_WE),
        ram_we: bit(RAM_WE),
        pc_inc: bit(PC_INC),
        pc_jmp: bit(PC_JMP),
        inst_we: bit(INST_WE),
        bus_source: BusSource::from_repr(source)?,
        inst_step_rst: bit(INST_STEP_RST),
        hlt: bit(HLT),
        alu_sub: bit(ALU_SUB),
    })
}

/// Execution only; fetch is added by create_rom.
fn execution_steps(op: Op) -> Vec<Control> {
    // Keep reset separate because the circuit resets the step counter
    // asynchronously.
    let end = Control {
        inst_step_rst: true,
        ..Default::default()
    };

    match op {
        Op::Nop => vec![end],
        Op::Lda => vec![
            Control {
                bus_source: BusSource::Inst,
                mar_we: true,
                ..Default::default()
            },
            Control {
                bus_source: BusSource::Ram,
                a_we: true,
                ..Default::default()
            },
            end,
        ],
        Op::Add => vec![
            Control {
                bus_source: BusSource::Inst,
                mar_we: true,
                ..Default::default()
            },
            Control {
                bus_source: BusSource::Ram,
                b_we: true,
                ..Default::default()
            },
            Control {
                bus_source: BusSource::Alu,
                a_we: true,
                ..Default::default()
            },
            end,
        ],
        Op::Sub => vec![
            Control {
                bus_source: BusSource::Inst,
                mar_we: true,
                ..Default::default()
            },
            Control {
                bus_source: BusSource::Ram,
                b_we: true,
                ..Default::default()
            },
            Control {
                bus_source: BusSource::Alu,
                a_we: true,
                alu_sub: true,
                ..Default::default()
            },
            end,
        ],
        Op::Sta => vec![
            Control {
                bus_source: BusSource::Inst,
                mar_we: true,
                ..Default::default()
            },
            Control {
                bus_source: BusSource::A,
                ram_we: true,
                ..Default::default()
            },
            end,
        ],
        Op::Ldi => vec![
            Control {
                bus_source: BusSource::Inst,
                a_we: true,
                ..Default::default()
            },
            end,
        ],
        Op::Jmp => vec![
            Control {
                bus_source: BusSource::Inst,
                pc_jmp: true,
                ..Default::default()
            },
            end,
        ],
        // caution, these instructions are only for when the jump condition is satisfied
        Op::Jz => vec![
            Control {
                bus_source: BusSource::Inst,
                pc_jmp: true,
                ..Default::default()
            },
            Control {
                inst_step_rst: true,
                ..Default::default()
            },
        ],
        Op::Hlt => vec![Control {
            hlt: true,
            ..Default::default()
        }],
    }
}

fn create_rom() -> [u16; ROM_WORDS] {
    let mut rom = [0_u16; ROM_WORDS];
    // opcode:xxxx step:xxxx control:xxxx
    // regardless of what opcode and control are, if step is 0000 or 0001 then we are fetching.
    // more generally, if step <= fetch.len()
    let fetch0 = Control {
        mar_we: true,
        bus_source: BusSource::Pc,
        ..Default::default()
    };
    let fetch1 = Control {
        inst_we: true,
        pc_inc: true,
        bus_source: BusSource::Ram,
        ..Default::default()
    };

    let fetch = vec![encode(&fetch0), encode(&fetch1)];
    let fetch_len = fetch.len();
    let exec_steps = Op::iter()
        .map(|o| (o, execution_steps(o)))
        .collect::<HashMap<Op, Vec<Control>>>();

    // addr will range from 0..ROM_WORDS (ie 12 bits)
    for (addr, ctrl) in rom.iter_mut().enumerate() {
        let op = addr >> 8 & 0b1111;
        let step = addr >> 4 & 0b1111;
        let flags = addr & 0b1111;
        let alu_is_zero = (addr & 1) != 0;

        if let Some(&fetch_ctrl) = fetch.get(step) {
            *ctrl = fetch_ctrl;
        }

        if let Some(o) = Op::from_repr(op as u8) {
            if step < fetch_len {
                continue;
            }
            if let Some(c) = exec_steps[&o].get(step - fetch_len) {
                if o == Op::Jz && !alu_is_zero {
                    *ctrl = encode(&exec_steps[&Op::Nop][0])
                } else {
                    *ctrl = encode(c);
                }
            }
        }
    }

    rom
}

fn parse_instruction(source: &str) -> Result<Instruction, String> {
    let mut words = source.split_whitespace();
    let mnemonic = words.next().ok_or("expected an instruction")?;
    let op: Op = mnemonic.parse().map_err(|_| "unknown op")?;

    let operand = match words.next() {
        Some(text) => text
            .parse::<u8>()
            .map_err(|_| format!("invalid operand {text:?}: expected 0..=15"))?,
        None if op.requires_operand() => {
            return Err(format!("{mnemonic} requires an operand"));
        }
        None => 0,
    };

    if operand > 15 {
        return Err(format!("operand {operand} is outside 0..=15"));
    }

    if let Some(extra) = words.next() {
        return Err(format!("unexpected extra token {extra:?}"));
    }

    Ok(Instruction { op, operand })
}

fn encode_instruction(inst: Instruction) -> u8 {
    // The parser validates user input; this catches programming mistakes.
    assert!(inst.operand <= 15, "operand must fit in four bits");
    ((inst.op as u8) << 4) | inst.operand
}

fn decode_instruction(byte: u8) -> Option<Instruction> {
    Some(Instruction {
        op: Op::from_repr(byte >> 4)?,
        operand: byte & 0x0f,
    })
}

fn assemble(source: &str) -> Result<[u8; RAM_WORDS], String> {
    let mut ram = [0; RAM_WORDS];
    let mut address = 0;

    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        let code = line
            .split_once(';')
            .map_or(line, |(before, _)| before)
            .trim();

        if code.is_empty() {
            continue;
        }

        if address == ram.len() {
            return Err(format!(
                "line {line_number}: program exceeds {RAM_WORDS} bytes"
            ));
        }

        let instruction =
            parse_instruction(code).map_err(|error| format!("line {line_number}: {error}"))?;

        ram[address] = encode_instruction(instruction);
        address += 1;
    }

    Ok(ram)
}

/// Formats either u8 RAM words or u16 ROM words.
/// Widths are minimum hexadecimal digit counts.
fn write_memory<T: LowerHex>(
    mem: &[T],
    writer: &mut impl Write,
    address_digits: usize,
    data_digits: usize,
) -> io::Result<()> {
    writeln!(writer, "v3.0 hex words addressed")?;

    for (row, words) in mem.chunks(16).enumerate() {
        let address = row * 16;
        write!(writer, "{address:0address_digits$x}:")?;

        for word in words {
            write!(writer, " {word:0data_digits$x}")?;
        }

        writeln!(writer)?;
    }

    Ok(())
}

fn save_memory<T: LowerHex>(
    path: impl AsRef<Path>,
    mem: &[T],
    address_digits: usize,
    data_digits: usize,
) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    write_memory(mem, &mut writer, address_digits, data_digits)?;
    writer.flush()
}

fn main() -> Result<(), Box<dyn Error>> {
    let source_path = std::env::args_os()
        .nth(1)
        .unwrap_or("programs/simple_program.txt".into());

    // Resolve paths relative to Cargo.toml, regardless of the working directory.
    let project = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = fs::read_to_string(source_path)?;

    // Generate both images before opening any output files.
    // let ram = assemble(&source)?;
    let ram = [0; 2_usize.pow(12)];
    let rom = create_rom();

    // The same writer can render into a buffer for conversion to String.
    let mut buffer = Vec::new();
    write_memory(&rom, &mut buffer, 2, 4)?;
    let text = String::from_utf8(buffer)?;
    print!("{text}");

    // Save both images in the repository root.
    save_memory(project.join("../ROM.txt"), &rom, 2, 4)?;
    save_memory(project.join("ROM.txt"), &rom, 2, 4)?;

    save_memory(project.join("../RAM.txt"), &ram, 1, 2)?;
    save_memory(project.join("RAM.txt"), &ram, 1, 2)?;

    let instructions = ram.map(|b| decode_instruction(b));
    println!("{:?}", instructions);
    Ok(())
}
