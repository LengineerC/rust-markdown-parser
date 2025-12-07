use crate::ast::Inline;
use std::borrow::Cow;

pub struct InlineParser<'a> {
    input: &'a str,
    // 字节索引
    pos: usize,
}

impl<'a> InlineParser<'a> {
    pub fn new(input: &'a str) -> Self {
        InlineParser { input, pos: 0 }
    }

    pub fn parse(&mut self) -> Vec<Inline<'a>> {
        let mut inlines = Vec::new();
        let mut text_buffer = String::new();

        while !self.eof() {
            let current_char = match self.current_char() {
                Some(c) => c,
                None => break,
            };

            match current_char {
                // === 1. 转义字符 ===
                '\\' => {
                    if let Some(next_char) = self.peek_char() {
                        if Self::is_special_char(next_char) {
                            text_buffer.push(next_char);
                            self.pos += '\\'.len_utf8() + next_char.len_utf8();
                            continue;
                        }
                    }
                    text_buffer.push('\\');
                    self.pos += '\\'.len_utf8();
                }

                // === 2. HTML 标签 ===
                '<' => {
                    if let Some(html_node) = self.try_parse_html_tag() {
                        Self::flush_text(&mut inlines, &mut text_buffer);
                        inlines.push(html_node);
                    } else {
                        text_buffer.push('<');
                        self.pos += '<'.len_utf8();
                    }
                }

                // === 3. 图片 (Image) ===
                '!' if self.peek_char() == Some('[') => {
                    if let Some(img_node) = self.try_parse_image() {
                        Self::flush_text(&mut inlines, &mut text_buffer);
                        inlines.push(img_node);
                    } else {
                        text_buffer.push('!');
                        self.pos += '!'.len_utf8();
                    }
                }

                // === 4. 链接 (Link) ===
                '[' => {
                    if let Some(link_node) = self.try_parse_link() {
                        Self::flush_text(&mut inlines, &mut text_buffer);
                        inlines.push(link_node);
                    } else {
                        text_buffer.push('[');
                        self.pos += '['.len_utf8();
                    }
                }

                // === 5. 行内代码 (CodeSpan) ===
                '`' => {
                    if let Some(node) = self.try_parse_code_span() {
                        Self::flush_text(&mut inlines, &mut text_buffer);
                        inlines.push(node);
                    } else {
                        text_buffer.push('`');
                        self.pos += '`'.len_utf8();
                    }
                }

                // === 6. 强调/粗体 (Emphasis/Strong) ===
                '*' => {
                    if let Some(node) = self.try_parse_emphasis() {
                        Self::flush_text(&mut inlines, &mut text_buffer);
                        inlines.push(node);
                    } else {
                        text_buffer.push('*');
                        self.pos += '*'.len_utf8();
                    }
                }

                // === 7. 删除线 (Strikethrough) ===
                '~' => {
                    if let Some(node) = self.try_parse_strikethrough() {
                        Self::flush_text(&mut inlines, &mut text_buffer);
                        inlines.push(node);
                    } else {
                        text_buffer.push('~');
                        self.pos += '~'.len_utf8();
                    }
                }

                // === 8. 普通字符 ===
                _ => {
                    text_buffer.push(current_char);
                    self.pos += current_char.len_utf8();
                }
            }
        }

        Self::flush_text(&mut inlines, &mut text_buffer);

