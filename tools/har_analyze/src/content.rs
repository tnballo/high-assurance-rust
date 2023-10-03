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
        /// Section data (optionally collected, line-oriented)
        lines: Option<Vec<String>>,
        /// Section word count
        word_count: usize,
    },
    /// An individual diagram
    Svg {
        /// Diagram path
        path: PathBuf,
        /// SVG data (optionally collected, line-oriented)
        lines: Option<Vec<String>>,
    },
}

impl Content {
    /// Get file path for this content
    pub fn get_path(&self) -> &PathBuf {
        match self {
            Self::Section { path, .. } => path,
            Self::Svg { path, .. } => path,
        }
    }
}

impl GetChapter for Content {
    fn get_chp(&self) -> Option<usize> {
        match self {
            Self::Section { path, .. } => path.get_chp(),
            Self::Svg { path, .. } => path.get_chp(),
        }
    }
}
