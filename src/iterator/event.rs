use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum Tag<'a> {
    // === Block ===
    Paragraph,
    Heading(u8),
    BlockQuote,
    CodeBlock(Option<Cow<'a, str>>),
    List(Option<u64>),
    Item,
    Table(Vec<Alignment>),
    TableHead,
    TableRow,
    TableCell,

    // === Inline ===
    Emphasis,
    Strong,
    Strikethrough,
    Link {
        url: Cow<'a, str>,
        title: Cow<'a, str>,
    },
    Image {
        url: Cow<'a, str>,
        title: Cow<'a, str>,
        // alt作为事件流处理
    },
}

#[derive(Debug, Clone)]
pub enum Alignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub enum Event<'a> {
    Start(Tag<'a>),
    End(Tag<'a>),

    Text(Cow<'a, str>),
    CodeSpan(Cow<'a, str>),
    Html(Cow<'a, str>),
    FootnoteReference(Cow<'a, str>),
    /// 软换行（渲染为空格）
    SoftBreak,
    /// 硬换行（渲染为<br>）
    HardBreak,
    /// 水平分割先
    ThematicBreak,
    TaskListMarker(bool),
}
