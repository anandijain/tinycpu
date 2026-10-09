use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Lines, Write};

fn main() -> io::Result<()> {
    let file = File::create("ROM.txt").unwrap();
    let mut writer = BufWriter::new(file);

    let mut mem: [u16; 256] = [0; 256];

    let a_in: u16 = 0b1000_0000_0000;
    let b_in: u16 = 0b0100_0000_0000;
    let c_in: u16 = 0b0010_0000_0000;
    let mar_in: u16 = 0b0001_0000_0000;
    let ram_in: u16 = 0b0000_1000_0000;
    let pc_inc: u16 = 0b0000_0100_0000;
    let inst_in: u16 = 0b0000_0010_0000;

    let a_out: u16 = 0 << 2;
    let b_out: u16 = 1 << 2;
    let c_out: u16 = 2 << 2;
    let alu_out: u16 = 3 << 2;
    let ram_out: u16 = 4 << 2;
    let pc_out: u16 = 5 << 2;
    let inst_out: u16 = 6 << 2;

    let inst_step_reset: u16 = 0b0000_0000_0010;
    let hlt: u16 = 1;

    let hlt_opcode: u16 = 0b1111;

    mem[0] = 0b0000;
    for i in 0..=255 {
        if i % 16 == 0 {
            // pc_out, mar_in
            mem[i] |= (0b101 << 2) as u16;
            mem[i] |= (0b1 << 8) as u16;
        } else if i % 16 == 1 {
            // ram out, inst_in, pc_in
            mem[i] = 0b0000011_100_00;
        }
    }

    // LDA
    mem[0b0001_0010] = inst_out | mar_in;
    mem[0b0001_0011] = ram_out | a_in;
    mem[0b0001_0100] = inst_step_reset;

    // ADD
    mem[0b0010_0010] = inst_out | mar_in;
    mem[0b0010_0011] = ram_out | b_in;
    mem[0b0010_0100] = alu_out | a_in;
    mem[0b0010_0101] = inst_step_reset;

    // STA
    // inst_reg=0100_1110 STA 14, means write the contents of A register to address 14 in ram
    mem[0b0100_0010] = inst_out | mar_in;
    mem[0b0100_0011] = a_out | ram_in;
    mem[0b0100_0100] = inst_step_reset;

    // LDI - "load immediate"
    // LDI 10 - load not the contents of ram at addr 10, but the actual value 10 into register A
    // opcode 0101
    // instr_reg = 0101_1010
    mem[0b0101_0010] = inst_out | a_in;
    mem[0b0101_0011] = inst_step_reset;
    
    // JMP ADDR - "set the pc to addr"
    // opcode 0110=6
    // inst_reg = 0110_0001
    mem[0b0110_0010] = inst_out | pc_;
    mem[0b0110_0011] = inst_step_reset;
    

    // HLT
    mem[0b1111_0010] = hlt;

    writeln!(writer, "v3.0 hex words addressed").unwrap();
    for (i, x) in mem.iter().enumerate() {
        if i % 16 == 0 {
            if i != 0 {
                writeln!(writer).unwrap();
            }
            write!(writer, "{i:02x}: ").unwrap();
        }
        write!(writer, "{x:03x} ").unwrap();
    }
    let file = File::open("README.md")?;
    let reader = BufReader::new(file);

    let ram_file = File::create("RAM.txt")?;
    let mut ram_writer = BufWriter::new(ram_file);

    writeln!(ram_writer, "v3.0 hex words addressed")?;
    write!(ram_writer, "00: ")?;
    let mut bytes: [u8; 16] = [0; 16];
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let ws: Vec<_> = line.split(' ').collect();
        let opcode: u8 = match ws[0] {
            "LDA" => 1,
            "ADD" => 2,
            // "SUB"=>3,
            "STA" => 4,
            "LDI" => 5,
            "HLT" => 15,
            _ => panic!(),
        };
        let data: u8 = (opcode << 4) | ws[1].parse::<u8>().unwrap();
        bytes[i] = data;
    }
    for b in bytes {
        write!(ram_writer, "{b:02x} ")?;
    }
    Ok(())
}
