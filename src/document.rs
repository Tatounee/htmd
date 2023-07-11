use bitflags::bitflags;

#[derive(Debug)]
pub struct Document {
    pub nodes: Vec<Node>,
}

#[derive(Debug)]
pub enum Node {
    H1(Text),
    H2(Text),
    H3(Text),
    H4(Text),
    H5(Text),
    Paragraphe(Text),
    UnorderedList(usize, Text), // deepth
    OrderedList(usize, Text),   // deepth
    CodeBlock(CodeBlock),
    LineBreak,
    Rule,
}

#[derive(Debug)]
pub struct CodeBlock {
    pub language: String,
    pub code: String,
}

pub fn compacte_nodes(nodes: Vec<Node>) -> Vec<Node> {
    use Node::*;

    let mut new_nodes = Vec::with_capacity(nodes.len());

    let mut in_paragraphe = None;
    let mut has_br = false;

    for node in nodes {
        if !matches!(node, Paragraphe(_)) && let Some(text) = in_paragraphe.take() {
            new_nodes.push(Paragraphe(text))
        }

        if !matches!(node, LineBreak) {
            has_br = false;
        }

        match node {
            Paragraphe(text) => {
                if let Some(p_text) = in_paragraphe.as_mut() {
                    p_text.appendnl(text)
                } else {
                    in_paragraphe = Some(text)
                }
            }
            LineBreak => {
                if !has_br {
                    new_nodes.push(node)
                }
            }
            _ => new_nodes.push(node),
        }
    }

    if let Some(text) = in_paragraphe.take() {
        new_nodes.push(Paragraphe(text))
    }

    new_nodes
}

#[derive(Debug)]
pub struct Text {
    pub content: Vec<TextFragment>,
}
impl Text {
    fn appendnl(&mut self, mut text: Text) {
        self.content
            .push(TextFragment::Stylised(Style::Normal, "\n".to_owned()));
        self.content.append(&mut text.content);
    }
}

#[derive(Debug)]
pub enum TextFragment {
    Stylised(Style, String),
    Link(String, String),
    Image(String, String),
}

impl Default for TextFragment {
    fn default() -> Self {
        Self::Stylised(Style::Normal, String::new())
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Style: u8 {
        const Normal = 0b00000001;
        const Strong = 0b00000010;
        const Emphasis = 0b00000100;
        const Code = 0b00001000;
        const Strikethrough = 0b00010000;

        const Modifier = 0b00100000;
    }
}

impl TextFragment {
    pub fn len(&self) -> usize {
        use TextFragment::*;
        match self {
            Stylised(_, s) => s.chars().count(),
            Link(alt, link) | Image(alt, link) => alt.chars().count() + link.len(),
        }
    }

    pub fn style_in(self, start: usize, end: usize, prefixe_len: usize, style: Style) -> Vec<Self> {
        assert!(start <= end);
        // println!("<{start}, {end}> ~{prefixe_len}");

        if let Self::Stylised(initial_style, s) = self {
            // println!("s = {s:?}");
            if end > s.len() {
                return Vec::new();
            }

            let (left_part, s) = s.split_at(start);
            let (left_modifier, s) = s.split_at(prefixe_len);

            let (middle_part, s) = s.split_at(end - start - prefixe_len);
            let (right_modifier, right_part) = s.split_at(prefixe_len);
            // let right_part = &s[prefixe_len..];

            let mut texts = Vec::with_capacity(3);
            if !left_part.is_empty() {
                texts.push(Self::Stylised(initial_style, left_part.to_owned()))
            }
            if !left_modifier.is_empty() {
                texts.push(Self::Stylised(Style::Modifier, left_modifier.to_owned()))
            }
            if !middle_part.is_empty() {
                texts.push(Self::Stylised(
                    initial_style | style,
                    middle_part.to_owned(),
                ))
            }
            if !right_modifier.is_empty() {
                texts.push(Self::Stylised(Style::Modifier, right_modifier.to_owned()))
            }
            if !right_part.is_empty() {
                texts.push(Self::Stylised(initial_style, right_part.to_owned()))
            }

            texts
        } else {
            panic!("Try to style unstylasible TextFormat with {style:?} in {self:?}")
        }
    }
}

pub fn style_text(
    text: &mut Vec<TextFragment>,
    prefixe_len: usize,
    mut start: usize,
    mut end: usize,
    style: Style,
) {
    assert!(start <= end);

    let mut offset = 0;
    let mut replaced_fragment = None;
    for (idx, text_fragment) in text.iter_mut().enumerate() {
        if start < text_fragment.len() + offset {
            replaced_fragment = Some(idx);
            start -= offset;
            end -= offset;
            break;
        }
        offset += text_fragment.len()
    }

    if let Some(idx) = replaced_fragment {
        let text_fragment = text.remove(idx);
        let new_fragment = text_fragment.style_in(start, end, prefixe_len, style);
        for text_fragment in new_fragment.into_iter().rev() {
            text.insert(idx, text_fragment)
        }
    }
}
