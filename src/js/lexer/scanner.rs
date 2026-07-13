use crate::span::Pos;

#[derive(Debug, Clone)]
pub struct Scanner<'a> {
    source: &'a str,
    offset: usize,
    line: usize,
    column: usize,
    pub has_line_break: bool,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            offset: 0,
            line: 1,
            column: 0,
            has_line_break: false,
        }
    }

    pub fn peek(&self) -> Option<char> {
        let rest = &self.source[self.offset..];
        rest.chars().next()
    }

    pub fn peek_ahead(&self, n: usize) -> Option<char> {
        let rest = &self.source[self.offset..];
        let mut chars = rest.chars();
        for _ in 0..n {
            chars.next()?;
        }
        chars.next()
    }

    pub fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.offset += ch.len_utf8();
        match ch {
            '\r' => {
                self.line += 1;
                self.column = 0;
                self.has_line_break = true;
                if self.peek() == Some('\n') {
                    self.offset += '\n'.len_utf8();
                }
            }
            '\n' => {
                self.line += 1;
                self.column = 0;
                self.has_line_break = true;
            }
            _ => {
                self.column += 1;
            }
        }
        Some(ch)
    }

    pub fn advance_if(&mut self, pred: impl FnOnce(char) -> bool) -> Option<char> {
        match self.peek() {
            Some(ch) if pred(ch) => self.advance(),
            _ => None,
        }
    }

    pub fn advance_if_eq(&mut self, ch: char) -> bool {
        self.advance_if(|c| c == ch).is_some()
    }

    pub fn position(&self) -> Pos {
        Pos::new(self.offset, self.line, self.column)
    }

    pub fn slice(&self, start: Pos) -> &'a str {
        &self.source[start.offset..self.offset]
    }

    pub fn slice_from(&self, start: usize) -> &'a str {
        &self.source[start..self.offset]
    }

    pub fn is_eof(&self) -> bool {
        self.offset >= self.source.len()
    }

    pub fn remaining(&self) -> &'a str {
        &self.source[self.offset..]
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn source(&self) -> &'a str {
        self.source
    }

    pub fn reset_line_break(&mut self) {
        self.has_line_break = false;
    }
}
