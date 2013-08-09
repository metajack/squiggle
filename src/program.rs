use std::rand::{Rand, Rng};

pub struct Program {
    id: ~str,
    expr: ~Expr,
}

pub enum UnaOp {
    Not,
    Shl1,
    Shr1,
    Shr4,
    Shr16,
}

pub enum BinOp {
    And,
    Or,
    Xor,
    Plus,
}

pub enum Expr {
    Zero,
    One,
    Ident(~str),
    If0(~Expr, ~Expr, ~Expr),
    Op1(UnaOp, ~Expr),
    Op2(BinOp, ~Expr, ~Expr),
}

impl FromStr for Program {
    pub fn from_str(s: &str) -> Option<Program> {
        None
    }
}

impl ToStr for Program {
    pub fn to_str(&self) -> ~str {
        let mut program = ~"(lambda (";
        program.push_str(self.id);
        program.push_str(") ");
        program.push_str(self.expr.to_str());
        program
    }
}

impl ToStr for Expr {
    pub fn to_str(&self) -> ~str {
        // serializer
        ~""
    }
}

impl Rand for Expr {
    pub fn rand<R: Rng>(rng: &mut R) -> Expr {
        Zero
    }
}