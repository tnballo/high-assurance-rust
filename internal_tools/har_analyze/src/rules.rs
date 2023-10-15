//! Rules the linter can apply to section data.

use crate::LintError;
use std::{fmt, path::PathBuf};

/// The signature for a rule, addable to linter builder
#[allow(clippy::type_complexity)]
pub struct Rule<'a>(pub &'a dyn Fn(&'a PathBuf, &[String]) -> Result<(), LintError<'a>>);

impl<'a> fmt::Debug for Rule<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rule: {:p}", self.0)
    }
}

/// Section is non-empty
pub fn rule_nonempty<'a>(path: &'a PathBuf, lines: &[String]) -> Result<(), LintError<'a>> {
    match lines.is_empty() {
        true => Err(LintError::Failed {
            path,
            line_number: 0.into(),
            line: "N/A".to_string(),
            reason: "Missing data/contents".to_string(),
        }),
        false => Ok(()),
    }
}

/// Section doesn't contain a draft filesystem path
pub fn rule_no_draft_path<'a>(path: &'a PathBuf, lines: &[String]) -> Result<(), LintError<'a>> {
    for (idx, line) in lines.iter().enumerate() {
        if line.contains("/book-draft/") {
            return Err(LintError::Failed {
                path,
                line_number: idx.into(),
                line: line.clone(),
                reason: "Contains book draft path".to_string(),
            });
        }
    }

    Ok(())
}

/// Section contains 1+ SVGs
pub fn rule_has_svg<'a>(path: &'a PathBuf, lines: &[String]) -> Result<(), LintError<'a>> {
    let p_start_idxs: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.to_lowercase().contains("<p align=\"center\">"))
        .map(|(i, _)| i)
        .collect();

    let p_end_idxs: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.to_lowercase().contains("</p>"))
        .map(|(i, _)| i)
        .collect();

    if p_start_idxs.len() != p_end_idxs.len() {
        return Err(LintError::Failed {
            path,
            line_number: 0.into(),
            line: "N/A".to_string(),
            reason: "Unbalanced paragraph start/end tags".to_string(),
        });
    }

    for (p_start, p_end) in p_start_idxs.into_iter().zip(p_end_idxs.into_iter()) {
        if p_end < p_start {
            return Err(LintError::Failed {
                path,
                line_number: p_end.into(),
                line: lines[p_end].to_string(),
                reason: "Paragraph end tag before start tag".to_string(),
            });
        } else {
            let paragraph = lines[p_start..=p_end].join("\n");
            let html_fragment = scraper::Html::parse_fragment(&paragraph);
            let img_selector = scraper::Selector::parse("img").unwrap();

            for img in html_fragment.select(&img_selector) {
                if let Some(src) = img.value().attr("src") {
                    if src.ends_with(".svg") {
                        return Ok(());
                    }
                }
            }
        }
    }

    Err(LintError::Failed {
        path,
        line_number: 0.into(),
        line: "N/A".to_string(),
        reason: "No centered paragraph SVGs found".to_string(),
    })
}

/// Section has only footer
pub fn rule_footer<'a>(path: &'a PathBuf, lines: &[String]) -> Result<(), LintError<'a>> {
    let sep_idxs: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.trim().eq("---"))
        .map(|(i, _)| i)
        .collect();

    match sep_idxs.len() {
        0 => Err(LintError::Failed {
            path,
            line_number: 0.into(),
            line: "N/A".to_string(),
            reason: "Missing header/footer separator".to_string(),
        }),
        1 => {
            if !lines[sep_idxs[0]..]
                .iter()
                .any(|l| l.starts_with("[^") && l.contains("]:"))
            {
                Err(LintError::Failed {
                    path,
                    line_number: sep_idxs[0].into(),
                    line: lines[sep_idxs[0]].clone(),
                    reason: "Footer must contain at least one footnote".to_string(),
                })
            } else {
                Ok(())
            }
        }
        _ => Err(LintError::Failed {
            path,
            line_number: sep_idxs[1].into(),
            line: lines[sep_idxs[1]].clone(),
            reason: "Cannot have 2+ header/footer separators in a section".to_string(),
        }),
    }
}

/// Section has a header and footer
pub fn rule_header_and_footer<'a>(
    path: &'a PathBuf,
    lines: &[String],
) -> Result<(), LintError<'a>> {
    let sep_idxs: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.trim().eq("---"))
        .map(|(i, _)| i)
        .collect();

    match sep_idxs.len() {
        0 => Err(LintError::Failed {
            path,
            line_number: 0.into(),
            line: "N/A".to_string(),
            reason: "Missing header/footer separator".to_string(),
        }),
        1 => Err(LintError::Failed {
            path,
            line_number: sep_idxs[0].into(),
            line: lines[sep_idxs[0]].clone(),
            reason: "Sole separator - missing header or footer".to_string(),
        }),
        2 => {
            if !lines[sep_idxs[1]..]
                .iter()
                .any(|l| l.starts_with("[^") && l.contains("]:"))
            {
                Err(LintError::Failed {
                    path,
                    line_number: sep_idxs[1].into(),
                    line: lines[sep_idxs[1]].clone(),
                    reason: "Footer must contain at least one footnote".to_string(),
                })
            } else {
                Ok(())
            }
        }
        _ => Err(LintError::Failed {
            path,
            line_number: 0.into(),
            line: "N/A".to_string(),
            reason: "Cannot have 3+ headers in a section".to_string(),
        }),
    }
}

