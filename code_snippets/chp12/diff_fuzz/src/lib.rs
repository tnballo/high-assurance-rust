/*!
Buggy version of the [`scapegoat` crate](https://docs.rs/scapegoat/latest/scapegoat/), for differential fuzzing blog post.
*/

#![forbid(unsafe_code)]
#![cfg_attr(not(any(test, fuzzing)), no_std)]
#![cfg_attr(not(any(test, fuzzing)), deny(missing_docs))]

mod tree;
pub use crate::tree::SgError;

mod map;
pub use crate::map::SgMap;

/// [`SgMap`][crate::map::SgMap]'s iterator return types and [`Entry`](crate::map_types::Entry) enum.
pub mod map_types;

mod set;
pub use crate::set::SgSet;

/// [`SgSet`][crate::set::SgSet]'s iterator return types.
pub mod set_types;
