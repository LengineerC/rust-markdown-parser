use std::borrow::Cow;

use crate::{
    ast::*,
    lexer::{InlineParser, ListParser},
};

pub struct Parser<'a> {
    input: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser { input }
    }

    pub fn parse(&mut self) -> Vec<Block<'a>> {
        self.parse_blocks()
    }

    pub fn preprocess(input: &str) -> String {
        input.replace("\r\n", "\n").replace("\t", "    ") + "\n"
    }

    fn flush_paragraph(&self, blocks: &mut Vec<Block<'a>>, lines: &mut Vec<&'a str>) {
        if lines.is_empty() {
            return;
        }

        let content = lines.join("\n");

        let mut inline_parser = InlineParser::new(&content);
        let inlines = inline_parser.parse();

        let owned_inlines = inlines.into_iter().map(|i| i.into_owned()).collect();

        blocks.push(Block::Paragraph {
            children: owned_inlines,
        });

        lines.clear();
    }

    fn flush_blockquote(&self, blocks: &mut Vec<Block<'a>>, lines: &mut Vec<&'a str>) {
        if lines.is_empty() {
            return;
        }

        let content = lines.join("\n");
        let mut child_parser = Parser::new(&content);
        let child_blocks = child_parser.parse();
        let owned_blocks = child_blocks.into_iter().map(|b| b.into_owned()).collect();
        blocks.push(Block::BlockQuote(owned_blocks));

        lines.clear();
    }

    fn parse_heading(&self, line: &'a str) -> Option<Block<'a>> {
        if !line.starts_with('#') {
            return None;
        }

        let mut level = 0;
        for c in line.chars() {
            if c == '#' {
                level += 1;
            } else {
                break;
            }
        }

        if level > 6 || level == 0 {
            return None;
        }

        let content_str = line[level..].trim();

        let mut inline_parser = InlineParser::new(&content_str);
        let inlines = inline_parser.parse();

        Some(Block::Heading {
            level: level as u8,
            children: inlines,
        })
    }

    fn is_thematic_break(&self, line: &str) -> bool {
        if line.len() < 3 {
            return false;
        }

        let mut chars = line.chars();
        let marker = match chars.next() {
            Some(c) => c,
            None => return false,
        };

        if !['-', '*', '_'].contains(&marker) {
            return false;
        }

        let mut cnt = 1;
        for c in chars {
            if c == marker {
                cnt += 1;
            } else if c.is_whitespace() {
                continue;
            } else {
                return false;
            }
        }

        cnt >= 3
    }

    fn count_indent(&self, line: &str) -> i32 {
        let mut cnt = 0;
        for c in line.chars() {
            if c == ' ' {
                cnt += 1;
            } else {
                break;
            }
        }

        cnt
    }

    fn parse_list_marker(&self, line: &str) -> Option<(bool, usize)> {
        let bytes = line.as_bytes();
        if bytes.is_empty() {
            return None;
        }

        let c = bytes[0] as char;

        if c == '-' || c == '*' || c == '+' {
            if line.len() == 1 || (line.len() > 1 && bytes[1] as char == ' ') {
                return Some((false, if line.len() > 1 { 2 } else { 1 }));
            }
        }

        if c.is_numeric() {
            let mut i = 1;
            while i < line.len() && (bytes[i] as char).is_numeric() {
                i += 1;
            }

            if i < line.len() && bytes[i] as char == '.' {
                if i + 1 == line.len() || (i + 1 < line.len() && bytes[i + 1] as char == ' ') {
                    return Some((true, i + 2));
                }
            }
        }

        None
    }

    fn flush_list_block(&self, blocks: &mut Vec<Block<'a>>, lines: &mut Vec<&'a str>) {
        if lines.is_empty() {
            return;
        }

        let mut list_parser = ListParser::new(lines);
        let list_blocks = list_parser.parse();
        blocks.extend(list_blocks);

        lines.clear();
    }

    fn is_table_delimiter(&self, line: &'a str) -> bool {
        let trimmed_line = line.trim();
        if !trimmed_line.contains('-') {
            return false;
        }
        trimmed_line
            .chars()
            .all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
    }

    fn parse_table_alignments(&self, line: &'a str) -> Vec<Alignment> {
        let parts = self.split_table_row(line);

        parts
            .into_iter()
            .map(|part| {
                let s = part.trim();
                let start_with_colon = s.starts_with(':');
                let end_with_colon = s.ends_with(':');

                if start_with_colon && end_with_colon {
                    Alignment::Center
                } else if end_with_colon {
                    Alignment::Right
                } else if start_with_colon {
                    Alignment::Left
                } else {
                    Alignment::None
                }
            })
            .collect()
    }

    fn split_table_row(&self, line: &'a str) -> Vec<&'a str> {
        let trimmed_line = line.trim();
        let content = if trimmed_line.starts_with('|') {
            &trimmed_line[1..]
        } else {
            trimmed_line
        };

        let content = if content.ends_with('|') {
            &content[..content.len() - 1]
        } else {
            content
        };

        content.split('|').collect()
    }

    fn parse_table_row(&self, line: &'a str) -> Vec<Vec<Inline<'a>>> {
        let parts = self.split_table_row(line);
        parts
            .into_iter()
            .map(|part| {
                let content = part.trim();
                let mut parser = InlineParser::new(content);
                parser.parse()
            })
            .collect()
    }

    fn parse_blocks(&mut self) -> Vec<Block<'a>> {
        let lines: Vec<&'a str> = self.input.lines().collect();
        let mut idx = 0;

        let mut blocks = Vec::new();
        let mut cur_paragraph_lines: Vec<&'a str> = Vec::new();

        // 引用块缓存
        let mut cur_quoto_lines: Vec<&'a str> = Vec::new();

        // codeblock
        let mut in_code_block = false;
        let mut cur_code_lines: Vec<&'a str> = Vec::new();
        let mut language = "";

        // list
        let mut cur_list_lines: Vec<&'a str> = Vec::new();

        while idx < lines.len() {
            let line = lines[idx];
            let trimmed_line = line.trim();

            // === CodeBlock ===
            if in_code_block {
                if trimmed_line.starts_with("```") {
                    blocks.push(Block::CodeBlock {
                        code: Cow::Owned(cur_code_lines.join("\n")),
                        language: Cow::Borrowed(language),
                    });
                    in_code_block = false;
                    cur_code_lines.clear();
                    language = "";
                } else {
                    cur_code_lines.push(line);
                }

                idx += 1;
                continue;
            }

            // 开始CodeBlock
            if trimmed_line.starts_with("```") {
                self.flush_paragraph(&mut blocks, &mut cur_paragraph_lines);
                if !cur_quoto_lines.is_empty() {
                    self.flush_blockquote(&mut blocks, &mut cur_quoto_lines);
                }
                if !cur_list_lines.is_empty() {
                    self.flush_list_block(&mut blocks, &mut cur_list_lines);
                }

                in_code_block = true;
                language = trimmed_line.strip_prefix("```").unwrap_or("").trim();

                idx += 1;
                continue;
            }

            // === Table ===
            if trimmed_line.contains('|') && idx + 1 < lines.len() {
                let next_line = lines[idx + 1].trim();

                if self.is_table_delimiter(next_line) {
                    self.flush_paragraph(&mut blocks, &mut cur_paragraph_lines);
                    if !cur_quoto_lines.is_empty() {
                        self.flush_blockquote(&mut blocks, &mut cur_quoto_lines);
                    }
                    if !cur_list_lines.is_empty() {
                        self.flush_list_block(&mut blocks, &mut cur_list_lines);
                    }

                    let headers = self.parse_table_row(trimmed_line);
                    let alignments = self.parse_table_alignments(next_line);

                    let mut rows = Vec::new();
                    idx += 2;

                    while idx < lines.len() {
                        let row_line = lines[idx].trim();

                        if !row_line.contains('|') || row_line.is_empty() {
                            break;
                        }

                        rows.push(self.parse_table_row(row_line));
                        idx += 1;
                    }

                    blocks.push(Block::Table {
                        headers,
                        rows,
                        alignments,
                    });
                    continue;
                }
            }

            // === ThematicBreak ===
            if self.is_thematic_break(line) {
                self.flush_paragraph(&mut blocks, &mut cur_paragraph_lines);

                if !cur_quoto_lines.is_empty() {
                    self.flush_blockquote(&mut blocks, &mut cur_quoto_lines);
                }
                if !cur_list_lines.is_empty() {
                    self.flush_list_block(&mut blocks, &mut cur_list_lines);
                }

                blocks.push(Block::ThematicBreak);
                idx += 1;
                continue;
            }

            // === List ===
            // 正在收集列表块
            if !cur_list_lines.is_empty() {
                let is_marker = self.parse_list_marker(trimmed_line).is_some();
                // 使用原行计算缩进
                let indent = self.count_indent(line);
                let is_indented = indent >= 2;

                // if self.is_thematic_break(trimmed) {
                //     self.flush_list_block(&mut blocks, &mut cur_list_lines);
                //     blocks.push(Block::ThematicBreak);
                //     continue;
                // }

                if trimmed_line.starts_with('#')
                    || trimmed_line.starts_with('>')
                    || trimmed_line.starts_with('`')
                {
                    self.flush_list_block(&mut blocks, &mut cur_list_lines);
                } else if trimmed_line.is_empty() {
                    cur_list_lines.push(line);

                    idx += 1;
                    continue;
                }
                // 如果是列表标记有缩进，继续收集
                else if is_marker || is_indented {
                    cur_list_lines.push(line);

                    idx += 1;
                    continue;
                } else {
                    self.flush_list_block(&mut blocks, &mut cur_list_lines);
                }
            }

            if self.parse_list_marker(trimmed_line).is_some() {
                self.flush_paragraph(&mut blocks, &mut cur_paragraph_lines);
                if !cur_quoto_lines.is_empty() {
                    self.flush_blockquote(&mut blocks, &mut cur_quoto_lines);
                }

                // if self.is_thematic_break(trimmed) {
                //     blocks.push(Block::ThematicBreak);
                //     continue;
                // }

                cur_list_lines.push(line);

                idx += 1;
                continue;
            }

            // === BlockQuote ===
            if trimmed_line.starts_with('>') {
                self.flush_paragraph(&mut blocks, &mut cur_paragraph_lines);
                let content = trimmed_line.strip_prefix('>').unwrap_or("").trim_start();
                cur_quoto_lines.push(content);

                idx += 1;
                continue;
            }

            if !cur_quoto_lines.is_empty() {
                self.flush_blockquote(&mut blocks, &mut cur_quoto_lines);
            }

            // === Empty ===
            if trimmed_line.is_empty() {
                self.flush_paragraph(&mut blocks, &mut cur_paragraph_lines);

                idx += 1;
                continue;
            }

            // === Heading ===
            if let Some(heading) = self.parse_heading(trimmed_line) {
                self.flush_paragraph(&mut blocks, &mut cur_paragraph_lines);
                blocks.push(heading);

                idx += 1;
                continue;
            }

            // === Paragraph ===
            cur_paragraph_lines.push(trimmed_line);
            idx += 1;
        }

        // 处理缓存
        self.flush_paragraph(&mut blocks, &mut cur_paragraph_lines);
        self.flush_blockquote(&mut blocks, &mut cur_quoto_lines);

        if in_code_block {
            blocks.push(Block::CodeBlock {
                code: Cow::Owned(cur_code_lines.join("\n")),
                language: Cow::Borrowed(language),
            });
        }

        self.flush_list_block(&mut blocks, &mut cur_list_lines);

        blocks
    }
}
