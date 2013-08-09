enum UnaOp {
    Not,
    Shl1,
    Shr1,
    Shr4,
    Shr16,
}

enum BinOp {
    And,
    Or,
    Xor,
    Plus,
}

enum Expr {
    Zero,
    One,
    Ident(~str),
    If0(~Expr, ~Expr, ~Expr),
    Op1(UnaOp, ~Expr),
    Op2(BinOp, ~Expr, ~Expr),
}

impl FromStr for ~Expr {
    pub fn from_str(s: &str) -> Option<~Expr> {
        None
    }
}