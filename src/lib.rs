//! An immutable parent pointer tree -- also called a cactus stack.
//!
//! A cactus stack is a (possibly empty) node with a (possibly null) pointer to a parent node. Any
//! given node has a unique path back to the root node. Rather than mutably updating the stack, one
//! creates and obtains access to immutable nodes (when a node becomes unreachable its memory is
//! automatically reclaimed). A new child node pointing to a parent can be created via the `child`
//! function (analogous to the normal `push`) and a parent can be retrieved via the `parent`
//! function (analogous to the normal `pop`).
//!
//! ```
//! use cactus::Cactus;
//! let c = Cactus::new();
//! assert!(c.is_empty());
//! let c2 = c.child(1);
//! assert_eq!(c2.len(), 1);
//! assert_eq!(*c2.val().unwrap(), 1);
//! let c3 = c2.parent().unwrap();
//! assert!(c3.is_empty());
//! ```
//!
//! From a given node one can create multiple sub-stacks:
//!
//! ```
//! use cactus::Cactus;
//! let c = Cactus::new().child(1);
//! let c2 = c.child(2);
//! let c3 = c.child(3);
//! assert!(c2 != c3);
//! assert_eq!(c2.vals().cloned().collect::<Vec<_>>(), [2, 1]);
//! assert_eq!(c3.vals().cloned().collect::<Vec<_>>(), [3, 1]);
//! ```

mod rc_cactus;

pub use rc_cactus::Cactus;
