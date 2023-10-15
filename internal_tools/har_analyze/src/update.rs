use std::{
    fs::{File, OpenOptions},
    io::{self, prelude::*, BufReader},
    iter,
    path::PathBuf,
};

use crate::{traits::GetMetrics, Book, Content, BOOK_SRC_DIR_RELATIVE, WORDS_PER_PAGE};

use separator::Separatable;

const BADGE_LINK: &str = "https://github.com/tnballo/high-assurance-rust";
const PAGE_BADGE_START: &str = "[![Pages](https://img.shields.io/badge/Pages";
const DIAGRAM_BADGE_START: &str = "[![Diagrams](https://img.shields.io/badge/Diagrams";

pub(crate) const META_TAGS: [&str; 12] = [
    "<meta name=\"title\" content=\"High Assurance Rust\">",
    "<meta name=\"description\" content=\"Developing Secure and Robust Software\">",
    "<meta property=\"og:title\" content=\"High Assurance Rust\">",
    "<meta property=\"og:description\" content=\"Developing Secure and Robust Software\">",
    "<meta property=\"og:type\" content=\"article\">",
    "<meta property=\"og:url\" content=\"https://highassurance.rs/\">",
    "<meta property=\"og:image\" content=\"https://highassurance.rs/img/har_logo_social.png\">",
    "<meta name=\"twitter:title\" content=\"High Assurance Rust\">",
    "<meta name=\"twitter:description\" content=\"Developing Secure and Robust Software\">",
    "<meta name=\"twitter:url\" content=\"https://highassurance.rs/\">",
    "<meta name=\"twitter:card\" content=\"summary_large_image\">",
    "<meta name=\"twitter:image\" content=\"https://highassurance.rs/img/har_logo_social.png\">",
];

// TODO: don't use `BOOK_SRC_DIR_RELATIVE`
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
                    format!(
                        "{}-{}-purple.svg)]({})",
                        PAGE_BADGE_START,
                        page_cnt.separated_string(),
                        BADGE_LINK
                    )
                } else if line.starts_with(DIAGRAM_BADGE_START) {
                    format!(
                        "{}-{}-blue.svg)]({})",
                        DIAGRAM_BADGE_START,
                        diagram_cnt.separated_string(),
                        BADGE_LINK
                    )
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

// TODO: double-check and add to `--update` flag
/// Add meta tags to the start of every single page of the book
pub fn update_meta_tags(book: &Book) -> io::Result<()> {
    for (_, chp) in &book.chapters {
        for content in &chp.contents {
            if let Content::Section { path, lines, .. } = content {
                let file = File::open(&path)?;
                let reader = BufReader::new(file);
                let current_contents = reader
                    .lines()
                    .map_while(Result::ok)
                    .collect::<Vec<String>>();

                // Check that in-memory representation still matches file
                if let Some(mem_lines) = lines {
                    mem_lines.iter().zip(current_contents.iter()).for_each(
                        |(mem_line, file_line)| {
                            debug_assert!(mem_line == file_line);
                        },
                    );
                }

                let new_contents = if starts_with_meta_tags(current_contents.iter()) {
                    current_contents
                } else {
                    let without_tags = remove_meta_tags(current_contents.into_iter());
                    let with_correct_tags = prefix_meta_tags(without_tags);
                    with_correct_tags.into_iter().collect()
                };

                let new_contents = new_contents
                    .join("\n")
                    .chars()
                    .chain(iter::once('\n'))
                    .collect::<String>();

                let mut new_file = OpenOptions::new().write(true).truncate(true).open(path)?;
                new_file.write_all(new_contents.as_bytes())?;
            }
        }
    }

    Ok(())
}

fn starts_with_meta_tags<'a>(lines: impl Iterator<Item = &'a String>) -> bool {
    for (meta_tag_line, actual_line) in META_TAGS.iter().zip(lines) {
        if *meta_tag_line != actual_line {
            return false;
        }
    }

    true
}

fn remove_meta_tags(lines: impl IntoIterator<Item = String>) -> Box<dyn Iterator<Item = String>> {
    Box::new(
        lines
            .into_iter()
            .filter(|l| !META_TAGS.iter().any(|t| *t == l))
            .collect::<Vec<_>>()
            .into_iter(),
    )
}

fn prefix_meta_tags(lines: impl IntoIterator<Item = String>) -> Box<dyn Iterator<Item = String>> {
    Box::new(
        META_TAGS
            .iter()
            .map(|tag| tag.to_string())
            .chain(iter::once("\n".to_string()))
            .chain(lines)
            .collect::<Vec<_>>()
            .into_iter(),
    )
}
