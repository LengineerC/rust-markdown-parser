use crate::ast::Inline;

pub struct InlineParser {
    input: Vec<char>,
    pos: usize,
}

impl InlineParser {
    pub fn new(input: &str) -> Self {
        InlineParser {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<Inline> {
        let mut inlines = Vec::new();
        let mut text_buffer = String::new();

        while self.pos < self.input.len() {
            let current_char = self.input[self.pos];

            // Image
            if current_char == '!' && self.peek_char() == Some('[') {
                self.flush_text(&mut inlines, &mut text_buffer);

                if let Some(img_node) = self.try_parse_image() {
                    inlines.push(img_node);
                } else {
                    text_buffer.push('!');
                    self.pos += 1;
                }
            }
            // Link
            else if current_char == '[' {
                self.flush_text(&mut inlines, &mut text_buffer);

                if let Some(link_node) = self.try_parse_link() {
                    inlines.push(link_node);
                } else {
                    text_buffer.push('[');
                    self.pos += 1;
                }
            }
            // CodeSpan
            else if current_char == '`' {
                self.flush_text(&mut inlines, &mut text_buffer);
                if let Some(node) = self.try_parse_code_span() {
                    inlines.push(node);
                } else {
                    text_buffer.push('`');
                    self.pos += 1;
                }
            }
            // Strong, Emphasis
            // 重新遇到*时刷新缓冲区保存节点
            else if current_char == '*' {
                if !text_buffer.is_empty() {
                    inlines.push(Inline::Text(text_buffer.clone()));
                    text_buffer.clear();
                }

                if let Some(node) = self.try_parse_emphasis() {
                    inlines.push(node);
                } else {
                    text_buffer.push('*');
                    self.pos += 1;
                }
            }
            // Strikethrough
            else if current_char == '~' {
                self.flush_text(&mut inlines, &mut text_buffer);

                if let Some(node) = self.try_parse_strikethrough() {
                    inlines.push(node);
                } else {
                    text_buffer.push('~');
                    self.pos += 1;
                }
            } else {
                text_buffer.push(current_char);
                self.pos += 1;
            }
        }

        if !text_buffer.is_empty() {
            inlines.push(Inline::Text(text_buffer));
        }

        inlines
    }

    // 解析*和**
    fn try_parse_emphasis(&mut self) -> Option<Inline> {
        let start_pos = self.pos;

        // 确定*字符个数
        let delimiter_count = self.count_delimiter('*');

        if delimiter_count > 2 {
            self.pos = start_pos;
            return None;

            todo!("只处理*为1和2，待处理*>2的情况");
        }

        // 处理*闭合
        let content_start = start_pos + delimiter_count;
        let mut current_search_pos = content_start;

        while current_search_pos < self.input.len() {
            if self.input[current_search_pos] == '*' {
                let mut close_count = 0;
                let mut temp_pos = current_search_pos;

                while temp_pos < self.input.len() && self.input[temp_pos] == '*' {
                    close_count += 1;
                    temp_pos += 1;
                }

                if close_count == delimiter_count {
                    // 前后*个数相匹配

                    let content_str: String = self.input[content_start..current_search_pos]
                        .iter()
                        .collect();

                    // 递归下降分析中间内容
                    let mut childer_parser = InlineParser::new(&content_str);
                    let children = childer_parser.parse();

                    self.pos = temp_pos;

                    if delimiter_count == 1 {
                        return Some(Inline::Emphasis(children));
                    } else {
                        return Some(Inline::Strong(children));
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

    fn count_delimiter(&mut self, char_to_match: char) -> usize {
        let mut count = 0;
        let temp_pos = self.pos;

        while temp_pos + count < self.input.len() && self.input[temp_pos + count] == char_to_match {
            count += 1;
        }

        count
    }

    fn flush_text(&self, inlines: &mut Vec<Inline>, buffer: &mut String) {
        if !buffer.is_empty() {
            inlines.push(Inline::Text(buffer.clone()));
            buffer.clear();
        }
    }

    fn peek_char(&self) -> Option<char> {
        if self.pos + 1 < self.input.len() {
            Some(self.input[self.pos + 1])
        } else {
            None
        }
    }

    fn try_parse_image(&mut self) -> Option<Inline> {
        let start_pos = self.pos;
        self.pos += 1; // 移动到[

        if let Some((label, url)) = self.parse_bracket_link() {
            return Some(Inline::Image { alt: label, url });
        }

        // 失败回退
        self.pos = start_pos;
        None
    }

    fn try_parse_link(&mut self) -> Option<Inline> {
        let start_pos = self.pos;

        if let Some((label, url)) = self.parse_bracket_link() {
            let mut inner_parser = InlineParser::new(&label);
            let children = inner_parser.parse();

            return Some(Inline::Link { children, url });
        }

        self.pos = start_pos;
        None
    }

    fn parse_bracket_link(&mut self) -> Option<(String, String)> {
        // [开头
        if self.pos >= self.input.len() || self.input[self.pos] != '[' {
            return None;
        }

        // 预扫描[]是否闭合
        let start_bracket_pos = self.pos;
        let mut end_bracket_pos = start_bracket_pos + 1;

        while end_bracket_pos < self.input.len() && self.input[end_bracket_pos] != ']' {
            end_bracket_pos += 1;
        }

        if end_bracket_pos >= self.input.len() {
            return None;
        }

        if end_bracket_pos + 1 >= self.input.len() || self.input[end_bracket_pos + 1] != '(' {
            return None;
        }

        // 匹配)
        let start_paren_pos = end_bracket_pos + 1;
        let mut end_paren_pos = start_paren_pos + 1;

        while end_paren_pos < self.input.len() && self.input[end_paren_pos] != ')' {
            end_paren_pos += 1;
        }

        if end_paren_pos >= self.input.len() {
            return None;
        }

        // 提取内容
        let label: String = self.input[start_bracket_pos + 1..end_bracket_pos]
            .iter()
            .collect();

        let url: String = self.input[start_paren_pos + 1..end_paren_pos]
            .iter()
            .collect();

        self.pos = end_paren_pos + 1;
        Some((label, url))
    }

    fn try_parse_code_span(&mut self) -> Option<Inline> {
        let start_pos = self.pos;
        let delimiter_count = self.count_delimiter('`');

        let content_start = start_pos + delimiter_count;
        let mut cur_search_pos = content_start;

        while cur_search_pos < self.input.len() {
            if self.input[cur_search_pos] == '`' {
                let mut close_count = 0;
                let mut temp_pos = cur_search_pos;

                while temp_pos < self.input.len() && self.input[temp_pos] == '`' {
                    close_count += 1;
                    temp_pos += 1;
                }

                // 前后`数量匹配
                if close_count == delimiter_count {
                    let content: String =
                        self.input[content_start..cur_search_pos].iter().collect();

                    self.pos = temp_pos;
                    return Some(Inline::CodeSpan(content));
                }

                cur_search_pos = temp_pos;
            } else {
                cur_search_pos += 1;
            }
        }

        self.pos = start_pos;
        None
    }

    fn try_parse_strikethrough(&mut self) -> Option<Inline> {
        let start_pos = self.pos;
        let delimiter_count = self.count_delimiter('~');

        if delimiter_count != 2 {
            self.pos = start_pos;
            return None;
        }

        let mut content_start = start_pos + delimiter_count;
        let mut cur_search_pos = content_start;

        while cur_search_pos < self.input.len() {
            if self.input[cur_search_pos] == '~' {
                let mut close_count = 0;
                let mut temp_pos = cur_search_pos;

                while temp_pos < self.input.len() && self.input[temp_pos] == '~' {
                    close_count += 1;
                    temp_pos += 1;
                }

                if close_count == 2 {
                    let content: String =
                        self.input[content_start..cur_search_pos].iter().collect();
                    let mut child_parser = InlineParser::new(&content);
                    let children = child_parser.parse();

                    self.pos = temp_pos;

                    return Some(Inline::Strikethrough(children));
                }

                cur_search_pos += 1;
            } else {
                cur_search_pos += 1;
            }
        }

        self.pos = start_pos;
        None
    }
}
