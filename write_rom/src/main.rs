use std::io::{self, BufWriter, Write};
use std::{fs::File};

fn main() {
    let file = File::create("output.txt").unwrap();
    let mut writer = BufWriter::new(file);

    let mut mem: [u16; 256] = [0; 256];
    mem[0] = 0b0000;
    for i in 0..=255 {
        if i % 16 == 0 {
            // pc_out, mar_in
            mem[i] |= (0b101 << 2) as u16;
            mem[i] |= (0b1 << 8) as u16;
        } else if i % 16 == 1 {
            // ram out, inst_in, pc_we
            mem[i] = 0b0000011_100_00;
        }
    }

    mem[0b0001_0010] = 0b0001_000_110_00;
    mem[0b0001_0011] = 0b1000_000_100_00;


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
}
