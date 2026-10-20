use std::{collections::HashMap, fmt::Display};

use crate::isa::{Instruction, N_RAM_WORDS, Op};

type IntT = u16;

#[derive(Debug, Clone)]
enum Expr {
    Sym(String),
    Int(IntT),
    List(Vec<Expr>),
}

peg::parser! {
    grammar parser() for str {
        pub rule ws() = [' ' | '\n' | '\r' | '\t']*
        pub rule ws1() = [' ' | '\n' | '\r' | '\t']+

        pub rule digit() -> char = ['0'..='9']
        pub rule int() -> u16 = n:$(digit()+) {? n.parse::<u16>().map_err(|_| "wtf")}

        pub rule char() -> char = ['a'..='z' | 'A'..='Z' | '=' | '+' | '*']
        pub rule sym() -> String = s:$(char()+) {s.to_owned()}

        pub rule atom() -> Expr = n:int() {Expr::Int(n)} / s:sym() {Expr::Sym(s)}
        pub rule list() -> Expr = "(" ws() args:expr()++ws1() ws() ")" {Expr::List(args)}
        pub rule expr() -> Expr = atom() / list()
        pub rule exprs() -> Vec<Expr> =  ws() exs:expr()++ws1() ws() {exs}
    }
}

#[derive(Debug)]
struct Ctx {
    addr_table: HashMap<String, usize>,
    next_addr: usize,
    /// for intermediate computations, we need to store to ram, this lets us generate identifiers to be used in the symbolic assembly
    next_intermediate: usize,
}
#[derive(Debug)]
enum Operand {
    Sym(String),
    Int(u16),
}
impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(n) => write!(f, "{n}"),
            Self::Sym(x) => f.write_str(x),
        }
    }
}
#[derive(Debug)]
struct SInstr {
    op: Op,
    operand: Operand,
}

#[derive(Debug)]
enum SasmInst {
    Asm(SInstr),
    MemAssign((String, usize)),
}
impl From<IntT> for Operand {
    fn from(value: IntT) -> Self {
        Self::Int(value)
    }
}

impl From<&str> for Operand {
    fn from(name: &str) -> Self {
        Self::Sym(name.to_owned())
    }
}
fn asm(op: Op, operand: impl Into<Operand>) -> SasmInst {
    SasmInst::Asm(SInstr {
        op,
        operand: operand.into(),
    })
}
/// if ur calling this, you've already checked Ctx doesnt have var
fn define(var: &str, ctx: &mut Ctx) -> SasmInst {
    let inst = SasmInst::MemAssign((var.to_string(), ctx.next_addr));
    ctx.addr_table.insert(var.to_string(), ctx.next_addr);
    ctx.next_addr -= 2;
    inst
}
fn define_intermediate(ctx: &mut Ctx) -> (String, SasmInst) {
    let var = format!("%{}", ctx.next_intermediate);
    let inst = SasmInst::MemAssign((var.to_string(), ctx.next_addr));
    ctx.addr_table.insert(var.to_string(), ctx.next_addr);
    ctx.next_addr -= 2;
    ctx.next_intermediate += 1;
    (var, inst)
}

fn render_sasm(sasm: &SasmInst) -> String {
    match sasm {
        SasmInst::Asm(SInstr { op, operand }) => {
            format!("{op:?} {operand}")
        }
        SasmInst::MemAssign((id, addr)) => format!("{id} = {addr}"),
    }
}

fn render_sasms(insts: &Vec<SasmInst>) -> String {
    insts
        .iter()
        .map(|inst| render_sasm(inst))
        .collect::<Vec<_>>()
        .join("\n")
}

/// the main result of compile_expr is that the A register will hold the value resulting in `expr`'s compilation
fn compile_expr(expr: &Expr, ctx: &mut Ctx, insts: &mut Vec<SasmInst>) -> Result<(), String> {
    match expr {
        Expr::Int(n) => {
            insts.push(asm(Op::Ldi, *n));
        }
        Expr::Sym(x) => {
            // should a compile error be thrown if x does not exist in addr_table
            if !ctx.addr_table.contains_key(x) {
                let err_str = format!("undef var {x} in {expr:?}");
                return Err(err_str);
            }
            insts.push(asm(Op::Lda, x.as_str()));
        }
        Expr::List(exs) => {
            let n_exs = exs.len();
            if n_exs != 3 {
                return Err(format!(
                    "Lists are only length 3 for now (binary ops) {:?}",
                    expr
                ));
            }
            match &exs[0] {
                // (3) => 3 => ldi 3
                // (3) => err because 3 is not a function

                // (x y z) too hard for rn
                Expr::Int(_) => return Err(format!("List head cannot be Int in {:?}", exs[0])),
                Expr::Sym(x) => {
                    match x.as_str() {
                        "=" => {
                            // (= ex1 ex2)
                            let ex1 = &exs[1];
                            let Expr::Sym(ex1_sym) = ex1 else {
                                return Err(format!(
                                    "Assignments must only be to Expr::Sym, so no (= 1 2) and no (= (+ x y) z)"
                                ));
                            };

                            compile_expr(&exs[2], ctx, insts)?;
                            // ex2 is in the A register
                            if ctx.addr_table.contains_key(ex1_sym) {
                                insts.push(asm(Op::Sta, ex1_sym.as_str()))
                            } else {
                                insts.push(define(ex1_sym, ctx));
                                insts.push(asm(Op::Sta, ex1_sym.as_str()))
                            }
                        }
                        "+" | "*" => {
                            let op = Op::from_src_sym(x).ok_or_else(|| {
                                format!("{x} is not a recognized built in operation")
                            })?;
                            // (+ ex1 ex2)
                            compile_expr(&exs[1], ctx, insts)?;
                            let (var, inst) = define_intermediate(ctx);
                            insts.push(inst);
                            insts.push(asm(Op::Sta, var.as_str()));
                            compile_expr(&exs[2], ctx, insts)?;

                            insts.push(asm(op, var.as_str()));
                        }
                        _ => {
                            let x: u64 = 1;
                            let y: u64 = 2;
                            let z = x * y;
                            return Err(format!(
                                "List heads must be built in symbols + or =, not {x} in {:?}",
                                exs[0]
                            ));
                        }
                    }
                }
                Expr::List(_) => return Err("List head cannot be List".to_string()),
            }
        }
    }
    Ok(())
}

pub fn compile(source: &str) -> Result<String, String> {
    let exprs = parser::exprs(source).map_err(|e| e.to_string())?;
    let mut ctx = Ctx {
        addr_table: HashMap::new(),
        next_addr: N_RAM_WORDS - 2,
        next_intermediate: 0,
    };
    let mut insts = vec![];
    for expr in exprs {
        compile_expr(&expr, &mut ctx, &mut insts)?;
    }
    insts.push(asm(Op::Hlt, 0));
    Ok(render_sasms(&insts))
}
