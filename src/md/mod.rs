mod error;

use std::str::FromStr;

use crate::document::{Document, Node, Text, TextFormat};

use error::MdError;

const RULE_CHARS: [char; 3] = ['*', '-', '_'];

pub struct MarkDown(Document);

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

fn parse_text(line: &str) -> Result<Text, MdError> {
    Ok(Text {
        content: vec![TextFormat::Normal(line.to_owned())],
    })
}

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
    if let Some(text) = line.strip_prefix("- ") {
        match parse_text(text) {
            Ok(text) => Some(Ok(Node::UnorderedList(text))),
            Err(e) => Some(Err(e)),
        }
    } else {
        None
    }
}

fn try_parse_ordered_list(line: &str) -> Option<Result<Node, MdError>> {
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
