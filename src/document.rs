
#[derive(Debug)]
pub struct Document {
    pub text: Vec<Node>
}

#[derive(Debug)]
pub enum Node {
    H1(Text),
    H2(Text),
    H3(Text),
    H4(Text),
    H5(Text),
    Paragraphe(Text),
    UnorderedList(Text),
    OrderedList(u32, Text),
    CodeBlock(String),
    LineBreak,
    Rule,
}

#[derive(Debug)]
pub struct Text {
    pub content: Vec<TextFormat>
}

#[derive(Debug)]
pub enum TextFormat {
    Normal(String),
    Bold(String),
    Italics(String),
    Code(String),
    Link(String, String),
    Image(String, String),
    Strikethrough(String)
}