use bitflags::bitflags;

#[derive(Debug)]
pub struct Document {
    pub nodes: Vec<Node>,
}

#[derive(Debug)]
pub enum Node {
    Header(usize, Text),
    Paragraphe(Text),
    List(ListKind, Text),
    CodeBlock(CodeBlock),
    LineBreak,
    Rule,
}

#[derive(Debug)]
pub struct CodeBlock {
    pub language: String,
    pub code: String,
}

#[derive(Debug)]
pub enum ListKind {
    Oredred(usize),
    Unordere(usize),
}

impl ListKind {
    #[inline]
    pub const fn deepth(&self) -> usize {
        match self {
            Self::Oredred(d) | Self::Unordere(d) => *d,
        }
    }
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
                    new_nodes.push(node);
                    has_br = true;
                }
            }
            Header(_, _) => {
                has_br = true;
                new_nodes.push(node)
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

    pub fn style(&mut self, prefixe_len: usize, mut span: Span, style: Style) {
        let modified_fragment = self.find_modified_fragment(&mut span);

        if let Some(idx) = modified_fragment {
            let text_fragment = self.content.remove(idx);
            let new_fragment = text_fragment.style_in(span, prefixe_len, style);
            for text_fragment in new_fragment.into_iter().rev() {
                self.content.insert(idx, text_fragment)
            }
        }
    }

    pub fn replace(&mut self, mut span: Span, frag: TextFragment) {
        let modified_fragment = self.find_modified_fragment(&mut span);

        if let Some(idx) = modified_fragment {
            let text_fragment = self.content.remove(idx);
            let new_fragment = text_fragment.replace(span, frag);
            for text_fragment in new_fragment.into_iter().rev() {
                self.content.insert(idx, text_fragment)
            }
        }
    }

    pub fn remove(&mut self, mut span: Span) {
        let modified_fragment = self.find_modified_fragment(&mut span);

        if let Some(idx) = modified_fragment {
            let text_fragment = self.content.remove(idx);
            let new_fragment = text_fragment.remove(span);
            for text_fragment in new_fragment.into_iter().rev() {
                self.content.insert(idx, text_fragment)
            }
        }
    }

    fn find_modified_fragment(&mut self, span: &mut Span) -> Option<usize> {
        let mut offset = 0;
        let mut replaced_fragment = None;
        for (idx, text_fragment) in self.content.iter_mut().enumerate() {
            if span.offset < text_fragment.len() + offset {
                replaced_fragment = Some(idx);
                span.offset -= offset;
                break;
            }
            offset += text_fragment.len()
        }
        replaced_fragment
    }
}

#[derive(Debug)]
pub enum TextFragment {
    Stylised(Style, String),
    Link(String, String),  // alt, link
    Image(String, String), // alt, path
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

    pub fn style_in(self, span: Span, prefixe_len: usize, style: Style) -> Vec<Self> {
        if let Self::Stylised(initial_style, s) = &self {
            if span.offset + span.length > s.len() {
                return vec![self];
            }

            let (left_part, s) = s.split_at(span.offset);
            let (left_modifier, s) = s.split_at(prefixe_len);

            let (middle_part, s) = s.split_at(span.length - prefixe_len);
            let (right_modifier, right_part) = s.split_at(prefixe_len);

            let mut texts = Vec::with_capacity(3);
            if !left_part.is_empty() {
                texts.push(Self::Stylised(*initial_style, left_part.to_owned()))
            }
            if !left_modifier.is_empty() {
                texts.push(Self::Stylised(Style::Modifier, left_modifier.to_owned()))
            }
            if !middle_part.is_empty() {
                texts.push(Self::Stylised(
                    *initial_style | style,
                    middle_part.to_owned(),
                ))
            }
            if !right_modifier.is_empty() {
                texts.push(Self::Stylised(Style::Modifier, right_modifier.to_owned()))
            }
            if !right_part.is_empty() {
                texts.push(Self::Stylised(*initial_style, right_part.to_owned()))
            }

            texts
        } else {
            panic!("Try to style unstylasible TextFormat with {style:?} in {self:?}")
        }
    }

    fn replace(self, span: Span, frag: TextFragment) -> Vec<Self> {
        if let Self::Stylised(initial_style, s) = &self {
            if span.offset + span.length > s.len() {
                return vec![self];
            }

            let (left_part, s) = s.split_at(span.offset);
            let (_, right_part) = s.split_at(span.length);

            vec![
                Self::Stylised(*initial_style, left_part.to_owned()),
                frag,
                Self::Stylised(*initial_style, right_part.to_owned()),
            ]
        } else {
            panic!("Try to replace unreplacable TextFormat with {frag:?} in {self:?}")
        }
    }

    fn remove(self, span: Span) -> Vec<Self> {
        if let Self::Stylised(initial_style, s) = &self {
            if span.offset + span.length > s.len() {
                return vec![self];
            }

            let (left_part, s) = s.split_at(span.offset);
            let (_, right_part) = s.split_at(span.length);

            vec![
                Self::Stylised(*initial_style, left_part.to_owned()),
                Self::Stylised(*initial_style, right_part.to_owned()),
            ]
        } else {
            panic!("Try to remove unexisting text")
        }
    }
}

pub struct Span {
    pub offset: usize,
    pub length: usize,
}

impl Span {
    pub fn from_start_end(start: usize, end: usize) -> Self {
        assert!(start <= end);
        Self {
            offset: start,
            length: end - start,
        }
    }
}
