use std::str;

pub type Id = uint;

fn id_to_str(mut num: Id) -> ~str {
    let mut s = str::with_capacity(5);
    loop {
        let (div, rem) = num.div_rem(&26);
        s.push_char((rem + 97) as char);
        if div == 0 {
            break;
        }
        num = div;
    }
    s
}

#[deriving(Eq)]
pub struct Program {
    id: Id,
    expr: ~Expr,
}

#[deriving(Rand,Eq,IterBytes)]
pub enum UnaOp {
    Not,
    Shl1,
    Shr1,
    Shr4,
    Shr16,
}

#[deriving(Rand,Eq,IterBytes)]
pub enum BinOp {
    And,
    Or,
    Xor,
    Plus,
}

#[deriving(Eq,IterBytes)]
pub enum Operator {
    Operator_Op1(UnaOp),
    Operator_Op2(BinOp),
    Operator_TFold,
    Operator_Fold,
    Operator_If0
}

impl FromStr for Operator {
    fn from_str(s: &str) -> Option<Operator> {
        match s {
            "not" => Some(Operator_Op1(Not)),
            "shl1" => Some(Operator_Op1(Shl1)),
            "shr1" => Some(Operator_Op1(Shr1)),
            "shr4" => Some(Operator_Op1(Shr4)),
            "shr16" => Some(Operator_Op1(Shr16)),
            "and" => Some(Operator_Op2(And)),
            "or" => Some(Operator_Op2(Or)),
            "xor" => Some(Operator_Op2(Xor)),
            "plus" => Some( Operator_Op2(Plus)),
            "if0" => Some(Operator_If0),
            "fold" => Some(Operator_Fold),
            "tfold" => Some(Operator_TFold),
            _ => None
        }
    }
}

#[deriving(Eq)]
pub enum Expr {
    Zero,
    One,
    Ident(Id),
    If0(~Expr, ~Expr, ~Expr),
    Op1(UnaOp, ~Expr),
    Op2(BinOp, ~Expr, ~Expr),
    Fold {
        foldee: ~Expr,
        init: ~Expr,
        next_id: Id,
        accum_id: Id,
        body: ~Expr
    }
}

impl Program {
    pub fn new(id: Id, expr: ~Expr) -> Program {
        Program {
            id: id,
            expr: expr,
        }
    }

    pub fn len(&self) -> u8 {
        1 + self.expr.len()
    }
}

impl Expr {
    pub fn len(&self) -> u8 {
        match *self {
            Zero => 1,
            One => 1,
            Ident(*) => 1,
            If0(ref test, ref then, ref other) => {
                1 + test.len() + then.len() + other.len()
            }
            Op1(_, ref expr) => 1 + expr.len(),
            Op2(_, ref left, ref right) => 1 + left.len() + right.len(),
            Fold {foldee: ref foldee, init: ref init, body: ref body, _ } => {
                2 + foldee.len() + init.len() + body.len()
            }
        }
    }
}

impl FromStr for Program {
    pub fn from_str(_s: &str) -> Option<Program> {
        None
    }
}

impl ToStr for Program {
    pub fn to_str(&self) -> ~str {
        fmt!("(lambda (%s) %s)", id_to_str(self.id), self.expr.to_str())
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
            Ident(ref id) => id.to_str(),
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
            Fold {
                foldee: ref foldee, init: ref init,
                accum_id, next_id,
                body: ref body
            } => {
                fmt!("(fold %s %s (lambda (%s %s) %s))",
                     foldee.to_str(), init.to_str(),
                     id_to_str(accum_id), id_to_str(next_id),
                     body.to_str())
            }
        }
    }
}
