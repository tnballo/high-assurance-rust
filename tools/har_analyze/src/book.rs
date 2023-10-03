use crate::{
    chapter::Chapter,
    content::Content,
    lint::{Level, Linter, LinterBuilder},
    rules::*,
    traits::{GetChapter, GetMetrics},
    BOOK_SRC_DIR, WORDS_PER_PAGE,
};

use std::{
    cmp::Reverse,
    collections::BTreeMap,
    error::Error,
    ffi::OsStr,
    fmt,
    fs::File,
    io::{prelude::*, BufReader},
};

use colored::*;
use rayon::prelude::*;
use regex::Regex;
use separator::Separatable;
use walkdir::WalkDir;

const NON_CHP_NUM: usize = 0;
const APPENDIX_CHP_NUM: usize = 16;

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
    pub fn try_new(collect_section_data: bool) -> Result<Self, Box<dyn Error>> {
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

    /// Get a linter for frontmatter that doesn't belong to any chapter
    pub fn get_non_chp_linter(&self) -> Linter<'_> {
        let mut linter = LinterBuilder::new()
            .add_rule(Level::Fatal, Rule(&rule_md_extension))
            .add_rule(Level::Fatal, Rule(&rule_nonempty));

        for (num, chp) in self.chapters.iter() {
            if *num == NON_CHP_NUM {
                for content in chp.contents.iter() {
                    if matches!(content, Content::Section { .. }) {
                        linter = linter.add_content(content);
                    }
                }
            }
        }

        linter.build()
    }

    /// Get a linter for chp intros
    pub fn get_chp_intro_linter(&self) -> Linter<'_> {
        let mut linter = LinterBuilder::new()
            .add_rule(Level::Fatal, Rule(&rule_md_extension))
            .add_rule(Level::Fatal, Rule(&rule_nonempty))
            .add_rule(Level::Fatal, Rule(&rule_header_and_footer))
            .add_rule(Level::Fatal, Rule(&rule_heading_sizes))
            .add_rule(Level::Fatal, Rule(&rule_meta_tags))
            .add_rule(Level::Warning, Rule(&rule_has_svg));

        for (num, chp) in self.chapters.iter() {
            if *num != NON_CHP_NUM && *num != APPENDIX_CHP_NUM {
                for content in chp.contents.iter() {
                    if matches!(content, Content::Section { .. }) {
                        if let Some(file_name) = content.get_path().as_path().file_name() {
                            if file_name.eq_ignore_ascii_case("_index.md") {
                                linter = linter.add_content(content);
                            }
                        }
                    }
                }
            }
        }

        linter.build()
    }

    /// Get a linter for chp non-intro sections
    pub fn get_chp_sections_linter(&self) -> Linter<'_> {
        let mut linter = LinterBuilder::new()
            .add_rule(Level::Fatal, Rule(&rule_md_extension))
            .add_rule(Level::Fatal, Rule(&rule_nonempty))
            .add_rule(Level::Fatal, Rule(&rule_footer))
            .add_rule(Level::Fatal, Rule(&rule_heading_sizes));

        for (num, chp) in self.chapters.iter() {
            if *num != NON_CHP_NUM {
                for content in chp.contents.iter() {
                    if matches!(content, Content::Section { .. }) {
                        if let Some(file_name) = content.get_path().as_path().file_name() {
                            if !file_name.eq_ignore_ascii_case("_index.md")
                                && !file_name.eq_ignore_ascii_case("tools.md")
                                && !file_name.eq_ignore_ascii_case("resources.md")
                                && !file_name.eq_ignore_ascii_case("books.md")
                                && !file_name.to_str().unwrap().ends_with("PLACEHOLDER.md")
                            {
                                linter = linter.add_content(content);
                            }
                        }
                    }
                }
            }
        }

        linter.build()
    }

    /// Get a linter for SVG files
    pub fn get_svg_linter(&self) -> Linter<'_> {
        let mut linter = LinterBuilder::new()
            .add_rule(Level::Fatal, Rule(&rule_nonempty))
            .add_rule(Level::Fatal, Rule(&rule_valid_svg));

        for (_, chp) in self.chapters.iter() {
            for content in chp.contents.iter() {
                if matches!(content, Content::Svg { .. }) {
                    linter = linter.add_content(content);
                }
            }
        }

        linter.build()
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
                let lines = reader
                    .lines()
                    .map_while(Result::ok)
                    .collect::<Vec<String>>();

                match path.extension().and_then(OsStr::to_str) {
                    Some("svg") | Some("SVG") => Content::Svg {
                        path,
                        lines: if collect_section_data {
                            Some(lines)
                        } else {
                            None
                        },
                    },
                    Some("md") | Some("MD") => Content::Section {
                        path,
                        word_count: Self::count_words(&lines, word_regex),
                        lines: if collect_section_data {
                            Some(lines)
                        } else {
                            None
                        },
                    },
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
