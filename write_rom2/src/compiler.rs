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

        pub rule char() -> char = ['a'..='z' | 'A'..='Z' | '=' | '+']
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
/// the compiler will read simple s_exprs like in 3.aj and convert that into symbolic assembly
/// then the assembler will convert the symbolic assembly to assembly
pub fn compile(source: &str) -> Result<String, String> {
    let mut ctx = Ctx {
        addr_table: HashMap::new(),
        next_addr: 4094,
        next_intermediate: 0,
    };

    let exprs = parser::exprs(source).map_err(|_| "failed to parse")?;
    let mut sym_asm_instructions = vec![];
    for expr in exprs {
        match expr {
            Expr::Int(n) => {
                // if i see a plain int in the program, i can naively add this immediate to the addr_table[%(next_intermediate)] @ next_addr,
                // increment next_intermediate, decr next_addr by word width
                // emit
                // LDI $n
                // STA %next_intermediate
                // ctx.addr_table
                //     .insert(format!("%{}", ctx.next_intermediate), ctx.next_addr);
                let var = format!("%{}", ctx.next_intermediate);
                let mut insts = vec![
                    SasmInst::MemAssign((var.clone(), ctx.next_addr)),
                    SasmInst::Asm(SInstr {
                        op: Op::Ldi,
                        operand: Operand::Int(n),
                    }),
                    SasmInst::Asm(SInstr {
                        op: Op::Sta,
                        operand: Operand::Sym(var.clone()),
                    }),
                ];
                ctx.next_addr -= 2;
                ctx.next_intermediate += 1;
                sym_asm_instructions.append(&mut insts);
            }
            Expr::Sym(x) => {
                let inst = SasmInst::Asm(SInstr {
                    op: Op::Lda,
                    operand: Operand::Sym(x),
                });
                sym_asm_instructions.push(inst);
            }
            Expr::List(args) => {
                // im just assuming all lists are of the form (= x 1)
                let f = &args[0];

                let mut insts = vec![];
                match f {
                    Expr::Sym(fsym) if fsym == "=" => {
                        let Expr::Sym(to_assign) = &args[1] else {
                            panic!("assignment var must be a symbol, not an int");
                        };
                        let val = &args[2];
                        if !ctx.addr_table.contains_key(to_assign) {
                            ctx.addr_table.insert(to_assign.to_string(), ctx.next_addr);
                            insts.push(SasmInst::MemAssign((to_assign.clone(), ctx.next_addr)));
                            ctx.next_addr -= 2;
                        }
                        let ld = match val {
                            Expr::Int(n) => SasmInst::Asm(SInstr {
                                op: Op::Ldi,
                                operand: Operand::Int(*n),
                            }),
                            Expr::Sym(x) => SasmInst::Asm(SInstr {
                                op: Op::Lda,
                                operand: Operand::Sym(x.to_string()),
                            }),
                            _ => panic!("no nesting yet"),
                        };
                        let st = SasmInst::Asm(SInstr {
                            op: Op::Sta,
                            operand: Operand::Sym(to_assign.to_string()),
                        });
                        insts.push(ld);
                        insts.push(st);
                        sym_asm_instructions.append(&mut insts)
                    }
                    Expr::Sym(fsym) if fsym == "+" => {
                        let arg1 = &args[1];
                        let arg2 = &args[2];
                        match (arg1, arg2) {
                            (Expr::Int(n1), Expr::Int(n2)) => {
                                let (var_n1, def1) = define_intermediate(&mut ctx);
                                let (var_n2, def2) = define_intermediate(&mut ctx);
                                insts.extend([
                                    def1,
                                    asm(Op::Ldi, *n1),
                                    asm(Op::Sta, var_n1.as_str()),
                                    def2,
                                    asm(Op::Ldi, *n2),
                                    asm(Op::Sta, var_n2.as_str()),
                                    asm(Op::Lda, var_n1.as_str()),
                                    asm(Op::Add, var_n2.as_str()),
                                ]);
                                /*
                                memassign a new intermediate to n1, then do a ldi n1 sta %x
                                then same for n2 to %y
                                then lda %x, add %y
                                 */
                                sym_asm_instructions.extend(insts);
                            }
                            (Expr::Int(n1), Expr::Sym(x1)) => {
                                /*
                                memassign for n1 to %n1 then ldi n1 sta %n1, then here we
                                assume that x1 already has an assignment so we just emit lda %n1 add x1
                                 */
                                let (var_n1, def1) = define_intermediate(&mut ctx);
                                insts.extend([
                                    def1,
                                    asm(Op::Ldi, *n1),
                                    asm(Op::Sta, var_n1.as_str()),
                                    asm(Op::Lda, var_n1.as_str()),
                                    asm(Op::Add, x1.as_str()),
                                ]);
                                sym_asm_instructions.extend(insts);

                            }
                            (Expr::Sym(x1), Expr::Int(n1)) => {
                                /*same as (int, sym) */

                                let (var_n1, def1) = define_intermediate(&mut ctx);
                                insts.extend([
                                    def1,
                                    asm(Op::Ldi, *n1),
                                    asm(Op::Sta, var_n1.as_str()),
                                    asm(Op::Lda, x1.as_str()),
                                    asm(Op::Add, var_n1.as_str()),
                                ]);
                                sym_asm_instructions.extend(insts);

                            }
                            (Expr::Sym(x1), Expr::Sym(x2)) => {

                                /*
                                lda x1
                                add x2
                                 */
                                insts.extend([
                                    asm(Op::Lda, x1.as_str()),
                                    asm(Op::Add, x2.as_str())
                                ]);
                                sym_asm_instructions.extend(insts);

                            }
                            _ => panic!("nested expressions not supported")
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    let hlt = SasmInst::Asm(SInstr {
        op: Op::Hlt,
        operand: Operand::Int(0),
    });
    sym_asm_instructions.push(hlt);
    Ok(sym_asm_instructions
        .iter()
        .map(|asm| render_sasm(asm))
        .collect::<Vec<_>>()
        .join("\n"))
    // Ok(sym_asm_instructions)
}

fn render_sasm(sasm: &SasmInst) -> String {
    match sasm {
        SasmInst::Asm(SInstr { op, operand }) => {
            format!("{op:?} {operand}")
        }
        SasmInst::MemAssign((id, addr)) => format!("{id} = {addr}"),
    }
}
