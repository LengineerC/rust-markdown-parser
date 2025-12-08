use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum Inline<'a> {
    Text(Cow<'a, str>),
    Emphasis(Vec<Inline<'a>>),
    Strong(Vec<Inline<'a>>),
    Link {
        children: Vec<Inline<'a>>,
        url: Cow<'a, str>,
    },
    Image {
        alt: Cow<'a, str>,
        url: Cow<'a, str>,
    },
    CodeSpan(Cow<'a, str>),
    Strikethrough(Vec<Inline<'a>>),
    RawHtml(Cow<'a, str>),
}

impl<'a> Inline<'a> {
    pub fn into_owned<'b>(self) -> Inline<'b> {
        match self {
            Inline::Text(c) => Inline::Text(Cow::Owned(c.into_owned())),
            Inline::Emphasis(v) => {
                Inline::Emphasis(v.into_iter().map(|i| i.into_owned()).collect())
            }
            Inline::Strong(v) => Inline::Strong(v.into_iter().map(|i| i.into_owned()).collect()),
            Inline::Strikethrough(v) => {
                Inline::Strikethrough(v.into_iter().map(|i| i.into_owned()).collect())
            }
            Inline::CodeSpan(c) => Inline::CodeSpan(Cow::Owned(c.into_owned())),
            Inline::Link { children, url } => Inline::Link {
                children: children.into_iter().map(|i| i.into_owned()).collect(),
                url: Cow::Owned(url.into_owned()),
            },
            Inline::Image { alt, url } => Inline::Image {
                alt: Cow::Owned(alt.into_owned()),
                url: Cow::Owned(url.into_owned()),
            },
            Inline::RawHtml(c) => Inline::RawHtml(Cow::Owned(c.into_owned())),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Block<'a> {
    Heading {
        level: u8,
        children: Vec<Inline<'a>>,
    },
    Paragraph {
        children: Vec<Inline<'a>>,
    },
    BlockQuote(Vec<Block<'a>>),
    CodeBlock {
        code: Cow<'a, str>,
        language: Cow<'a, str>,
    },
    ThematicBreak,
    List {
        ordered: bool,
        items: Vec<ListItem<'a>>,
    },
    Table {
        headers: Vec<Vec<Inline<'a>>>,
        rows: Vec<Vec<Vec<Inline<'a>>>>,
        alignments: Vec<Alignment>,
    },
}

impl<'a> Block<'a> {
    pub fn into_owned<'b>(self) -> Block<'b> {
        match self {
            Block::Heading { level, children } => Block::Heading {
                level,
                children: children.into_iter().map(|i| i.into_owned()).collect(),
            },
            Block::Paragraph { children } => Block::Paragraph {
                children: children.into_iter().map(|i| i.into_owned()).collect(),
            },
            Block::CodeBlock { code, language } => Block::CodeBlock {
                code: Cow::Owned(code.into_owned()),
                language: Cow::Owned(language.into_owned()),
            },
            Block::BlockQuote(children) => {
                Block::BlockQuote(children.into_iter().map(|b| b.into_owned()).collect())
            }
            Block::ThematicBreak => Block::ThematicBreak,
            Block::List { ordered, items } => Block::List {
                ordered,
                items: items.into_iter().map(|li| li.into_owned()).collect(),
            },
            Block::Table {
                headers,
                rows,
                alignments,
            } => Block::Table {
                headers: headers
                    .into_iter()
                    .map(|h| h.into_iter().map(|i| i.into_owned()).collect())
                    .collect(),
                rows: rows
                    .into_iter()
                    .map(|r| {
                        r.into_iter()
                            .map(|cell| cell.into_iter().map(|i| i.into_owned()).collect())
                            .collect()
                    })
                    .collect(),
                alignments,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ListItem<'a> {
    pub children: Vec<Block<'a>>,
}

impl<'a> ListItem<'a> {
    pub fn into_owned<'b>(self) -> ListItem<'b> {
        ListItem {
            children: self.children.into_iter().map(|b| b.into_owned()).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Alignment {
    None,
    Left,
    Center,
    Right,
}
