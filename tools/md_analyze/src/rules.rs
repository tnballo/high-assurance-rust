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
            line_number: 0,
            line: "N/A".to_string(),
            reason: "Missing data/contents".to_string(),
        }),
        false => Ok(()),
    }
}
