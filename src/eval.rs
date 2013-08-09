use program::*;

// immutable linked list
struct Scope<'self> {
    id: &'self str,
    val: u64,
    parent: Option<&'self Scope<'self>>
}

impl<'self> Scope<'self> {
    fn push(&'self self, id: &'self str, val: u64) -> Scope<'self> {
        Scope {
            id: id,
            val: val,
            parent: Some(self)
        }
    }

    fn lookup(&self, id: &str) -> u64 {
        if id == self.id {
            self.val
        } else {
            match self.parent {
                Some(p) => p.lookup(id),
                None => fail!("ident %s not found", id)
            }
        }
    }

    fn eval(&self, expr: &Expr) -> u64 {
        match *expr {
            Zero => 0,
            One => 1,
            Ident(ref s) => self.lookup(*s),
            If0(~ref cond, ~ref then, ~ref els) => {
                if self.eval(cond) == 0 {
                    self.eval(then)
                } else {
                    self.eval(els)
                }
            }
            Op1(op, ~ref expr) => {
                let e = self.eval(expr);

                match op {
                    Not => !e,
                    Shl1 => e << 1,
                    Shr1 => e >> 1,
                    Shr4 => e >> 4,
                    Shr16 => e >> 16
                }
            }
            Op2(op, ~ref e1, ~ref e2) => {
                let e1 = self.eval(e1);
                let e2 = self.eval(e2);

                match op {
                    And => e1 & e2,
                    Or => e1 | e2,
                    Xor => e1 ^ e2,
                    Plus => e1 + e2
                }
            }
        }
    }
}

pub trait Eval {
    fn eval(&self, val: u64) -> u64;
}
impl Eval for Program {
    fn eval(&self, val: u64) -> u64 {
        (Scope {
                id: self.id,
                val: val,
                parent: None
            }).eval(self.expr)
    }
}
