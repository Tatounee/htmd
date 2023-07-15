use std::fmt;

use crate::{
    document::{Document, ListKind, Node, Style, Text, TextFragment},
    md::MarkDown,
};

pub struct HTML<'a>(pub Document<'a>);

impl<'a> From<MarkDown<'a>> for HTML<'a> {
    fn from(markdown: MarkDown<'a>) -> Self {
        Self(markdown.0)
    }
}

impl<'a> fmt::Display for HTML<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut lists: Vec<&ListKind> = Vec::new();

        for line in self.0.nodes.iter() {
            let mut reset_list = true;
            match line {
                Node::Header(level, text) => f.write_fmt(format_args!(
                    "<h{level}>{}</h{level}>",
                    text_to_htlm(text)
                ))?,
                Node::Paragraphe(text) => f.write_fmt(format_args!(
                    "<p>{}</p>",
                    text_to_htlm(text).replace('\n', "<br>")
                ))?,
                Node::CodeBlock(codeblock) => f.write_fmt(format_args!(
                    "<pre><code class={}>{}</code></pre>",
                    codeblock.language,
                    codeblock.fetch().unwrap()
                ))?,
                Node::List(list_kind, text) => {
                    reset_list = false;
                    let deepth_delta = lists
                        .last()
                        .map(|pre_list| list_kind.deepth() as isize - pre_list.deepth() as isize)
                        .unwrap_or(1);

                    match deepth_delta {
                        0 => f.write_fmt(format_args!("<li>{}</li>", text_to_htlm(text)))?,
                        d if d > 0 => {
                            for _ in 0..d {
                                lists.push(list_kind);
                                init_list_html(list_kind, f)?;
                            }
                            f.write_fmt(format_args!("<li>{}</li>", text_to_htlm(text)))?;
                        }
                        d if d < 0 => {
                            for _ in 0..d.abs() {
                                let pre_list = lists.pop().unwrap();
                                end_list_html(pre_list, f)?;
                            }
                            f.write_fmt(format_args!("<li>{}</li>", text_to_htlm(text)))?;
                        }
                        _ => unreachable!(),
                    }
                }
                Node::LineBreak => f.write_str("<br>")?,
                Node::Rule => f.write_str("<hr>")?,
            }

            if reset_list {
                for pre_list in lists.drain(..).rev() {
                    end_list_html(pre_list, f)?;
                }
            }
        }

        Ok(())
    }
}

fn init_list_html(list_kind: &ListKind, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    match list_kind {
        ListKind::Oredred(_) => {
            f.write_str("<ol>")?;
        }
        ListKind::Unordere(_) => {
            f.write_str("<ul>")?;
        }
    };
    Ok(())
}

fn end_list_html(list_kind: &ListKind, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    match list_kind {
        ListKind::Oredred(_) => {
            f.write_str("</ol>")?;
        }
        ListKind::Unordere(_) => {
            f.write_str("</ul>")?;
        }
    };
    Ok(())
}

fn text_to_htlm(text: &Text) -> String {
    text.content
        .iter()
        .filter_map(|frag| match frag {
            TextFragment::Stylised(styles, text) if !styles.contains(Style::Modifier) => {
                Some(apply_styles_html(text, *styles))
            }
            TextFragment::Link(alt, link) => Some(format!("<a href=\"{link}\">{alt}</a>")),
            TextFragment::Image(alt, src) => Some(format!("<img src=\"{src}\" alt=\"{alt}\">")),
            _ => None::<String>,
        })
        .collect()
}

fn apply_styles_html(text: &str, styles: Style) -> String {
    let mut text = text.to_owned();
    if styles.contains(Style::Strong) {
        text = format!("<strong>{text}</strong>")
    }
    if styles.contains(Style::Emphasis) {
        text = format!("<em>{text}</em>")
    }
    if styles.contains(Style::Code) {
        text = format!("<code>{text}</code>")
    }
    if styles.contains(Style::Strikethrough) {
        text = format!("<s>{text}</s>")
    }

    text
}
