//! Build the higher-level language compiler here, one small step at a time.

use crate::isa::N_RAM_WORDS;

#[derive(Debug)]
enum Expr {
    Sym(String),
    Int(u16),
    List(Vec<Expr>)
}

peg::parser! {
    grammar parser() for str {
        pub rule ws() = [' ' | '\t' | '\n' | '\r']*;
        pub rule ws1() = [' ' | '\t' | '\n' | '\r']+;

        pub rule digit() = ['0'..='9'];
        pub rule int() -> u16 = x:$(digit()+) {?
            x.parse::<u16>().map_err(|_| "int must be in u16")
        };

        pub rule char() = ['a'..='z' | 'A'..='Z' | '=' | '+'];
        pub rule sym() -> String = text:$(char()+) {text.to_owned()};

        pub rule atom() -> Expr = n:int() {Expr::Int(n)} / s:sym() {Expr::Sym(s)}
        pub rule expr() -> Expr = a:atom() {a} / "(" ws() xs:expr() ++ ws1() ws() ")" {Expr::List(xs)}
        pub rule exprs() -> Vec<Expr> = ws() es:expr()++ ws1() {es}
    }
}

// Start by adding a small peg::parser! grammar above this CLI placeholder.
pub fn compile(_source: &str) -> Result<[u8; N_RAM_WORDS], String> {
    // Err("compiler not implemented yet; use `assemble` for assembly programs".into())
    let x = parser::digit("5");
    println!("{:?}", x);
    let y = parser::int(&(1_usize << 17).to_string());
    println!("{:?}", y);
    let y = parser::int(&(1_usize << 15).to_string());
    println!("{:?}", y);
    
    println!("{:?}", parser::expr("(a)"));
    println!("{:?}", parser::expr("(a b)"));
    println!("{:?}", parser::expr("( a b )"));
    println!("{:?}", parser::expr("(= a 5)"));
    println!("{:?}", parser::exprs(_source));

    let z = parser::sym("hello");
    println!("{:?}", z);

    let cur_addr = 4094;
    //x=5
    // if ctx has x, do nothing, otherwise add x->4094 to ctx and decr cur_addr by 2 
    // e=(a+b)-(c+d)
    Ok([0_u8; N_RAM_WORDS])
}
