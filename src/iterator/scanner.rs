#[derive(Debug, Clone)]
pub struct Scanner<'a> {
    input: &'a str,
    bytes: &'a [u8],
    /// 字节偏移量
    pos: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut pos = 0;

        // BOM
        if input.as_bytes().starts_with(b"\xEF\xBB\xBF") {
            pos = 3;
        }

        Self {
            input,
            bytes: input.as_bytes(),
            pos,
        }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn set_position(&mut self, pos: usize) {
        if pos > self.bytes.len() {
            self.pos = self.bytes.len();
        } else {
            self.pos = pos;
        }
    }

    pub fn remaining_str(&self) -> &'a str {
        &self.input[self.pos..]
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    pub fn peek(&self) -> Option<char> {
        if self.is_eof() {
            return None;
        }

        let byte = self.bytes[self.pos];

        if byte == b'\r' {
            return Some('\n');
        }

        if byte < 128 {
            Some(byte as char)
        } else {
            self.remaining_str().chars().next()
        }
    }

    pub fn next_char(&mut self) -> Option<char> {
        if self.is_eof() {
            return None;
        }

        let byte = self.bytes[self.pos];

        if byte == b'\r' {
            if self.pos + 1 < self.bytes.len() && self.bytes[self.pos + 1] == b'\n' {
                self.pos += 2;
            } else {
                self.pos += 1;
            }
            return Some('\n');
        }

        if byte < 128 {
            self.pos += 1;
            return Some(byte as char);
        }

        let c = self.remaining_str().chars().next()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    pub fn match_char(&mut self, expected: char) -> bool {
        if Some(expected) == self.peek() {
            self.next_char();
            true
        } else {
            false
        }
    }

    /// 匹配前缀
    pub fn match_str(&mut self, expected: &str) -> bool {
        let current = self.remaining_str();
        if current.starts_with(expected) {
            self.pos += expected.len();
            true
        } else {
            false
        }
    }

    pub fn skip_whitespace(&mut self) -> usize {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' {
                self.pos += 1;
            } else {
                break;
            }
        }
        self.pos - start
    }

    pub fn find(&self, needle: u8) -> Option<usize> {
        self.bytes[self.pos..].iter().position(|&b| b == needle)
    }

    pub fn slice_from(&self, start: usize) -> &'a str {
        &self.input[start..self.pos]
    }

    pub fn slice(&self, start: usize, end: usize) -> &'a str {
        &self.input[start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_basics() {
        let mut s = Scanner::new("Hello # 世界");

        assert_eq!(s.next_char(), Some('H'));
        assert!(s.match_str("ello"));
        assert_eq!(s.skip_whitespace(), 1);
        assert!(s.match_char('#'));
        assert_eq!(s.skip_whitespace(), 1);

        assert_eq!(s.peek(), Some('世'));
        assert_eq!(s.next_char(), Some('世'));

        assert_eq!(s.remaining_str(), "界");
    }

    #[test]
    fn test_match_char_crlf() {
        let mut s = Scanner::new("A\r\nB");

        assert!(s.match_char('A'));

        assert!(s.match_char('\n'));

        assert_eq!(s.peek(), Some('B'));
        assert!(s.match_char('B'));
    }

    #[test]
    fn test_match_char_cr() {
        let mut s = Scanner::new("A\rB");
        assert!(s.match_char('A'));
        assert!(s.match_char('\n'));
        assert_eq!(s.peek(), Some('B'));
    }
}
