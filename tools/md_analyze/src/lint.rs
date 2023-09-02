use crate::{rules::Rule, Content};
use std::path::PathBuf;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Level {
    Fatal,
    Warning,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum LintError<'a> {
    Failed {
        path: &'a PathBuf,
        line_number: usize,
        line: String,
        reason: String,
    },
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum LeveledLintError<'a> {
    Fatal(LintError<'a>),
    Warning(LintError<'a>),
}

#[cfg(test)]
impl<'a> PartialEq for Rule<'a> {
    fn eq(&self, other: &Self) -> bool {
        // XXX: this is a test-only crime
        format!("{:?}", self) == format!("{:?}", other)
    }
}

#[derive(Default, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Linter<'a> {
    rules: Vec<(Level, Rule<'a>)>,
    contents: Vec<&'a Content>,
}

impl<'a> Linter<'a> {
    pub fn builder() -> LinterBuilder<'a> {
        LinterBuilder::default()
    }

    pub fn run(&self) -> Result<(), LeveledLintError> {
        for content in &self.contents {
            if let Content::Section { lines, path, .. } = content {
                match lines {
                    Some(lines) => {
                        for (level, rule) in &self.rules {
                            rule.0(path, lines).map_err(|err| match level {
                                Level::Fatal => LeveledLintError::Fatal(err),
                                Level::Warning => LeveledLintError::Warning(err),
                            })?;
                        }
                    }
                    None => {
                        return Err(LeveledLintError::Fatal(LintError::Failed {
                            path,
                            line_number: 0,
                            line: "N/A".to_string(),
                            reason: format!("Empty content"),
                        }))
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Default)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct LinterBuilder<'a> {
    rules: Vec<(Level, Rule<'a>)>,
    contents: Vec<&'a Content>,
}

impl<'a> LinterBuilder<'a> {
    pub fn new() -> LinterBuilder<'a> {
        LinterBuilder {
            rules: Vec::new(),
            contents: Vec::new(),
        }
    }

    pub fn add_rule(mut self, level: Level, rule: Rule<'a>) -> LinterBuilder<'a> {
        self.rules.push((level, rule));
        self
    }

    pub fn add_content(mut self, content: &'a Content) -> LinterBuilder<'a> {
        self.contents.push(content);
        self
    }

    pub fn build(self) -> Linter<'a> {
        Linter {
            rules: self.rules,
            contents: self.contents,
        }
    }
}

#[test]
fn test_lint_builder() {
    use crate::rules::rule_nonempty;
    use std::path::PathBuf;

    let empty_section = Content::Section {
        path: PathBuf::from("/test/path/to/file.md"),
        word_count: 0,
        lines: None,
    };

    let default_svg = Content::Svg {
        path: PathBuf::default(),
    };

    let linter = Linter {
        rules: vec![(Level::Fatal, Rule(&rule_nonempty))],
        contents: vec![&default_svg, &empty_section],
    };

    let linter_from_builder: Linter = LinterBuilder::new()
        .add_rule(Level::Fatal, Rule(&rule_nonempty))
        .add_content(&default_svg)
        .add_content(&empty_section)
        .build();

    assert_eq!(linter, linter_from_builder);
    assert!(matches!(linter.run(), Err(LeveledLintError::Fatal(_))));
}
