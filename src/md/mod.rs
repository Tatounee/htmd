mod queue;

use std::str::FromStr;

use crate::document::{style_text, CodeBlock, Document, Node, Style, Text, TextFragment, compacte_nodes};

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

    let mut offset = 0;

    let mut chars = line.chars().peekable();
    loop {
        if chars.peek().is_none() {
            break;
        }
        push_prefixe_idx_in(&mut chars, &mut offset, '*', &mut asterisks);
        push_prefixe_idx_in(&mut chars, &mut offset, '_', &mut underscores);
        push_prefixe_idx_in(&mut chars, &mut offset, '`', &mut backticks);
        push_prefixe_idx_in(&mut chars, &mut offset, '~', &mut tildes);

        while let Some( c) = chars.peek() && !['*', '_', '`', '~'].contains(c) {
            // println!("'{c}'");
            offset += c.len_utf8();
            if *c == '\\' {
                chars.next();
            }
            chars.next();
        }
    }

    let mut buffers = [asterisks, underscores, backticks, tildes];
    let mut content = vec![TextFragment::Stylised(Style::Normal, line.to_owned())];

    while let Some(((start, end), (x, y))) = pop_min2(&mut buffers) {
        match y {
            // Asterisk * and underscore _
            0 | 1 => match x {
                0 => style_text(&mut content, x + 1, start, end, Style::Emphasis),
                1 => style_text(&mut content, x + 1, start, end, Style::Strong),
                2 => {
                    style_text(
                        &mut content,
                        x + 1,
                        start,
                        end,
                        Style::Emphasis | Style::Strong,
                    );
                }
                _ => unreachable!(),
            },
            // Backtick `
            2 => style_text(&mut content, x + 1, start, end, Style::Code),
            // Tilde ~
            3 if x == 1 => style_text(&mut content, x + 1, start, end, Style::Strikethrough),
            _ => (),
        }

        // println!("{content:?}")
    }

    Text { content }
}

use std::iter::Peekable;
fn push_prefixe_idx_in(
    text: &mut Peekable<impl Iterator<Item = char>>,
    offset: &mut usize,
    prefixe: char,
    buffers: &mut [Queue<usize>; 3],
) {
    // println!(" -> [{prefixe}], {offset}");
    let mut occurence = 0;
    let mut prefixe_offset = 0;
    while let Some(c) = text.peek() && *c == prefixe {
        occurence += 1;
        prefixe_offset += c.len_utf8();

        text.next();
    }
    // println!(" {occurence}, {prefixe_offset}");

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
        Some(match hierachy {
            1 => Node::H1(text),
            2 => Node::H2(text),
            3 => Node::H3(text),
            4 => Node::H4(text),
            h if h >= 5 => Node::H5(text),
            _ => unreachable!(),
        })
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

    text.map(|text| Node::UnorderedList(deepth, parse_text(text)))
}

fn try_parse_ordered_list(line: &str) -> Option<Node> {
    let deepth = calcule_deepth(line);
    let line = line.trim();

    let text = line.trim_start_matches(char::is_numeric);
    if text.len() == line.len() {
        return None;
    }

    let text = text.strip_prefix(". ");

    text.map(|text| Node::OrderedList(deepth, parse_text(text)))
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
