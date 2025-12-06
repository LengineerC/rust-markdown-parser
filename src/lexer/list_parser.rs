use crate::{
    ast::{Block, ListItem},
    lexer::InlineParser,
};

pub struct ListParser {
    lines: Vec<String>,
    pos: usize,
}

impl ListParser {
    pub fn new(lines: &[String]) -> Self {
        ListParser {
            lines: lines.to_vec(),
            pos: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<Block> {
        let mut blocks = Vec::new();
        while self.pos < self.lines.len() {
            if let Some(block) = self.parse_list(0) {
                blocks.push(block);
            } else {
                self.pos += 1;
            }
        }

        blocks
    }

    fn parse_list(&mut self, min_indent: usize) -> Option<Block> {
        if self.pos >= self.lines.len() {
            return None;
        }

        // 1. 检查当前行是否是列表项，且缩进达标
        let line = &self.lines[self.pos];
        let indent = self.count_indent(line);

        // 缩进变小返回
        if indent < min_indent {
            return None;
        }

        let trimmed = line.trim();
        let (is_ordered, content_start) = self.parse_list_marker(trimmed)?;

        let mut items = Vec::new();

        // 2. 循环解析同级的列表项
        while self.pos < self.lines.len() {
            let line = &self.lines[self.pos];
            let indent = self.count_indent(line);

            if indent < min_indent {
                break;
            }

            let marker_res = self.parse_list_marker(line.trim());
            match marker_res {
                Some((ordered, start)) => {
                    // 如果列表类型变了打断
                    if ordered != is_ordered {
                        break;
                    }

                    // === 解析列表项 (ListItem) ===
                    self.pos += 1;

                    // 1. 提取当前行的内容
                    let content_text = line.trim()[start..].trim().to_string();
                    let mut item_children = Vec::new();

                    let mut inline_parser = InlineParser::new(&content_text);
                    item_children.push(Block::Paragraph {
                        children: inline_parser.parse(),
                    });

                    // 2. 贪婪解析属于该 Item 的后续行 (子列表或多行内容)
                    // 子列表的缩进必须 > 当前 indent
                    loop {
                        if self.pos >= self.lines.len() {
                            break;
                        }
                        let next_line = &self.lines[self.pos];
                        let next_indent = self.count_indent(next_line);

                        // 跳空行
                        if next_line.trim().is_empty() {
                            self.pos += 1;
                            continue;
                        }

                        // 如果下一行缩进更深，尝试递归解析子列表
                        if next_indent > indent {
                            if let Some(sub_list) = self.parse_list(next_indent) {
                                item_children.push(sub_list);
                            } else {
                                // 如果不是子列表（可能是多行文本），暂时忽略或当文本处理
                                // 这里简化：只处理子列表，忽略多行文本拼接
                                self.pos += 1;
                            }
                        } else {
                            // 下一行缩进持平或变小，说明该 Item 结束
                            break;
                        }
                    }

                    items.push(ListItem {
                        children: item_children,
                    });
                }
                None => {
                    break;
                }
            }
        }

        Some(Block::List {
            ordered: is_ordered,
            items,
        })
    }

    fn count_indent(&self, line: &str) -> usize {
        line.chars().take_while(|c| *c == ' ').count()
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
}