/// Section uses correct heading sizes
pub fn rule_heading_sizes<'a>(path: &'a PathBuf, lines: &[String]) -> Result<(), LintError<'a>> {
    #[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
    enum HeadingState {
        Uninit = 0,
        H1 = 1,
        H2 = 2,
        H3 = 3,
        H4 = 4,
        H5 = 5,
        H6 = 6,
    }

    // Rules:
    // 1. Can only go down 1 level at a time (e.g. H3 -> H4)
    // 2. Can stay at current level (e.g. H3 -> H3)
    // 3. Can go up to any higher level (e.g. H3 -> H2 | H1)
    impl HeadingState {
        pub fn next<'a>(
            self,
            path: &'a PathBuf,
            line_number: usize,
            line: &String,
        ) -> Result<Self, LintError<'a>> {
            assert_ne!(self, HeadingState::H5);
            assert_ne!(self, HeadingState::H6);
            match line {
                line if line.starts_with("# ") => Ok(Self::H1),
                line if line.starts_with("## ") => {
                    if matches!(self, Self::H1 | Self::H2) || self > Self::H2 {
                        Ok(Self::H2)
                    } else {
                        Err(LintError::Failed {
                            path,
                            line_number: line_number.into(),
                            line: line.to_string(),
                            reason: "H2 not preceded by H1, H2, or smaller".to_string(),
                        })
                    }
                }
                line if line.starts_with("### ") => {
                    if matches!(self, Self::H2 | Self::H3) || self > Self::H3 {
                        Ok(Self::H3)
                    } else {
                        Err(LintError::Failed {
                            path,
                            line_number: line_number.into(),
                            line: line.to_string(),
                            reason: "H3 not preceded by H2, H3, or smaller".to_string(),
                        })
                    }
                }
                line if line.starts_with("#### ") => {
                    if matches!(self, Self::H3 | Self::H4) || self > Self::H4 {
                        Ok(Self::H4)
                    } else {
                        Err(LintError::Failed {
                            path,
                            line_number: line_number.into(),
                            line: line.to_string(),
                            reason: "H4 not preceded by H3, H4, or smaller".to_string(),
                        })
                    }
                }
                line if line.starts_with("##### ") => Err(LintError::Failed {
                    path,
                    line_number: line_number.into(),
                    line: line.to_string(),
                    reason: "H5 should not be used".to_string(),
                }),
                line if line.starts_with("###### ") => Err(LintError::Failed {
                    path,
                    line_number: line_number.into(),
                    line: line.to_string(),
                    reason: "H6 should not be used".to_string(),
                }),
                _ => Ok(self),
            }
        }
    }

    // Some content
    let Some(first_effective) = lines
        .iter()
        .position(|l| !l.is_empty() && !l.starts_with("<meta"))
    else {
        return Err(LintError::Failed {
            path,
            line_number: 0.into(),
            line: "N/A".to_string(),
            reason: "Section must be non-empty".to_string(),
        });
    };

    // Title after meta tags
    if !lines[first_effective].starts_with("# ") {
        return Err(LintError::Failed {
            path,
            line_number: first_effective.into(),
            line: lines[first_effective].to_string(),
            reason: "Section must start with an H1 heading".to_string(),
        });
    }

    // Chapter intros end with `## Learning Outcomes`
    if path.file_name() == Some(std::ffi::OsStr::new("_index.md")) {
        match lines.iter().rposition(|l| l.eq("## Learning Outcomes")) {
            Some(outcomes_start) => {
                if lines[outcomes_start..]
                    .iter()
                    .filter(|l| l.starts_with("* "))
                    .count()
                    < 3
                {
                    return Err(LintError::Failed {
                        path,
                        line_number: outcomes_start.into(),
                        line: lines[outcomes_start].clone(),
                        reason: "Chapter intro \"## Learning Outcomes\" must contain at least 3 outcomes".to_string(),
                    });
                }
            }
            None => {
                return Err(LintError::Failed {
                    path,
                    line_number: 0.into(),
                    line: "N/A".to_string(),
                    reason: "Chapter intro missing \"## Learning Outcomes\"".to_string(),
                });
            }
        }
    }

    // Obeys state machine heading rules
    let mut heading_state = HeadingState::Uninit;
    for (line_number, line) in lines.iter().enumerate() {
        heading_state = heading_state.next(path, line_number + 1, line)?;
    }

    Ok(())
}

