//! Assembly parsing and program generation. This module does not write files
//! or generate the CPU's control ROM.

use std::collections::HashMap;

use crate::isa::{
    INSTRUCTION_BYTES, Instruction, N_RAM_WORDS, OPERAND_MAX_VALUE, OPERAND_WIDTH, Op,
    encode_instruction,
};

#[derive(Debug, Clone)]
pub enum SasmOperand {
    Sym(String),
    Int(usize),
}

#[derive(Debug)]
pub struct SasmInst {
    op: Op,
    sasm_operand: Option<SasmOperand>,
}

#[derive(Debug)]
pub enum SasmExpr {
    Asm(SasmInst),
    /// x = 100
    /// y = x not allowed
    Assign {
        name: String,
        addr: usize,
    },
    Label {
        name: String,
    },
}

peg::parser! {
    pub grammar asm_parser() for str {
        pub rule hws() = [' ' | '\t']* // need to add comments
        pub rule hws1() = [' ' | '\t']+
        pub rule newline() = "\r\n" / "\n" / "\r"

        pub rule gap() = ([' ' | '\t' | '\n' | '\r'] / comment())*
        pub rule comment() = ";" [^ '\n' | '\r']*
        pub rule line_end() = hws() comment()? (newline() / ![_])

        // pub rule comment() = anything after a semicolon within a line ';' .*
        pub rule digit() -> char = ['0'..='9']
        pub rule int() -> usize = n:$(digit()+) {? n.parse::<usize>().map_err(|_| "wtf")}

        pub rule identifier() -> String = s:$(['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '_' | '0'..='9']*) {s.to_string()}

        rule opcode() -> Op
            = name:identifier() {?
                name.parse::<Op>()
                    .map_err(|_| "valid instruction mnemonic")
    }
        pub rule operand() -> SasmOperand = n:int() {SasmOperand::Int(n)} / sym:identifier() {SasmOperand::Sym(sym)}
        // x = 5
        pub rule assignment() -> SasmExpr =
            hws() name:$(identifier()) hws() "=" hws() arg:int() line_end() {SasmExpr::Assign { name: name.to_string(), addr: arg }}

        pub rule label() -> SasmExpr = hws() name:$(identifier()) ":" hws() line_end() {? Ok(SasmExpr::Label { name: name.to_string() })}
        pub rule asm() -> SasmExpr = hws() op:(opcode()) arg:(hws1() value:operand() { value })? hws() line_end() {?

            if Op::requires_operand(op) {
                if arg.is_none() {
                    return Err("got an op that requires an arg, but none was provided")
                }
            }

            Ok(SasmExpr::Asm(SasmInst { op, sasm_operand: arg }))
        }

        pub rule expr() -> SasmExpr = asm() / assignment() / label()
        pub rule exprs() -> Vec<SasmExpr> = gap() es:(e:expr() gap() {e})* {es}
    }
}

/// Compile assembly and named address definitions into a big-endian RAM image.
/// Symbols must be defined before use; definitions do not occupy program bytes.
pub fn assemble(exprs: &Vec<SasmExpr>) -> Result<Vec<Instruction>, String> {
    let mut next_addr = 0; // points to where the next instruction in RAM will go 
    let mut ctx: HashMap<String, usize> = HashMap::new();

    // let exs = asm_parser::exprs(source).map_err(|e| e.to_string())?;
    let mut insts = vec![];
    //
    for (index, ex) in exprs.iter().enumerate() {
        match ex {
            SasmExpr::Asm(asm) => {
                next_addr += 2;
            }
            SasmExpr::Label { name } => {
                ctx.insert(name.to_string(), next_addr);
            }

            SasmExpr::Assign { name, addr } => {
                ctx.insert(name.to_string(), *addr);
            }
        }
    }
    // next_addr = 0;
    for (index, ex) in exprs.iter().enumerate() {
        match ex {
            SasmExpr::Asm(asm) => {
                let operand: u16 = match asm.sasm_operand.clone() {
                    Some(x) => match x {
                        SasmOperand::Int(n) => n as u16,
                        SasmOperand::Sym(s) => *ctx.get(&s).unwrap() as u16,
                    },
                    None => 0,
                };
                let inst = Instruction {
                    op: asm.op,
                    operand,
                };
                insts.push(inst);
                // next_addr += 2;
            }
            _ => {}
        }
    }

    // Ok(ram)
    Ok(insts)
}

pub fn assemble_ram(insts: &Vec<Instruction>) -> Result<[u8; N_RAM_WORDS], String> {
    let mut ram = [0; N_RAM_WORDS];
    let mut addr = 0;
    for (i, inst) in insts.iter().enumerate() {
        ram[addr..addr+2].copy_from_slice(&encode_instruction(inst).to_be_bytes());
        addr +=2
    }

    Ok(ram)
}
