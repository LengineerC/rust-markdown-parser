use std::{borrow::Cow, collections::VecDeque};

use crate::iterator::{Event, Scanner, Tag};

#[derive(Debug, Clone, Copy, PartialEq)]
enum BlockContext {
    Document,   // 顶层文档
    BlockQuote, // 在引用块内
    List,       // 在列表内 (预留)
}

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    event_queue: VecDeque<Event<'a>>,
    finished: bool,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            scanner: Scanner::new(input),
            event_queue: VecDeque::new(),
            finished: false,
        }
    }

    fn parse_next_block(&mut self) {
        self.skip_blank_lines();

        if self.scanner.is_eof() {
            self.finished = true;
            return;
        }

        self.dispatch_block(BlockContext::Document);
    }

    fn skip_blank_lines(&mut self) {
        loop {
            let start = self.scanner.position();
            self.scanner.skip_whitespace();
            if self.scanner.match_char('\n') {
                continue;
            }

            self.scanner.set_position(start);
            break;
        }
    }

    fn dispatch_block(&mut self, ctx: BlockContext) {
        if self.is_at_heading_start() {
            self.parse_heading();
            return;
        }

        if self.is_at_blockquote_start() {
            self.parse_blockquote(ctx);
            return;
        }

        if self.is_at_list_item_start() {
            self.parse_list(ctx);
            return;
        }

        self.parse_paragraph(ctx);
    }

    fn parse_heading(&mut self) {
        let mut level = 0;
        while self.scanner.match_char('#') {
            level += 1;
        }
        // 容错处理
        if level > 6 {
            level = 6;
        }

        self.scanner.skip_whitespace();

        let content_start = self.scanner.position();

        while let Some(c) = self.scanner.peek() {
            if c == '\n' {
                break;
            }
            self.scanner.next_char();
        }
        let content_end = self.scanner.position();
        self.scanner.match_char('\n'); // 消费换行

        let content = self.scanner.slice(content_start, content_end);

        let tag = Tag::Heading(level as u8);
        self.event_queue.push_back(Event::Start(tag.clone()));
        self.event_queue
            .push_back(Event::Text(Cow::Borrowed(content)));
        self.event_queue.push_back(Event::End(tag));
    }

    fn parse_blockquote(&mut self, _parent_ctx: BlockContext) {
        let tag = Tag::BlockQuote;
        self.event_queue.push_back(Event::Start(tag.clone()));

        loop {
            // 1. 检查引用前缀 ">"
            let line_start = self.scanner.position();
            self.scanner.skip_whitespace();

            if !self.scanner.match_char('>') {
                // 没有 ">"，引用块结束
                // (注意：这里暂不支持 Lazy Continuation，即引用块中间夹杂无 > 的行)
                self.scanner.set_position(line_start);
                break;
            }

            // 吃掉可选空格
            self.scanner.match_char(' ');

            // 2. 递归解析内部结构

            // 处理空引用行"> \n"
            if self.scanner.peek() == Some('\n') {
                self.scanner.next_char();
            } else {
                self.dispatch_block(BlockContext::BlockQuote);
            }

            if self.scanner.is_eof() {
                break;
            }
        }

        self.event_queue.push_back(Event::End(tag));
    }

    fn parse_list(&mut self, _parent_ctx: BlockContext) {
        let tag = Tag::List(None); // 简化：默认为无序
        self.event_queue.push_back(Event::Start(tag.clone()));

        while self.is_at_list_item_start() {
            self.event_queue.push_back(Event::Start(Tag::Item));

            // 消费"- "
            self.scanner.skip_whitespace();
            self.scanner.next_char();
            self.scanner.skip_whitespace();

            // 解析内容
            // 列表项也是容器，可以把 Context 设为 List (或者继续传递 BlockContext)
            // 这里为了简单，列表项内部当作普通 Block 处理
            if self.scanner.peek() == Some('\n') {
                self.scanner.next_char();
            } else {
                self.dispatch_block(BlockContext::List);
            }

            self.event_queue.push_back(Event::End(Tag::Item));

            // 检查下一行是否还是列表项
            let pos = self.scanner.position();
            self.scanner.skip_whitespace();
            // 如果遇到空行，CommonMark 稍微复杂，这里简化处理：跳过空行继续找 Item
            if self.scanner.match_char('\n') {
            } else {
                self.scanner.set_position(pos);
            }

            if !self.is_at_list_item_start() {
                break;
            }
        }
        self.event_queue.push_back(Event::End(tag));
    }

    fn parse_paragraph(&mut self, ctx: BlockContext) {
        let tag = Tag::Paragraph;
        self.event_queue.push_back(Event::Start(tag.clone()));

        loop {
            // 读取当前行内容
            self.parse_text_line();

            if self.scanner.is_eof() {
                break;
            }

            // 决定是否继续读取下一行
            let save_pos = self.scanner.position();

            // 1. 下一行是空行，段落结束
            self.scanner.skip_whitespace();
            if self.scanner.match_char('\n') {
                break;
            }

            // 2. 下一行开始了新的块结构, 打断段落
            self.scanner.set_position(save_pos);
            if self.is_interrupted(ctx) {
                break;
            }

            // 3. 处于BlockQuote上下文中，下一行必须有>
            if ctx == BlockContext::BlockQuote {
                let _ = self.scanner.position();
                self.scanner.skip_whitespace();
                if !self.scanner.match_char('>') {
                    self.scanner.set_position(save_pos);
                    break;
                }
                self.scanner.match_char(' ');
            }
        }

        self.event_queue.push_back(Event::End(tag));
    }

    fn is_interrupted(&mut self, _ctx: BlockContext) -> bool {
        let start = self.scanner.position();

        if self.is_at_heading_start() {
            return true;
        }
        if self.is_at_blockquote_start() {
            return true;
        }
        if self.is_at_list_item_start() {
            return true;
        }

        self.scanner.set_position(start);
        false
    }

    fn is_at_heading_start(&mut self) -> bool {
        let start = self.scanner.position();
        // 必须在行首 (或者前导空白已被跳过)
        // self.scanner.skip_whitespace(); // 假设调用前已处理或位置正确

        let mut level = 0;
        while self.scanner.match_char('#') {
            level += 1;
        }

        if level == 0 || level > 6 {
            self.scanner.set_position(start);
            return false;
        }

        let valid = match self.scanner.peek() {
            Some(' ') | Some('\t') | Some('\n') | None => true,
            _ => false,
        };

        self.scanner.set_position(start);
        valid
    }

    fn is_at_blockquote_start(&mut self) -> bool {
        let start = self.scanner.position();
        self.scanner.skip_whitespace();
        let res = self.scanner.match_char('>');
        self.scanner.set_position(start);
        res
    }

    fn is_at_list_item_start(&mut self) -> bool {
        let start = self.scanner.position();
        self.scanner.skip_whitespace();
        if let Some(c) = self.scanner.peek() {
            if c == '-' || c == '*' || c == '+' {
                self.scanner.next_char();
                if matches!(self.scanner.peek(), Some(' ') | Some('\n') | None) {
                    self.scanner.set_position(start);
                    return true;
                }
            }
        }
        self.scanner.set_position(start);
        false
    }

    fn parse_text_line(&mut self) {
        let start = self.scanner.position();
        while let Some(c) = self.scanner.peek() {
            if c == '\n' {
                break;
            }
            self.scanner.next_char();
        }
        let end = self.scanner.position();

        if end > start {
            let text = self.scanner.slice(start, end);
            self.event_queue.push_back(Event::Text(Cow::Borrowed(text)));
        }

        if self.scanner.match_char('\n') {
            let raw = self.scanner.slice(start, end);
            if raw.ends_with("  ") {
                self.event_queue.push_back(Event::HardBreak);
            } else {
                self.event_queue.push_back(Event::SoftBreak);
            }
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.event_queue.pop_front() {
            return Some(event);
        }

        if self.finished {
            return None;
        }

        self.parse_next_block();

        self.event_queue.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_detection() {
        let inputs = vec![
            ("# Foo", true),
            ("## Foo", true),
            ("###### Foo", true),
            ("#", true),
            ("#\n", true),
            ("####### Foo", false),
            ("#Foo", false),
            (" # Foo", false),
        ];

        for (input, expected) in inputs {
            let mut parser = Parser::new(input);

            assert_eq!(
                parser.is_at_heading_start(),
                expected,
                "Failed for input: {:?}",
                input
            );
        }
    }
}
