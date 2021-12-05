use colored::*;


const BANNER_CHAR: &str = "-";
const BANNER_H_PADDING: usize = 3;


/// Prints a banner with the given title.
pub fn flex_banner(text: impl ToString) -> String {
    let text = text.to_string();
    let width = text.len() + 2 + BANNER_H_PADDING * 2;
    let banner = BANNER_CHAR.repeat(width);
    let padding = BANNER_CHAR.repeat(BANNER_H_PADDING);

    format!("{0}\n{1} {2} {1}\n{0}", banner, padding, text)
}

pub fn command(text: &str) -> colored::ColoredString {
    format!("$ {}", text).cyan()
}

pub fn error(text: &str) -> colored::ColoredString {
    format!("{}", text).red()
}