/// Section contains meta tags
pub fn rule_meta_tags<'a>(path: &'a PathBuf, lines: &[String]) -> Result<(), LintError<'a>> {
    for tag in crate::update::META_TAGS {
        if !lines.iter().any(|l| l.starts_with(tag)) {
            return Err(LintError::Failed {
                path,
                line_number: 0.into(),
                line: "N/A".to_string(),
                reason: format!("Section missing meta tag {}", tag),
            });
        }
    }

    Ok(())
}

/// File has MD extension
pub fn rule_md_extension<'a>(path: &'a PathBuf, _: &[String]) -> Result<(), LintError<'a>> {
    if let Some(file_name) = path.as_path().file_name() {
        let file_name = file_name.to_str().unwrap();
        if !file_name.ends_with(".md") {
            return Err(LintError::Failed {
                path,
                line_number: 0.into(),
                line: "N/A".to_string(),
                reason: format!("Unexpected file extension \"{}\"", file_name),
            });
        }
    }

    Ok(())
}

/// Valid SVG file
pub fn rule_valid_svg<'a>(path: &'a PathBuf, lines: &[String]) -> Result<(), LintError<'a>> {
    if let Some(file_name) = path.as_path().file_name() {
        let file_name = file_name.to_str().unwrap().to_lowercase();
        if !file_name.ends_with(".svg") {
            return Err(LintError::Failed {
                path,
                line_number: 0.into(),
                line: "N/A".to_string(),
                reason: format!("Unexpected file extension \"{}\"", file_name),
            });
        }
    }

    let data = lines.join("\n");
    let Ok(svg) = svg::read(&data) else {
        return Err(LintError::Failed {
            path,
            line_number: 0.into(),
            line: "N/A".to_string(),
            reason: "Unexpected error - svg crate failed to read &str".to_string(),
        });
    };

    for event in svg {
        match event {
            // Valid SVG
            svg::parser::Event::Error(e) => {
                return Err(LintError::Failed {
                    path,
                    line_number: 0.into(),
                    line: "N/A".to_string(),
                    reason: format!("svg parse error: {}", e),
                })
            }
            // No JS
            svg::parser::Event::Tag(tag, ..) => {
                if tag.to_lowercase() == "script" {
                    return Err(LintError::Failed {
                        path,
                        line_number: 0.into(),
                        line: "N/A".to_string(),
                        reason: format!("svg contains JavaScript: {:?}", event),
                    });
                }
            }
            _ => continue,
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn test_valid_svg() {
        use super::rule_valid_svg;

        const SVG: &'static str = include_str!("../../../src/chp1/bugs_venn.svg");
        let path = PathBuf::from("../../../src/chp1/bugs_venn.svg");
        let lines: Vec<_> = SVG.lines().map(|l| l.to_string()).collect();

        assert!(rule_valid_svg(&path, &lines).is_ok());
    }

    #[test]
    fn test_invalid_svg_with_js() {
        use super::rule_valid_svg;

        const SVG: &'static str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
            <path d="M50,3l12,36h38l-30,22l11,36l-31-21l-31,21l11-36l-30-22h38z"
            fill="#FF0" stroke="#FC0" stroke-width="2"/>
            <script xlink:href="external.js" />
            </svg>"##;
        let path = PathBuf::from("/test/path/to/file.svg");
        let lines: Vec<_> = SVG.lines().map(|l| l.to_string()).collect();

        assert!(rule_valid_svg(&path, &lines).is_err());
    }

    #[test]
    fn test_valid_headings() {
        use super::{rule_heading_sizes, rule_md_extension};

        const MD: &'static str = r#######"
            # Heading 1
            text
            ## Heading 2
            text
            ### Heading 3
            text
            ## Back to Heading 2
            text
            "#######;

        let path = PathBuf::from("/test/path/to/file.md");
        let lines: Vec<_> = MD.lines().map(|l| l.trim().to_string()).collect();

        assert!(rule_md_extension(&path, &lines).is_ok());
        assert!(rule_heading_sizes(&path, &lines).is_ok());
    }

    #[test]
    fn test_invalid_headings() {
        use super::{rule_heading_sizes, rule_md_extension};

        const MD: &'static str = r#######"
            # Heading 1
            text
            ### Heading 3 (Invalid)
            text
            ### Heading 3
            text
            ## Back to Heading 2
            text
            "#######;

        let path = PathBuf::from("/test/path/to/file.md");
        let lines: Vec<_> = MD.lines().map(|l| l.trim().to_string()).collect();

        assert!(rule_md_extension(&path, &lines).is_ok());
        assert!(rule_heading_sizes(&path, &lines).is_err());
    }
}
