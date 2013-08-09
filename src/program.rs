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

impl Program {
    pub fn new(id: ~str, expr: ~Expr) -> Program {
        Program {
            id: id,
            expr: expr,
        }
    }
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
        program.push_str(")");
        program
    }
}

impl ToStr for UnaOp {
    pub fn to_str(&self) -> ~str {
        match *self {
            Not => ~"not",
            Shl1 => ~"shl1",
            Shr1 => ~"shr1",
            Shr4 => ~"shr4",
            Shr16 => ~"shr16",
        }
    }
}

impl ToStr for BinOp {
    pub fn to_str(&self) -> ~str {
        match *self {
            And => ~"and",
            Or => ~"or",
            Xor => ~"xor",
            Plus => ~"plus",
        }
    }
}

impl ToStr for Expr {
    pub fn to_str(&self) -> ~str {
        match *self {
            Zero => ~"0",
            One => ~"1",
            Ident(ref id) => id.clone(),
            If0(ref test, ref then, ref other) => {
                let mut e = ~"(if0 ";
                e.push_str(test.to_str());
                e.push_str(" ");
                e.push_str(then.to_str());
                e.push_str(" ");
                e.push_str(other.to_str());
                e
            }
            Op1(op, ref expr) => {
                let mut e = ~"(";
                e.push_str(op.to_str());
                e.push_str(" ");
                e.push_str(expr.to_str());
                e.push_str(")");
                e
            }
            Op2(op, ref left, ref right) => {
                let mut e = ~"(";
                e.push_str(op.to_str());
                e.push_str(" ");
                e.push_str(left.to_str());
                e.push_str(" ");
                e.push_str(right.to_str());
                e.push_str(")");
                e
            }
        }
    }
}

impl Rand for Expr {
    pub fn rand<R: Rng>(rng: &mut R) -> Expr {
        Zero
    }
}