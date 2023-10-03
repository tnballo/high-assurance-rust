use std::path::{Component, PathBuf};

/// Get model metrics
pub trait GetMetrics {
    fn get_word_count(&self) -> usize;
    fn get_diagram_count(&self) -> usize;
}

/// Get chapter number
pub trait GetChapter {
    fn get_chp(&self) -> Option<usize>;
}

impl GetChapter for &PathBuf {
    fn get_chp(&self) -> Option<usize> {
        const CHP_PREFIX: &str = "chp";
        const OPT_CHP_SUFFIX: &str = "_appendix";

        // Non-ambiguity in chp labels (1 per path)
        debug_assert!(
            2 > self
                .components()
                .filter(|c| {
                    match c {
                        Component::Normal(name) => name
                            .to_str()
                            .map_or(false, |n| n.strip_prefix(CHP_PREFIX).is_some()),
                        _ => false,
                    }
                })
                .count(),
            "Ambiguous chapter for path: {}",
            &self.to_str().unwrap(),
        );

        for component in self.components().rev() {
            match component {
                // No chapter number
                Component::RootDir | Component::Prefix(_) => return None,
                // Cannot determine chapter number
                Component::CurDir | Component::ParentDir => continue,
                // Extract chapter number
                Component::Normal(name) => match name.to_str() {
                    Some(name) => match name.strip_prefix(CHP_PREFIX) {
                        Some(number) => {
                            let number = match number.strip_suffix(OPT_CHP_SUFFIX) {
                                Some(number) => number,
                                None => number,
                            };

                            if let Ok(number) = number.parse() {
                                return Some(number);
                            }
                        }
                        None => continue,
                    },
                    None => continue,
                },
            }
        }

        // Other (non-chapter) files
        Some(0)
    }
}
