mod queue;

use std::str::FromStr;

use crate::document::{
    compacte_nodes, CodeBlock, Document, ListKind, Node, Span, Style, Text, TextFragment,
};

use queue::Queue;

use self::queue::pop_min2;

const RULE_CHARS: [char; 3] = ['*', '-', '_'];

pub struct MarkDown(pub Document);

impl FromStr for MarkDown {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut nodes = Vec::new();

        let mut codeblock = None;
        for line in s.lines() {
            if let Some(node) = parse_line(line, &mut codeblock) {
                nodes.push(node);
            }
        }

        let nodes = compacte_nodes(nodes);
        Ok(MarkDown(Document { nodes }))
    }
}

// =============================================== TEXT ===============================================

fn parse_text(line: &str) -> Text {
    let mut asterisks = [Queue::new(), Queue::new(), Queue::new()];
    let mut underscores = [Queue::new(), Queue::new(), Queue::new()];
    let mut backticks = [Queue::new(), Queue::new(), Queue::new()];
    let mut tildes = [Queue::new(), Queue::new(), Queue::new()];

    let mut links_images = Vec::new();
    let mut escaped = Vec::new();

    let mut offset = 0;

    let mut chars = line.chars().peekable();
    loop {
        if chars.peek().is_none() {
            break;
        }

        try_push_link_image_in(&mut chars, &mut offset, &mut links_images);

        try_push_prefixe_idx_in(&mut chars, &mut offset, '*', &mut asterisks);
        try_push_prefixe_idx_in(&mut chars, &mut offset, '_', &mut underscores);
        try_push_prefixe_idx_in(&mut chars, &mut offset, '`', &mut backticks);
        try_push_prefixe_idx_in(&mut chars, &mut offset, '~', &mut tildes);

        while let Some( c) = chars.peek() && !['*', '_', '`', '~'].contains(c) {
            offset += c.len_utf8();
            if *c == '\\' {
                escaped.push(offset - c.len_utf8());
                chars.next();
            }
            chars.next();
        }
    }

    let mut buffers = [asterisks, underscores, backticks, tildes];
    let mut text = Text {
        content: vec![TextFragment::Stylised(Style::Normal, line.to_owned())],
    };

    while let Some(((start, end), (x, y))) = pop_min2(&mut buffers) {
        let span = Span::from_start_end(start, end);
        match y {
            // Asterisk * and underscore _
            0 | 1 => match x {
                0 => text.style(x + 1, span, Style::Emphasis),
                1 => text.style(x + 1, span, Style::Strong),
                2 => {
                    text.style(x + 1, span, Style::Emphasis | Style::Strong);
                }
                _ => unreachable!(),
            },
            // Backtick `
            2 => text.style(x + 1, span, Style::Code),
            // Tilde ~
            3 if x == 1 => text.style(x + 1, span, Style::Strikethrough),
            _ => (),
        }
    }

    for (span, frag) in links_images {
        text.replace(span, frag)
    }

    for escape_char in escaped {
        text.remove(Span {
            offset: escape_char,
            length: 1,
        })
    }

    text
}

use std::iter::Peekable;
fn try_push_link_image_in(
    text: &mut Peekable<impl Iterator<Item = char> + Clone>,
    offset: &mut usize,
    buffer: &mut Vec<(Span, TextFragment)>,
) {
    let c = text.peek().cloned();
    if c.is_none() || !['!', '['].contains(&c.unwrap()) {
        return;
    }

    let mut text_cloned = text.clone();
    let mut link_offset = 0;

    let is_image = c.unwrap() == '!';
    if is_image {
        text_cloned.next();
        link_offset += '!'.len_utf8();
    }

    // First '['
    if let Some(c) = text_cloned.next() && c == '[' {
        link_offset += '['.len_utf8();
    } else {
        return;
    }

    // Alt
    let mut alt = String::new();
    let mut succes = false;
    for c in text_cloned.by_ref() {
        link_offset += c.len_utf8();
        if c != ']' {
            alt.push(c);
        } else {
            succes = true;
            break;
        }
    }
    if !succes {
        return;
    }

    // First '('
    if let Some(c) = text_cloned.next() && c == '(' {
        link_offset += '('.len_utf8();
    } else {
        return;
    }

    // link
    let mut link = String::new();
    let mut succes = false;
    for c in text_cloned.by_ref() {
        link_offset += c.len_utf8();
        if c != ')' {
            link.push(c);
        } else {
            succes = true;
            break;
        }
    }
    if !succes {
        return;
    }

    // "Return"
    let span = Span {
        offset: *offset,
        length: link_offset,
    };
    if is_image {
        buffer.push((span, TextFragment::Image(alt, link)))
    } else {
        buffer.push((span, TextFragment::Link(alt, link)))
    }
    *text = text_cloned;
    *offset += link_offset;
}

