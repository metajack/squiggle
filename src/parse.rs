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
    len: uint,
}

impl<'self> Parser<'self> {
    pub fn new<'r>(src: &'r str) -> Parser<'r> {
        Parser {
            pos: 0,
            src: src,
            len: src.len(),
        }
    }

    pub fn parse(&mut self) -> ParseResult {
        self.consume_lambda()
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.len
    }

    pub fn skip_ws(&mut self) -> bool {
        let mut skipped = false;
        for c in self.src.iter() {
            if c.is_whitespace() {
                self.pos += 1;
                skipped = true;
            } else {
                break;
            }
        }
        skipped
    }

    pub fn skip_str(&mut self, expected: &str) -> bool {
        if self.src.starts_with(expected) {
            self.pos += expected.len();
            true
        } else {
            false
        }
    }

    pub fn consume_ws(&mut self) -> ParseResult {
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
    fn test_is_eof() {
        let mut p = Parser::new(" ");
        assert_eq!(p.is_eof(), false);
        p.skip_ws();
        assert_eq!(p.is_eof(), true);
    }

    #[test]
    fn test_skip_ws() {
        let mut p = Parser::new(" \t\n hi");
        assert_eq!(p.skip_ws(), true);
        assert_eq!(p.pos, 4);
    }

    #[test]
    fn test_skip_ws_not() {
        let mut p = Parser::new("yarrgh ");
        assert_eq!(p.skip_ws(), false);
        assert_eq!(p.pos, 0);
    }

    #[test]
    fn test_skip_str() {
        let mut p = Parser::new("hurro");
        assert_eq!(p.skip_str("hooroo"), false);
        assert_eq!(p.pos, 0);
        assert_eq!(p.skip_str("hurr"), true);
        assert_eq!(p.pos, 4);
    }
}
