#![feature(let_chains)]
#![feature(char_indices_offset)]

mod document;
mod html;
mod md;

pub use html::HTML;
pub use md::MarkDown;
