// utilities for displaying elphy
use pad::PadStr;

pub fn ascii_reversed(art: &str) -> String {
    let lines = art.split('\n').collect::<Vec<&str>>();
    let width = lines
        .iter()
        .max_by(|&&x, &&y| x.len().cmp(&y.len()))
        .unwrap()
        .len();

    lines
        .into_iter()
        .map(|s| s.pad_to_width(width).chars().rev().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n")
}
