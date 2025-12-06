#[derive(Debug, Clone)]
pub enum Inline {
    Text(String),
    Emphasis(Vec<Inline>),
    Strong(Vec<Inline>),
    Link { children: Vec<Inline>, url: String },
    Image { alt: String, url: String },
    CodeSpan(String),
    Strikethrough(Vec<Inline>),
    RawHtml(String),
}

#[derive(Debug, Clone)]
pub enum Block {
    Heading {
        level: u8,
        children: Vec<Inline>,
    },
    Paragraph {
        children: Vec<Inline>,
    },
    BlockQuote(Vec<Block>),
    CodeBlock {
        code: String,
        language: String,
    },
    ThematicBreak,
    List {
        ordered: bool,
        items: Vec<ListItem>,
    },
    Table {
        headers: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
        alignments: Vec<Alignment>,
    },
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub children: Vec<Block>,
}

#[derive(Debug, Clone)]
pub enum Alignment {
    None,
    Left,
    Center,
    Right,
}
