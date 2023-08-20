use crate::{
    chapter::Chapter,
    content::Content,
    traits::{GetChapter, GetMetrics},
    BOOK_SRC_DIR, WORDS_PER_PAGE,
};

use std::{
    cmp::Reverse,
    collections::BTreeMap,
    ffi::OsStr,
    fmt,
    fs::File,
    io::{prelude::*, BufReader},
};

use anyhow::Result;
use colored::*;
use rayon::prelude::*;
use regex::Regex;
use separator::Separatable;
use walkdir::WalkDir;

/// Displayable book data model
pub struct Book {
    /// Chapters by number
    pub chapters: BTreeMap<usize, Chapter>,
}

impl GetMetrics for Book {
    fn get_word_count(&self) -> usize {
        self.chapters.values().map(|c| c.get_word_count()).sum()
    }

    fn get_diagram_count(&self) -> usize {
        self.chapters.values().map(|c| c.get_diagram_count()).sum()
    }
}

impl Book {
    /// Construct a book data model
    pub fn try_new(collect_section_data: bool) -> Result<Self> {
        let word_regex = Regex::new(r"([a-zA-Z]{1,})")?;
        let contents = Self::collect_contents(collect_section_data, &word_regex);
        let mut chapters = BTreeMap::<usize, Chapter>::new();

        contents.into_iter().for_each(|content| {
            if let Some(number) = content.get_chp() {
                match chapters.get_mut(&number) {
                    Some(chp) => chp.contents.push(content),
                    None => {
                        chapters.insert(
                            number,
                            Chapter {
                                contents: vec![content],
                                number,
                            },
                        );
                    }
                }
            }
        });

        // Sort each chapter's sections contents by word count, descending
        for chp in chapters.values_mut() {
            chp.contents.sort_by_key(|c| {
                Reverse(match c {
                    Content::Section { word_count, .. } => *word_count,
                    Content::Svg { .. } => 0,
                })
            });
        }

        Ok(Book { chapters })
    }

    // Collection book contents
    // Adapted from: https://da-data.blogspot.com/2020/10/no-c-still-isnt-cutting-it.html
    fn collect_contents(collect_section_data: bool, word_regex: &Regex) -> Vec<Content> {
        WalkDir::new(BOOK_SRC_DIR)
            .into_iter()
            .filter_map(Result::ok)
            // Markdown and SVG extension names
            .filter(|dir_ent| {
                matches!(
                    dir_ent.path().extension().and_then(OsStr::to_str),
                    Some("md") | Some("MD") | Some("svg") | Some("SVG")
                )
            })
            // Openable
            .map(|dir_entry| (dir_entry.clone(), File::open(dir_entry.path())))
            .filter_map(|(dir_entry, file)| match file {
                Ok(file) => Some((dir_entry, file)),
                _ => None,
            })
            // Actual files
            .filter(|(_, file)| {
                file.metadata()
                    .map(|meta_data| meta_data.is_file())
                    .unwrap_or(false)
            })
            .par_bridge()
            // Construct content data model
            .map(|(dir_entry, file)| {
                let path = dir_entry.path().to_path_buf();
                let reader = BufReader::new(file);

                match path.extension().and_then(OsStr::to_str) {
                    Some("svg") | Some("SVG") => Content::Svg { path },
                    Some("md") | Some("MD") => {
                        let lines = reader
                            .lines()
                            .map_while(Result::ok)
                            .collect::<Vec<String>>();

                        Content::Section {
                            path,
                            word_count: Self::count_words(&lines, word_regex),
                            lines: if collect_section_data {
                                Some(lines)
                            } else {
                                None
                            },
                        }
                    }
                    _ => unreachable!("File extensions pre-filtered"),
                }
            })
            .collect()
    }

    // Count words in a given file
    fn count_words(lines: &[String], word_regex: &Regex) -> usize {
        lines
            .iter()
            .map(|line| word_regex.captures_iter(line).count())
            .sum()
    }
}

impl fmt::Display for Book {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let word_count = self.get_word_count();

        for chp in self.chapters.values() {
            writeln!(f, "{}", chp)?;
        }

        writeln!(
            f,
            "{}: {} words ({} pages), {} diagrams",
            "BOOK TOTAL".yellow(),
            word_count.separated_string().bright_green(),
            (word_count / WORDS_PER_PAGE)
                .separated_string()
                .bright_cyan(),
            self.get_diagram_count().separated_string().bright_blue(),
        )
    }
}
