mod error;
mod queue;

use std::str::FromStr;

use crate::{
    document::{style_text, Document, Node, Style, Text, TextFragment},
};

use error::MdError;
use queue::Queue;

use self::queue::pop_min2;

const RULE_CHARS: [char; 3] = ['*', '-', '_'];

pub struct MarkDown(pub Document);

impl FromStr for MarkDown {
    type Err = MdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut text = Vec::new();

        for line in s.lines() {
            text.push(parse_line(line)?);
        }

        Ok(MarkDown(Document { text }))
    }
}

// =============================================== TEXT ===============================================

fn parse_text(line: &str) -> Result<Text, MdError> {
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
            println!("'{c}'");
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

        println!("{content:?}")
    }

    Ok(Text { content })
}

use std::iter::Peekable;
fn push_prefixe_idx_in(
    text: &mut Peekable<impl Iterator<Item = char>>,
    offset: &mut usize,
    prefixe: char,
    buffers: &mut [Queue<usize>; 3],
) {
    println!(" -> [{prefixe}], {offset}");
    let mut occurence = 0;
    let mut prefixe_offset = 0;
    while let Some(c) = text.peek() && *c == prefixe {
        occurence += 1;
        prefixe_offset += c.len_utf8();

        text.next();
    }
    println!(" {occurence}, {prefixe_offset}");

    match occurence {
        0 => (),
        1 => buffers[0].push(*offset),
        2 => buffers[1].push(*offset),
        _ => buffers[2].push(*offset),
    }
    *offset += prefixe_offset;
}

// =============================================== LINE ===============================================

fn parse_line(line: &str) -> Result<Node, MdError> {
    let line_trimed = line.trim();

    if line_trimed.is_empty() {
        return Ok(Node::LineBreak);
    }

    if let Some(node) = try_parse_header(line_trimed) {
        return node;
    }

    if let Some(node) = try_parse_unordered_list(line_trimed) {
        return node;
    }

    if let Some(node) = try_parse_ordered_list(line_trimed) {
        return node;
    }

    if let Some(node) = try_parse_code_block(line_trimed) {
        return node;
    }

    if let Some(node) = try_parse_rule(line_trimed) {
        return node;
    }

    Ok(Node::Paragraphe(parse_text(line)?))
}

fn try_parse_header(line: &str) -> Option<Result<Node, MdError>> {
    let text = line.trim_start_matches('#');
    if text.len() == line.len() {
        return None;
    }

    if let Some(text) = text.strip_prefix(char::is_whitespace) {
        let hierachy = line.len() - text.len() - 1;

        match parse_text(&line[hierachy + 1..]) {
            Ok(text) => Some(Ok(match hierachy {
                1 => Node::H1(text),
                2 => Node::H2(text),
                3 => Node::H3(text),
                4 => Node::H4(text),
                h if h >= 5 => Node::H5(text),
                _ => unreachable!(),
            })),
            Err(e) => Some(Err(e)),
        }
    } else {
        None
    }
}

fn try_parse_unordered_list(line: &str) -> Option<Result<Node, MdError>> {
    todo!("Support indentation");

    let text = line
        .strip_prefix("- ")
        .or(line.strip_prefix("+ "))
        .or(line.strip_prefix("* "));

    if let Some(text) = text {
        match parse_text(text) {
            Ok(text) => Some(Ok(Node::UnorderedList(text))),
            Err(e) => Some(Err(e)),
        }
    } else {
        None
    }
}

fn try_parse_ordered_list(line: &str) -> Option<Result<Node, MdError>> {
    todo!("Support indentation");

    let text = line.trim_start_matches(char::is_numeric);
    if text.len() == line.len() {
        return None;
    }

    if let Some(text) = text.strip_prefix(". ") {
        let number = line[0..(line.len() - text.len() - 2)]
            .parse::<u32>()
            .unwrap();
        match parse_text(text) {
            Ok(text) => Some(Ok(Node::OrderedList(number, text))),
            Err(e) => Some(Err(e)),
        }
    } else {
        None
    }
}

fn try_parse_code_block(line: &str) -> Option<Result<Node, MdError>> {
    let language = line.trim_start_matches('`');

    if line.len() - language.len() == 3 {
        Some(Ok(Node::CodeBlock(language.to_owned())))
    } else {
        None
    }
}

fn try_parse_rule(line: &str) -> Option<Result<Node, MdError>> {
    let mut character = None;
    let mut char_occ = 0;
    for c in line.chars().filter(|c| !c.is_whitespace()) {
        match character {
            Some(character) if character == c => char_occ += 1,
            Some(_) => return None,
            None => {
                character = Some(c);
                char_occ += 1;
            }
        }
    }

    match character {
        Some(character) if char_occ >= 3 && RULE_CHARS.contains(&character) => Some(Ok(Node::Rule)),
        _ => None,
    }
}
