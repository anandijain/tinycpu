//! Shared instruction set and encoding used by the compiler and control ROM.

use strum::{EnumIter, EnumString, FromRepr};

pub const OPERAND_WIDTH: usize = 12;
pub const OPERAND_MAX_VALUE: usize = (1 << OPERAND_WIDTH) - 1;
pub const RAM_ADDR_WIDTH: usize = OPERAND_WIDTH;
pub const N_RAM_WORDS: usize = 1 << RAM_ADDR_WIDTH;
pub const INSTRUCTION_BYTES: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, EnumString, FromRepr)]
#[strum(ascii_case_insensitive)]
#[repr(u8)]
pub enum Op {
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
    pub fn requires_operand(self) -> bool {
        !matches!(self, Self::Nop | Self::Hlt)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction {
    pub op: Op,
    pub operand: u16,
}

pub fn encode_instruction(i: Instruction) -> u16 {
    ((i.op as u16) << OPERAND_WIDTH) | i.operand
}
