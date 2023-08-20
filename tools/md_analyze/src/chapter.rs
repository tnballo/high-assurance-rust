use crate::{content::Content, traits::GetMetrics, WORDS_PER_PAGE};

use std::{ffi::OsStr, fmt};

use colored::*;
use separator::Separatable;

/// Displayable chapter data model
pub struct Chapter {
    /// Chapter contents
    pub contents: Vec<Content>,
    pub(crate) number: usize,
}

impl GetMetrics for Chapter {
    fn get_word_count(&self) -> usize {
        self.contents
            .iter()
            .map(|c| match c {
                Content::Section { word_count, .. } => word_count,
                Content::Svg { .. } => &0,
            })
            .sum()
    }

    fn get_diagram_count(&self) -> usize {
        self.contents
            .iter()
            .map(|c| match c {
                Content::Section { .. } => 0,
                Content::Svg { .. } => 1,
            })
            .sum()
    }
}

impl fmt::Display for Chapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let word_count = self.get_word_count();
        writeln!(
            f,
            "{}{} {} words ({} pages), {} diagrams",
            match self.number {
                0 => "(frontmatter):".yellow(),
                _ => "chp ".yellow(),
            },
            match self.number {
                0 => "".yellow(),
                _ => (self.number.to_string() + ":").yellow(),
            },
            word_count.separated_string().bright_green(),
            (word_count / WORDS_PER_PAGE)
                .separated_string()
                .bright_cyan(),
            self.get_diagram_count().separated_string().bright_blue()
        )?;

        for content in self.contents.iter() {
            match content {
                Content::Section {
                    path, word_count, ..
                } => {
                    if let Some(file_name) = path.as_path().file_name().and_then(OsStr::to_str) {
                        writeln!(
                            f,
                            " - {}: {}",
                            file_name.bright_magenta(),
                            word_count.separated_string().bright_green()
                        )?;
                    }
                }
                Content::Svg { .. } => continue,
            }
        }

        Ok(())
    }
}
