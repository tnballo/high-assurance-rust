//! MD Book metrics/linting.
//! Internal tool for <https://highassurance.rs>
#![deny(missing_docs)]

mod book;
pub use book::*;

mod chapter;
pub use chapter::*;

mod content;
pub use content::*;

#[allow(missing_docs)]
mod lint;
pub use lint::*;

mod badge;
pub use badge::*;

mod rules;

mod traits;

pub(crate) const BOOK_SRC_DIR_RELATIVE: &str = "../../src";
pub(crate) const WORDS_PER_PAGE: usize = 500;
