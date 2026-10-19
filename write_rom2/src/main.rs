use std::{
    collections::HashMap,
    error::Error,
    fmt::LowerHex,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

use bitbybit::{bitenum, bitfield};
use strum::{EnumIter, EnumString, FromRepr};

const OPERAND_WIDTH: usize = 12;

const OPERAND_MAX_VALUE: usize = 2_usize.pow(OPERAND_WIDTH as u32) - 1;

const ROM_ADDR_WIDTH: usize = 12;
const ROM_DATA_WIDTH: usize = 32;
const N_ROM_WORDS: usize = 2_usize.pow(ROM_ADDR_WIDTH as u32);

const RAM_ADDR_WIDTH: usize = OPERAND_WIDTH;
const N_RAM_WORDS: usize = 2_usize.pow(RAM_ADDR_WIDTH as u32);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, EnumString, FromRepr)]
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
    operand: u16,
}

#[bitenum(u3, exhaustive = true)]
enum BusSource {
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
struct Control {
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

fn parse_instruction(
    source: &str,
    ctx: Option<&mut HashMap<String, usize>>,
) -> Result<Instruction, String> {
    let mut words = source.split_whitespace();
    let mnemonic = words.next().ok_or("expected an instruction")?;
    let op: Op = mnemonic.parse().map_err(|_| "unknown op")?;

    let operand = match words.next() {
        Some(text) => {
            let mut res: u16 = 0;
            if let Some(c) = ctx {
                if let Some(v) = c.get(text) {
                    res = *v as u16
                }
            } else {
                res = text
                .parse::<u16>()
                .map_err(|_| format!("invalid operand {text:?}: expected 0..={OPERAND_MAX_VALUE}"))?;
            }
            res
        }
        None if op.requires_operand() => {
            return Err(format!("{mnemonic} requires an operand"));
        }
        None => 0,
    };

    if operand > OPERAND_MAX_VALUE as u16 {
        return Err(format!(
            "operand {operand} is too large to store in {OPERAND_WIDTH} bits"
        ));
    }

    if let Some(extra) = words.next() {
        return Err(format!("unexpected extra token {extra:?}"));
    }

    Ok(Instruction { op, operand })
}

fn encode_instruction(i: Instruction) -> u16 {
    ((i.op as u16) << 12) | i.operand
}

fn parse_sym_instruction(
    source: &str,
    ctx: &mut HashMap<String, usize>,
) -> Result<Option<Instruction>, String> {
    let words: Vec<_> = source.split_whitespace().collect();
    if let Some(x) = words.get(1) {
        if *x == "=" {
            let id = words[0];
            let addr = words[2].parse::<usize>().unwrap();
            ctx.insert(id.to_string(), addr);
            return Ok(None);
        }
    }
    parse_instruction(source, Some(ctx)).map(Some)
}

/// maintain a HashMap<str, usize> for id->addr
/// when parsing operands, if the operand parses to int then ok, otherwise
/// look up the string/id in the hashmap,
fn assemble(source: &str) -> Result<[u8; N_RAM_WORDS], String> {
    let mut ram: [u8; N_RAM_WORDS] = [0; N_RAM_WORDS];
    let mut addr = 0;

    let mut ctx: HashMap<String, usize> = HashMap::new();
    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        let code = line
            .split_once(';')
            .map_or(line, |(before, _)| before)
            .trim();

        if code.is_empty() {
            continue;
        }

        if addr + 2 > ram.len() {
            return Err(format!(
                "line {line_number}: program exceeds {N_RAM_WORDS} bytes"
            ));
        }

        let instr = parse_sym_instruction(code, &mut ctx)
            .map_err(|error| format!("line {line_number}: {error}"))?;
        println!("{:?}", instr);
        if let None = instr {
            continue;
        }
        let ie = encode_instruction(instr.unwrap());
        ram[addr..addr + 2].copy_from_slice(&ie.to_be_bytes());
        addr += 2;
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

/// Builds the control ROM indexed by opcode, microinstruction step, and flags.
fn create_rom() -> [u32; N_ROM_WORDS] {
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

fn main() -> Result<(), Box<dyn Error>> {
    let project = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| project.join("programs/simple.txt"));

    let ram_path = std::env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| project.join("rams_n_roms/RAM.txt"));


    let c = Control::DEFAULT
        .with_a_we(true)
        .with_bus_source(BusSource::Alu);

    println!("{:032b}", c.raw_value());

    let rom = create_rom();

    // Resolve paths relative to Cargo.toml, regardless of the working directory.

    save_memory(project.join("../ROM.txt"), &rom, 2, 4)?;
    save_memory(project.join("rams_n_roms/ROM.txt"), &rom, 2, 4)?;


    let source = fs::read_to_string(source_path)?;
    println!("{}", source);
    let ram = assemble(&source)?;

    save_memory(&ram_path, &ram, 2, 2)?;
    save_memory(&ram_path, &ram, 2, 2)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembles_in_order_while_ignoring_comments_and_blank_lines() {
        let source = "; initialize A\r\n\r\n  LDI 291 ; 0x123\r\nSTA 32\r\n\t\r\nHLT\r\n";
        let ram = assemble(source).unwrap();

        assert_eq!(&ram[..6], &[0x51, 0x23, 0x40, 0x20, 0xf0, 0x00]);
        assert!(ram[6..].iter().all(|&byte| byte == 0));
    }

    #[test]
    fn accepts_full_ram_and_rejects_one_more_instruction() {
        let mut source = "LDI 4095\n".repeat(N_RAM_WORDS / 2);
        let ram = assemble(&source).unwrap();
        assert!(ram.chunks_exact(2).all(|word| word == [0x5f, 0xff]));

        source.push_str("; comments do not consume memory\n\nHLT\n");
        assert_eq!(
            assemble(&source).unwrap_err(),
            "line 2051: program exceeds 4096 bytes"
        );
    }

    #[test]
    fn reports_the_original_source_line_for_invalid_instructions() {
        assert_eq!(
            assemble("; comment\n\nLDI\n").unwrap_err(),
            "line 3: LDI requires an operand"
        );
    }
}
