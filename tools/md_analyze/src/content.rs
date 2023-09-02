use crate::traits::GetChapter;

use std::path::PathBuf;

/// Displayable content data model
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Content {
    /// An individual X.Y book section or chapter intro
    Section {
        /// Section path
        path: PathBuf,
        /// Section word count
        word_count: usize,
        /// Section data (optionally collected)
        lines: Option<Vec<String>>,
    },
    /// An individual diagram
    Svg {
        /// Diagram path
        path: PathBuf,
    },
}

impl GetChapter for Content {
    fn get_chp(&self) -> Option<usize> {
        match self {
            Self::Section { path, .. } => path.get_chp(),
            Self::Svg { path } => path.get_chp(),
        }
    }
}
