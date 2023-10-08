use std::{
    fs::{File, OpenOptions},
    io::{self, prelude::*, BufReader},
    iter,
    path::PathBuf,
};

use crate::{traits::GetMetrics, Book, BOOK_SRC_DIR_RELATIVE, WORDS_PER_PAGE};

use separator::Separatable;

const BADGE_LINK: &str = "https://github.com/tnballo/high-assurance-rust";
const PAGE_BADGE_START: &str = "[![Pages](https://img.shields.io/badge/Pages";
const DIAGRAM_BADGE_START: &str = "[![Diagrams](https://img.shields.io/badge/Diagrams";

/// Update page/diagram count badges in book `landing.md` and `README.md`
pub fn update_badges(book: &Book) -> io::Result<()> {
    let page_cnt = book.get_word_count() / WORDS_PER_PAGE;
    let diagram_cnt = book.get_diagram_count();
    let landing_path = PathBuf::from(BOOK_SRC_DIR_RELATIVE).join("landing.md");
    let readme_path = PathBuf::from(BOOK_SRC_DIR_RELATIVE)
        .as_path()
        .parent()
        .expect("Failed to find book root path")
        .join("README.md");

    for path in [readme_path, landing_path].into_iter() {
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let content = reader
            .lines()
            .map_while(Result::ok)
            .map(|line| {
                if line.starts_with(PAGE_BADGE_START) {
                    new_page_badge_line(page_cnt)
                } else if line.starts_with(DIAGRAM_BADGE_START) {
                    new_diagram_badge_line(diagram_cnt)
                } else {
                    line
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
            .chars()
            .chain(iter::once('\n'))
            .collect::<String>();

        let mut new_file = OpenOptions::new().write(true).truncate(true).open(path)?;
        new_file.write_all(content.as_bytes())?;
    }

    Ok(())
}

fn new_page_badge_line(page_cnt: usize) -> String {
    format!(
        "{}-{}-purple.svg)]({})",
        PAGE_BADGE_START,
        page_cnt.separated_string(),
        BADGE_LINK
    )
}

fn new_diagram_badge_line(diagram_cnt: usize) -> String {
    format!(
        "{}-{}-blue.svg)]({})",
        DIAGRAM_BADGE_START,
        diagram_cnt.separated_string(),
        BADGE_LINK
    )
}
