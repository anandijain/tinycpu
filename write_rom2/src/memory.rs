//! Logisim memory-image output, shared by the CLI commands.

use std::{
    fmt::LowerHex,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
};

/// Formats RAM bytes or control-ROM words for Logisim.
/// Widths are minimum hexadecimal digit counts.
pub fn write_memory<T: LowerHex>(
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

pub fn save_memory<T: LowerHex>(
    path: impl AsRef<Path>,
    mem: &[T],
    address_digits: usize,
    data_digits: usize,
) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    write_memory(mem, &mut writer, address_digits, data_digits)?;
    writer.flush()
}
