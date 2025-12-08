use crate::dom::ast::*;

pub struct HtmlRenderer;

impl HtmlRenderer {
    pub fn render(blocks: &[Block]) -> String {
        let mut output = String::new();

        for block in blocks {
            output.push_str(&Self::render_block(block));
            output.push('\n');
        }

        output
    }

    fn render_block(block: &Block) -> String {
        match block {
            Block::Heading { level, children } => {
                let content = Self::render_inlines(children);
                format!("<h{level}>{content}</h{level}>")
            }
            Block::Paragraph { children } => {
                let content = Self::render_inlines(children);
                format!("<p>{content}</p>")
            }
            Block::BlockQuote(children) => {
                let content = Self::render_blocks(children);
                format!("<blockquote>\n{content}</blockquote>")
            }
            Block::CodeBlock { code, language } => {
                let safe_code = Self::escape_html(code);
                let lang_class = if language.is_empty() {
                    String::new()
                } else {
                    format!(" class=\"language-{}\"", Self::escape_html(language))
                };
                format!("<pre><code{}>{}</code></pre>", lang_class, safe_code)
            }
            Block::ThematicBreak => "<hr />".to_string(),
            Block::List { ordered, items } => {
                let tag = if *ordered { "ol" } else { "ul" };
                let mut content = String::new();

                for item in items {
                    let item_html = Self::render_blocks(&item.children);
                    content.push_str(&format!("<li>{}</li>\n", item_html));
                }

                format!("<{}>\n{}</{}>", tag, content, tag)
            }
            Block::Table {
                headers,
                rows,
                alignments,
            } => {
                let mut html = String::from("<table>\n");

                // 表头
                html.push_str("<thead>\n<tr>\n");
                for (i, header_cell) in headers.iter().enumerate() {
                    let align = alignments.get(i).unwrap_or(&Alignment::None);
                    let style = match align {
                        Alignment::Left => " style=\"text-align: left\"",
                        Alignment::Center => " style=\"text-align: center\"",
                        Alignment::Right => " style=\"text-align: right\"",
                        Alignment::None => "",
                    };
                    let content = Self::render_inlines(header_cell);
                    html.push_str(&format!("<th{}>{}</th>\n", style, content));
                }
                html.push_str("</tr>\n</thead>\n");

                // 表体
                html.push_str("<tbody>\n");
                for row in rows {
                    html.push_str("<tr>\n");
                    for (i, cell) in row.iter().enumerate() {
                        let align = alignments.get(i).unwrap_or(&Alignment::None);
                        let style = match align {
                            Alignment::Left => " style=\"text-align: left\"",
                            Alignment::Center => " style=\"text-align: center\"",
                            Alignment::Right => " style=\"text-align: right\"",
                            Alignment::None => "",
                        };
                        let content = Self::render_inlines(cell);
                        html.push_str(&format!("<td{}>{}</td>\n", style, content));
                    }
                    html.push_str("</tr>\n");
                }
                html.push_str("</tbody>\n</table>");

                html
            }
        }
    }

    fn render_blocks(blocks: &[Block]) -> String {
        let mut output = String::new();

        for block in blocks {
            output.push_str(&Self::render_block(block));
            output.push('\n');
        }

        output
    }

    fn render_inlines(inlines: &[Inline]) -> String {
        let mut output = String::new();

        for inline in inlines {
            output.push_str(&Self::render_inline(inline));
        }

        output
    }

    fn render_inline(inline: &Inline) -> String {
        match inline {
            Inline::Text(text) => Self::escape_html(&text),
            Inline::Strong(children) => {
                let content = Self::render_inlines(children);
                format!("<strong>{content}</strong>")
            }
            Inline::Emphasis(children) => {
                let content = Self::render_inlines(children);
                format!("<em>{content}</em>")
            }
            Inline::Link { children, url } => {
                let content = Self::render_inlines(children);
                let safe_url = Self::escape_html(url);
                format!("<a href=\"{}\">{}</a>", safe_url, content)
            }
            Inline::Image { alt, url } => {
                let safe_alt = Self::escape_html(alt);
                let safe_url = Self::escape_html(url);
                format!("<img src=\"{}\" alt=\"{}\" />", safe_url, safe_alt)
            }
            Inline::CodeSpan(code) => {
                let safe_code = Self::escape_html(code);
                format!("<code>{}</code>", safe_code)
            }
            Inline::Strikethrough(children) => {
                let content = Self::render_inlines(children);
                format!("<del>{}</del>", content)
            }
            Inline::RawHtml(html) => html.to_string(),
        }
    }

    fn escape_html(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
}
