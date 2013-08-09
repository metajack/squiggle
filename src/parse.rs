use std::hashmap::HashMap;
use program::*;

pub trait Parse {
    pub fn parse(&self) -> Program;
}

impl<'self> Parse for &'self str {
    pub fn parse(&self) -> Program {
        Parser::new(*self).parse()
    }
}

pub struct Parser<'self> {
    src: &'self str,
    interned: HashMap<~str, uint>,
    next_id: uint
}

impl<'self> Parser<'self> {
    pub fn new<'r>(src: &'r str) -> Parser<'r> {
        Parser {
            src: src,
            interned: HashMap::new(),
            next_id: 0
        }
    }


    pub fn parse(&mut self) -> Program {
        self.skip_ws();
        self.skip_str("(");
        self.skip_str("lambda");
        self.skip_str("(");
        let id = self.consume_id();
        self.skip_str(")");

        let expr = self.consume_expr();

        Program {
            id: id,
            expr: expr
        }
    }

    pub fn bump(&mut self) {
        self.src = self.src.slice_from(
            self.src.char_range_at(0).next
        );
    }

    pub fn skip_ws(&mut self) {
        let mut offset = 0;
        for (new_offset, c) in self.src.char_offset_iter() {
            if c.is_whitespace() {
                offset = new_offset + c.len_utf8_bytes();
            } else {
                break;
            }
        }
        self.src = self.src.slice_from(offset);
    }

    pub fn skip_str(&mut self, expected: &str) {
        if self.src.starts_with(expected) {
            self.src = self.src.slice_from(expected.len());
            self.skip_ws();
        } else {
            fail!("expected: %s, found: %s", expected, self.src)
        }
    }

    pub fn consume_expr(&mut self) -> ~Expr {
        let ret = match self.src.char_at(0) {
            '0' => { self.bump(); ~Zero }
            '1' => { self.bump(); ~One }
            '(' => {
                self.bump();

                let s = self.consume_ident_str();
                let r = match s.as_slice() {
                    "not" => self.consume_op1(Not),
                    "shl1" => self.consume_op1(Shl1),
                    "shr1" => self.consume_op1(Shr1),
                    "shr4" => self.consume_op1(Shr4),
                    "shr16" => self.consume_op1(Shr16),
                    "and" => self.consume_op2(And),
                    "or" => self.consume_op2(Or),
                    "xor" => self.consume_op2(Xor),
                    "plus" =>  self.consume_op2(Plus),
                    "if0" => {
                        let cond = self.consume_expr();
                        let then = self.consume_expr();
                        let els = self.consume_expr();
                        ~If0(cond, then, els)
                    }
                    "fold" => {
                        let foldee = self.consume_expr();
                        let init = self.consume_expr();

                        self.skip_str("(");
                        self.skip_str("lambda");
                        self.skip_str("(");

                        let next = self.consume_id();
                        let accum = self.consume_id();
                        self.skip_str(")");

                        let body = self.consume_expr();
                        ~Fold {
                            foldee: foldee,
                            init: init,
                            next_id: next,
                            accum_id: accum,
                            body: body
                        }
                    }
                     _ => fail!("unrecognised op %s", s)
                };
                self.skip_str(")");
                r
            }
            _ => {
                ~Ident(self.consume_id())
            }
        };
        self.skip_ws();
        ret
    }

    pub fn consume_op1(&mut self, op: UnaOp) -> ~Expr {
        ~Op1(op, self.consume_expr())
    }

    pub fn consume_op2(&mut self, op: BinOp) -> ~Expr {
        let e1 = self.consume_expr();
        let e2 = self.consume_expr();
        ~Op2(op, e1, e2)
    }

    pub fn consume_id(&mut self) -> Id {
        let s = self.consume_ident_str();
        let id = do self.interned.find_or_insert_with(s) |_| {
            let num = self.next_id;
            self.next_id += 1;
            num
        };
        *id
    }

    pub fn consume_ident_str(&mut self) -> ~str {
        let mut offset = 0;
        for (new_offset, c) in self.src.char_offset_iter() {
            if c.is_alphanumeric() {
                offset = new_offset + c.len_utf8_bytes();
            } else {
                break
            }
        }
        if offset == 0 {
            fail!("expected ident, found %s", self.src);
        }
        let ret = self.src.slice_to(offset).to_owned();
        self.src = self.src.slice_from(offset);
        self.skip_ws();
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use program::*;

    #[test]
    fn test_skip_ws() {
        let mut p = Parser::new(" \t\n hi");
        p.skip_ws();
        assert_eq!(p.src, "hi");
    }

    #[test]
    fn test_skip_ws_not() {
        let mut p = Parser::new("yarrgh ");
        p.skip_ws();
        assert_eq!(p.src, "yarrgh ");
    }

    #[test]
    fn test_skip_str() {
        let mut p = Parser::new("hurro");
        p.skip_str("hurr");
        assert_eq!(p.src, "o");
    }

    #[test]
    fn test_parse() {
       let mut p = Parser::new("(lambda (x) (or x (shl1 (if0 x 0 1))))");
       assert_eq!(p.parse(), Program::new(0, ~Op2(Or, ~Ident(0),
                                                  ~Op1(Shl1,
                                                       ~If0(~Ident(0),~Zero,~One)))));
    }
    #[test]
    fn test_parse_fold() {
       let mut p = Parser::new("(lambda (x) (fold x 0 (lambda (a b) (plus a b))))");
       assert_eq!(p.parse(), Program::new(0, ~Fold {
                        foldee: ~Ident(0),
                        init: ~Zero,
                        next_id: 1,
                        accum_id: 2,
                        body: ~Op2(Plus, ~Ident(1), ~Ident(2))
                    }));
    }
}
