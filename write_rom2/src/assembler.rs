//! Assembly parsing and program generation. This module does not write files
//! or generate the CPU's control ROM.

use std::collections::HashMap;

use crate::isa::{
    INSTRUCTION_BYTES, Instruction, N_RAM_WORDS, OPERAND_MAX_VALUE, OPERAND_WIDTH, Op,
    encode_instruction,
};

fn parse_instruction(source: &str, ctx: &HashMap<String, usize>) -> Result<Instruction, String> {
    let mut words = source.split_whitespace();
    let mnemonic = words.next().ok_or("expected an instruction")?;
    let op: Op = mnemonic.parse().map_err(|_| "unknown op")?;

    let operand = match words.next() {
        Some(text) => text.parse::<usize>().or_else(|_| {
            ctx.get(text).copied().ok_or_else(|| {
                format!("invalid operand {text:?}: expected a number or defined symbol")
            })
        })?,
        None if op.requires_operand() => {
            return Err(format!("{mnemonic} requires an operand"));
        }
        None => 0,
    };

    if operand > OPERAND_MAX_VALUE {
        return Err(format!(
            "operand {operand} is too large to store in {OPERAND_WIDTH} bits"
        ));
    }

    if let Some(extra) = words.next() {
        return Err(format!("unexpected extra token {extra:?}"));
    }

    Ok(Instruction {
        op,
        operand: operand as u16,
    })
}

fn parse_sym_instruction(
    source: &str,
    ctx: &mut HashMap<String, usize>,
) -> Result<Option<Instruction>, String> {
    let words: Vec<_> = source.split_whitespace().collect();
    if words.get(1) == Some(&"=") {
        if words.len() != 3 {
            return Err("expected a symbol definition in the form NAME = ADDRESS".into());
        }
        let addr = words[2].parse::<usize>().map_err(|_| {
            format!(
                "invalid address {:?}: expected 0..={OPERAND_MAX_VALUE}",
                words[2]
            )
        })?;
        if addr > OPERAND_MAX_VALUE {
            return Err(format!(
                "address {addr} is too large to store in {OPERAND_WIDTH} bits"
            ));
        }
        ctx.insert(words[0].to_string(), addr);
        return Ok(None);
    }
    parse_instruction(source, ctx).map(Some)
}

/// Compile assembly and named address definitions into a big-endian RAM image.
/// Symbols must be defined before use; definitions do not occupy program bytes.
pub fn assemble(source: &str) -> Result<[u8; N_RAM_WORDS], String> {
    let mut ram = [0; N_RAM_WORDS];
    let mut addr = 0;
    let mut ctx = HashMap::new();

    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        let code = line
            .split_once(';')
            .map_or(line, |(before, _)| before)
            .trim();
        if code.is_empty() {
            continue;
        }

        let instr = parse_sym_instruction(code, &mut ctx)
            .map_err(|error| format!("line {line_number}: {error}"))?;
        let Some(instr) = instr else {
            continue;
        };

        if addr + INSTRUCTION_BYTES > ram.len() {
            return Err(format!(
                "line {line_number}: program exceeds {N_RAM_WORDS} bytes"
            ));
        }
        let encoded = encode_instruction(instr);
        ram[addr..addr + INSTRUCTION_BYTES].copy_from_slice(&encoded.to_be_bytes());
        addr += INSTRUCTION_BYTES;
    }

    Ok(ram)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_symbols_without_losing_numeric_operands() {
        let source = "x = 4094\ny = 4092\nLDI 5\nSTA x\nLDI 3\nADD y\nHLT\n";
        let ram = assemble(source).unwrap();
        assert_eq!(
            &ram[..10],
            &[0x50, 0x05, 0x4f, 0xfe, 0x50, 0x03, 0x2f, 0xfc, 0xf0, 0x00]
        );
    }

    #[test]
    fn rejects_invalid_symbols_and_operands_instead_of_panicking_or_using_zero() {
        for source in [
            "x =",
            "x = nope",
            "x = 65536",
            "x = 5 extra",
            "LDA missing",
            "LDI 65536",
        ] {
            assert!(
                assemble(source).unwrap_err().starts_with("line 1:"),
                "{source}"
            );
        }
    }

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