        inlines
    }

    fn is_special_char(c: char) -> bool {
        matches!(
            c,
            '\\' | '`'
                | '*'
                | '_'
                | '{'
                | '}'
                | '['
                | ']'
                | '('
                | ')'
                | '#'
                | '+'
                | '-'
                | '.'
                | '!'
                | '|'
                | '>'
                | '~'
        )
    }

    // 解析*和**
    fn try_parse_emphasis(&mut self) -> Option<Inline<'a>> {
        let start_pos = self.pos;

        let delimiter_count = self.count_delimiter('*');

        // 修改点 A: 允许最多 3 个 (即 ***)
        if delimiter_count > 3 {
            self.pos = start_pos;
            return None;
        }

        let bytes = self.input.as_bytes();

        // 2. 处理 * 闭合
        let content_start = start_pos + delimiter_count;
        let mut current_search_pos = content_start;

        while current_search_pos < self.input.len() {
            if bytes[current_search_pos] == b'*' {
                let mut close_count = 0;
                let mut temp_pos = current_search_pos;
                while temp_pos < self.input.len() && bytes[temp_pos] == b'*' {
                    close_count += 1;
                    temp_pos += 1;
                }

                if close_count == delimiter_count {
                    // 前后 * 个数相匹配
                    let content_str = &self.input[content_start..current_search_pos];

                    // 递归下降分析中间内容
                    let mut child_parser = InlineParser::new(&content_str);
                    let children = child_parser.parse();

                    self.pos = temp_pos;

                    match delimiter_count {
                        1 => return Some(Inline::Emphasis(children)),
                        2 => return Some(Inline::Strong(children)),
                        3 => {
                            let inner_emph = Inline::Emphasis(children);
                            return Some(Inline::Strong(vec![inner_emph]));
                        }
                        _ => return None,
                    }
                }
                current_search_pos = temp_pos;
            } else {
                current_search_pos += 1;
            }
        }

        self.pos = start_pos;
        None
    }

    fn count_delimiter(&self, char_to_match: char) -> usize {
        debug_assert!(
            char_to_match.is_ascii(),
            "count_delimiter only supports ASCII chars"
        );

        let mut count = 0;
        let bytes = self.input.as_bytes();
        let byte_to_match = char_to_match as u8;

        let start = self.pos;
        while start + count < bytes.len() && bytes[start + count] == byte_to_match {
            count += 1;
        }
        count
    }

    fn flush_text(inlines: &mut Vec<Inline<'a>>, buffer: &mut String) {
        if !buffer.is_empty() {
            inlines.push(Inline::Text(Cow::from(buffer.clone())));
            buffer.clear();
        }
    }

    fn current_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn peek_char(&self) -> Option<char> {
        let mut chars = self.input[self.pos..].chars();
        chars.next();
        chars.next()
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn try_parse_image(&mut self) -> Option<Inline<'a>> {
        let start_pos = self.pos;
        self.pos += 1; // 移动到[

        if let Some((label, url)) = self.parse_bracket_link() {
            return Some(Inline::Image {
                alt: Cow::from(label),
                url: Cow::from(url),
            });
        }

        // 失败回退
        self.pos = start_pos;
        None
    }

    fn try_parse_link(&mut self) -> Option<Inline<'a>> {
        let start_pos = self.pos;

        if let Some((label, url)) = self.parse_bracket_link() {
            let mut inner_parser = InlineParser::new(&label);
            let children = inner_parser.parse();

            return Some(Inline::Link {
                children,
                url: Cow::from(url),
            });
        }

        self.pos = start_pos;
        None
    }

    fn parse_bracket_link(&mut self) -> Option<(&'a str, &'a str)> {
        let bytes = self.input.as_bytes();

        // [开头
        if self.pos >= self.input.len() || bytes[self.pos] != b'[' {
            return None;
        }

        // 预扫描[]是否闭合
        let start_bracket_pos = self.pos;
        let mut end_bracket_pos = start_bracket_pos + 1;

        while end_bracket_pos < self.input.len() && bytes[end_bracket_pos] != b']' {
            end_bracket_pos += 1;
        }

        if end_bracket_pos >= self.input.len() {
            return None;
        }

        if end_bracket_pos + 1 >= self.input.len() || bytes[end_bracket_pos + 1] != b'(' {
            return None;
        }

        // 匹配)
        let start_paren_pos = end_bracket_pos + 1;
        let mut end_paren_pos = start_paren_pos + 1;

        while end_paren_pos < self.input.len() && bytes[end_paren_pos] != b')' {
            end_paren_pos += 1;
        }

        if end_paren_pos >= self.input.len() {
            return None;
        }

        // 提取内容
        let label = &self.input[start_bracket_pos + 1..end_bracket_pos];

        let url = &self.input[start_paren_pos + 1..end_paren_pos];

        self.pos = end_paren_pos + 1;
        Some((label, url))
    }

    fn try_parse_code_span(&mut self) -> Option<Inline<'a>> {
        let start_pos = self.pos;
        let c = self.current_char()?;
        let delimiter_count = self.count_delimiter(c);

        let content_start = start_pos + delimiter_count;
        let mut cur_search_pos = content_start;

        let bytes = self.input.as_bytes();
        while cur_search_pos < self.input.len() {
            if bytes[cur_search_pos] == b'`' {
                let mut close_count = 0;
                let mut temp_pos = cur_search_pos;

                while temp_pos < self.input.len() && bytes[temp_pos] == b'`' {
                    close_count += 1;
                    temp_pos += 1;
                }

                // 前后`数量匹配
                if close_count == delimiter_count {
                    let content = &self.input[content_start..cur_search_pos];

                    self.pos = temp_pos;
                    return Some(Inline::CodeSpan(Cow::Borrowed(content)));
                }

                cur_search_pos = temp_pos;
            } else {
                cur_search_pos += 1;
            }
        }

        self.pos = start_pos;
        None
    }

    fn try_parse_strikethrough(&mut self) -> Option<Inline<'a>> {
        let start_pos = self.pos;
        let delimiter_count = self.count_delimiter('~');

        if delimiter_count != 2 {
            self.pos = start_pos;
            return None;
        }

        let bytes = self.input.as_bytes();

        let content_start = start_pos + delimiter_count;
        let mut cur_search_pos = content_start;

        while cur_search_pos < self.input.len() {
            if bytes[cur_search_pos] == b'~' {
                let mut close_count = 0;
                let mut temp_pos = cur_search_pos;

                while temp_pos < self.input.len() && bytes[temp_pos] == b'~' {
                    close_count += 1;
                    temp_pos += 1;
                }

                if close_count == 2 {
                    let content = &self.input[content_start..cur_search_pos];
                    let mut child_parser = InlineParser::new(content);
                    let children = child_parser.parse();

                    self.pos = temp_pos;

                    return Some(Inline::Strikethrough(children));
                }

                cur_search_pos = temp_pos;
            } else {
                cur_search_pos += 1;
            }
        }

        self.pos = start_pos;
        None
    }

    fn try_parse_html_tag(&mut self) -> Option<Inline<'a>> {
        let start_pos = self.pos;
        let bytes = self.input.as_bytes();

        if bytes[self.pos] != b'<' {
            return None;
        }

        let mut end_pos = start_pos + 1;
        while end_pos < self.input.len() {
            if bytes[end_pos] == b'>' {
                break;
            }
            end_pos += 1;
        }

        if end_pos >= self.input.len() {
            return None; // 没有闭合的 >
        }

        let tag_content = &self.input[start_pos..=end_pos];

        // 4. 简单验证看起来像不像标签
        // 规则：< 后面必须紧跟字母、/ 或 ! (注释 <!--)
        if tag_content.len() < 3 {
            return None;
        }
        let second_char = tag_content.chars().nth(1).unwrap();
        if !second_char.is_alphanumeric()
            && second_char != '/'
            && second_char != '!'
            && second_char != '?'
        {
            return None;
        }

        // 5. 成功
        self.pos = end_pos + 1;
        Some(Inline::RawHtml(Cow::from(tag_content)))
    }
}