fn try_push_prefixe_idx_in(
    text: &mut Peekable<impl Iterator<Item = char>>,
    offset: &mut usize,
    prefixe: char,
    buffers: &mut [Queue<usize>; 3],
) {
    let mut occurence = 0;
    let mut prefixe_offset = 0;
    while let Some(c) = text.peek() && *c == prefixe {
        occurence += 1;
        prefixe_offset += c.len_utf8();

        text.next();
    }

    match occurence {
        0 => (),
        1 => buffers[0].push(*offset),
        2 => buffers[1].push(*offset),
        _ => buffers[2].push(*offset),
    }
    *offset += prefixe_offset;
}

// =============================================== LINE ===============================================

fn parse_line(line: &str, codeblock: &mut Option<CodeBlock>) -> Option<Node> {
    match codeblock {
        Some(codeblock_inner) => {
            if let Some(language) = is_code_block_annonce(line) && language.is_empty() {
                Some(Node::CodeBlock(codeblock.take().unwrap()))
            } else {
                codeblock_inner.code.push_str(line);
                codeblock_inner.code.push('\n');
                None
            }
        }
        None => {
            if let Some(language) = is_code_block_annonce(line) {
                *codeblock = Some(CodeBlock {language, code: String::new()});
                return None;
            }

            if line.trim().is_empty() {
                return Some(Node::LineBreak);
            }

            if let Some(node) = try_parse_header(line) {
                return Some(node);
            }

            if let Some(node) = try_parse_unordered_list(line) {
                return Some(node);
            }

            if let Some(node) = try_parse_ordered_list(line) {
                return Some(node);
            }

            if let Some(node) = try_parse_rule(line) {
                return Some(node);
            }

            Some(Node::Paragraphe(parse_text(line)))
        }
    }
}

fn try_parse_header(line: &str) -> Option<Node> {
    let line = line.trim();

    let text = line.trim_start_matches('#');
    if text.len() == line.len() {
        return None;
    }

    if let Some(text) = text.strip_prefix(char::is_whitespace) {
        let hierachy = line.len() - text.len() - 1;

        let text = parse_text(&line[hierachy + 1..]);
        Some(Node::Header(hierachy.min(5), text))
    } else {
        None
    }
}

fn try_parse_unordered_list(line: &str) -> Option<Node> {
    let deepth = calcule_deepth(line);
    let line = line.trim();

    let text = line
        .strip_prefix("- ")
        .or(line.strip_prefix("+ "))
        .or(line.strip_prefix("* "));

    text.map(|text| Node::List(ListKind::Unordere(deepth), parse_text(text)))
}

fn try_parse_ordered_list(line: &str) -> Option<Node> {
    let deepth = calcule_deepth(line);
    let line = line.trim();

    let text = line.trim_start_matches(char::is_numeric);
    if text.len() == line.len() {
        return None;
    }

    let text = text.strip_prefix(". ");

    text.map(|text| Node::List(ListKind::Oredred(deepth), parse_text(text)))
}

fn calcule_deepth(line: &str) -> usize {
    let mut tab_occ = 0;
    let mut space_occ = 0;
    let mut chars = line.chars();
    while let Some(c) = chars.next() && c.is_whitespace() {
        if c == '\t' {
            tab_occ += 1
        } else {
            space_occ += 1
        }
    }

    tab_occ + (space_occ / 4)
}

fn is_code_block_annonce(line: &str) -> Option<String> {
    let line = line.trim();

    let language = line.trim_start_matches('`');

    if line.len() - language.len() == 3 {
        Some(language.to_owned())
    } else {
        None
    }
}

fn try_parse_rule(line: &str) -> Option<Node> {
    let line = line.trim();

    let mut character = None;
    let mut char_occ = 0;
    for c in line.chars().filter(|c| !c.is_whitespace()) {
        match character {
            Some(character) if character == c => char_occ += 1,
            None if RULE_CHARS.contains(&c) => {
                character = Some(c);
                char_occ += 1;
            }
            _ => return None,
        }
    }

    if character.is_some() && char_occ >= 3 {
        Some(Node::Rule)
    } else {
        None
    }
    // match character {
    //     Some(character) if char_occ >= 3 && RULE_CHARS.contains(&character) => Some(Node::Rule),
    //     _ => None,
    // }
}
