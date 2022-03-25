mod node_dispatch;
pub use node_dispatch::SmallNode;

mod arena;

pub(super) mod node;

mod iter;
pub use iter::{IntoIter, Iter, IterMut};

mod error;
pub use error::SgError;

#[allow(clippy::module_inception)]
mod tree;
pub use tree::{Idx, SgTree};
