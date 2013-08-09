use program::Expr;

pub type ParseResult = Result<~Expr, ~str>;

pub trait Parse {
    pub fn parse(&self) -> ParseResult;
}

impl<'self> Parse for &'self str {
    pub fn parse(&self) -> ParseResult {
        Parser::new(*self).parse()
    }
}

pub struct Parser<'self> {
    pos: uint,
    src: &'self str,
}

impl<'self> Parser<'self> {
    pub fn new<'r>(src: &'r str) -> Parser<'r> {
        Parser {
            pos: 0,
            src: src,
        }
    }

    pub fn parse(&mut self) -> ParseResult {
        self.consume_lambda()
    }

    pub fn skip_ws(&mut self) {
        for c in self.src.iter() {
            if c.is_whitespace() { self.pos += 1 } else { break };
        }
    }

    pub fn consume_ws(&mut self) -> ParseResult {
        Err(~"Not yet implemented.")
    }

    pub fn consume_str(&mut self, _expected: &str) -> ParseResult {
        Err(~"Not yet implemented.")
    }

    pub fn consume_lambda(&mut self) -> ParseResult {
        Err(~"Not yet implemented.")
    }

    pub fn comsume_expression(&mut self) -> ParseResult {
        Err(~"Not yet implemented.")
    }

    pub fn consume_op1(&mut self) -> ParseResult {
        Err(~"Not yet implemented.")
    }

    pub fn consume_op2(&mut self) -> ParseResult {
        Err(~"Not yet implemented.")
    }

    pub fn consume_id(&mut self) -> ParseResult {
        Err(~"Not yet implemented.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_ws() {
        let mut p = Parser::new(" \t\n");
        p.skip_ws();
        assert_eq!(p.pos, 3);
    }
}
