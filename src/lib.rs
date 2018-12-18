// Copyright (c) 2018 King's College London created by the Software Development Team
// <http://soft-dev.org/>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, or the UPL-1.0 license <http://opensource.org/licenses/UPL>
// at your option. This file may not be copied, modified, or distributed except according to those
// terms.

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

use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// An immutable cactus stack node. May be empty or contain a value; may have a pointer to a parent
/// or not.
#[derive(Clone, Default)]
pub struct Cactus<T> {
    node: Option<Rc<Node<T>>>
}

#[derive(Clone)]
struct Node<T> {
    val: T,
    parent: Option<Rc<Node<T>>>
}

impl<T> Cactus<T> {
    /// Return an empty cactus stack node.
    pub fn new() -> Cactus<T> {
        Cactus{node: None}
    }

    /// Is this cactus stack node empty?
    ///
    /// # Examples
    /// ```
    /// use cactus::Cactus;
    /// let c = Cactus::new();
    /// assert!(c.is_empty());
    /// let c2 = c.child(1);
    /// assert!(!c2.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.node.is_none()
    }

    /// How many items are there in this cactus stack?
    pub fn len(&self) -> usize {
        self.vals().count()
    }

    /// Create a new cactus stack node containing value `val` and pointing to parent `self`.
    ///
    /// # Examples
    /// ```
    /// use cactus::Cactus;
    /// let c = Cactus::new();
    /// let c2 = c.child(1);
    /// let c3 = c2.child(2);
    /// assert_eq!(c3.vals().cloned().collect::<Vec<_>>(), [2, 1]);
    /// ```
    pub fn child(&self, val: T) -> Cactus<T> {
        Cactus {
            node: Some(Rc::new(Node{val,
                                    parent: self.node.clone()
                                   }))
        }
    }

    /// Return this cactus stack node's parent node or `None` if this cactus stack is empty.
    ///
    /// # Examples
    /// ```
    /// use cactus::Cactus;
    /// let c = Cactus::new();
    /// let c2 = c.child(1);
    /// assert_eq!(c.parent(), None);
    /// assert_eq!(c2.val(), Some(&1));
    /// assert_eq!(c2.parent().unwrap(), Cactus::new());
    /// ```
    pub fn parent(&self) -> Option<Cactus<T>> {
        self.node.as_ref()
                 .map(|n| Cactus{node: n.parent.clone()} )
    }

    /// Return a reference to this cactus stack node's value or `None` if this cactus stack is
    /// empty.
    ///
    /// # Examples
    /// ```
    /// use cactus::Cactus;
    /// let c = Cactus::new().child(1);
    /// assert_eq!(c.val(), Some(&1));
    /// assert_eq!(c.parent().unwrap().val(), None);
    /// ```
    pub fn val(&self) -> Option<&T> {
        self.node.as_ref().map(|n| &n.val)
    }

    /// Return an iterator over this cactus stack's nodes. Note that the iterator produces nodes
    /// starting from this node and then walking up towards the root.
    ///
    /// # Examples
    /// ```
    /// use cactus::Cactus;
    /// let c = Cactus::new().child(1).child(2).child(3);
    /// assert_eq!(c.nodes().skip(1).next(), Some(Cactus::new().child(1).child(2)));
    /// ```
    pub fn nodes(&self) -> CactusNodesIter<T> {
        CactusNodesIter{next: self.node.as_ref()}
    }

    /// Return an iterator over this cactus stack's values. Note that the iterator produces values
    /// starting from this node and then walking up towards the root.
    ///
    /// # Examples
    /// ```
    /// use cactus::Cactus;
    /// let c = Cactus::new().child(1).child(2).child(3);
    /// assert_eq!(c.vals().cloned().collect::<Vec<_>>(), [3, 2, 1]);
    /// ```
    pub fn vals(&self) -> CactusValsIter<T> {
        CactusValsIter{next: self.node.as_ref()}
    }

    /// Try to consume this Cactus node and return its data. If the cactus node has no children,
    /// this succeeds; if the cactus node has children, it fails, and returns the original
    /// cactus node.
    ///
    /// # Examples
    /// ```
    /// use cactus::Cactus;
    /// let c = Cactus::new().child(1).child(2);
    /// let p = c.parent().unwrap();
    /// assert_eq!(c.try_unwrap().unwrap(), 2);
    /// // At this point the c variable can no longer be referenced (its value has moved).
    /// assert_eq!(p.val(), Some(&1));
    ///
    /// let d = Cactus::new().child(1);
    /// let d1 = d.child(2);
    /// let d2 = d.child(3);
    /// // At this point d.try_unwrap().unwrap() would return an Err, as d has two children that
    /// // prevent the underlying Cactus from being consumed. We then need to manually clone the
    /// // value if we want to access it uniformly.
    /// assert_eq!(d.try_unwrap().unwrap_or_else(|c| c.val().unwrap().clone()), 1);
    /// // At this point the d variable can no loner be referenced (its value has moved),
    /// // but we can still access the contents it once pointed to:
    /// assert_eq!(*d1.parent().unwrap().val().unwrap(), 1);
    /// ```
    pub fn try_unwrap(self) -> Result<T, Cactus<T>> {
        match self.node {
            None => Err(Cactus{node: None}),
            Some(x) =>  {
                match Rc::try_unwrap(x) {
                    Ok(n) => Ok(n.val),
                    Err(rc) => Err(Cactus{node: Some(rc)})
                }
            }
        }
    }
}

/// An iterator over a `Cactus` stack's nodes. Note that the iterator produces nodes starting
/// from this node and then walking up towards the root.
pub struct CactusNodesIter<'a, T> where T: 'a {
    next: Option<&'a Rc<Node<T>>>
}

impl<'a, T> Iterator for CactusNodesIter<'a, T> {
    type Item = Cactus<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|n| {
            self.next = n.parent.as_ref();
            Cactus{node: Some(n.clone())}
        })
    }
}

/// An iterator over a `Cactus` stack's values. Note that the iterator produces values starting
/// from this node and then walking up towards the root.
pub struct CactusValsIter<'a, T> where T: 'a {
    next: Option<&'a Rc<Node<T>>>
}

impl<'a, T> Iterator for CactusValsIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|n| {
            self.next = n.parent.as_ref();
            &n.val
        })
    }
}

impl<T: PartialEq> PartialEq for Cactus<T> {
    fn eq(&self, other: &Cactus<T>) -> bool {
        // This is, in a sense, a manually expanded self.vals().zip(other.vals()) -- doing so
        // allows us to potentially shortcut the checking of every element using Rc::ptr_eq.
        let mut si = self.node.as_ref();
        let mut oi = other.node.as_ref();
        while si.is_some() && oi.is_some() {
            let sn = si.unwrap();
            let on = oi.unwrap();
            // If we're lucky, the two Rc's are pointer equal, proving that the two cactuses are
            // equal even without ascending the parent hierarchy.
            if Rc::ptr_eq(sn, on) {
                return true;
            }
            if sn.val != on.val {
                return false;
            }
            if let Some(n) = si.take() {
                si = n.parent.as_ref();
            }
            if let Some(n) = oi.take() {
                oi = n.parent.as_ref();
            }
        }
        // If one of the iterators finished before the other, the two cactuses were of different
        // length and thus unequal by definition; otherwise they were equal.
        !(si.is_some() || oi.is_some())
    }
}

impl<T: Eq> Eq for Cactus<T> {}

impl<T: Hash> Hash for Cactus<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for v in self.vals() {
            v.hash(state);
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Cactus<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cactus[")?;
        for (i, x) in self.vals().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", x)?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::collections::hash_map::DefaultHasher;
    use super::*;

    #[test]
    fn test_simple() {
        let r = Cactus::new();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
        assert!(r.val().is_none());
        assert!(r.parent().is_none());
        let r2 = r.child(2);
        assert!(!r2.is_empty());
        assert_eq!(r2.len(), 1);
        assert_eq!(*r2.val().unwrap(), 2);
        let r3 = r2.parent().unwrap();
        assert_eq!(r3.is_empty(), true);
        assert_eq!(r3.len(), 0);
        let r4 = r.child(3);
        assert_eq!(r4.len(), 1);
        assert_eq!(*r4.val().unwrap(), 3);
        let r5 = r4.parent().unwrap();
        assert!(r5.is_empty());
        let r6 = r4.child(4);
        assert_eq!(r6.len(), 2);
        assert_eq!(*r6.val().unwrap(), 4);
        assert_eq!(*r6.parent().unwrap().val().unwrap(), 3);
    }

    #[test]
    fn test_vals() {
        let c = Cactus::new().child(3).child(2).child(1);
        assert_eq!(c.vals().cloned().collect::<Vec<_>>(), [1, 2, 3]);
    }

    #[test]
    fn test_vals_nodes() {
        let c = Cactus::new().child(3).child(2).child(1);
        assert_eq!(c.nodes().skip(1).next().unwrap(), Cactus::new().child(3).child(2));
        assert_eq!(c.nodes().skip(2).next().unwrap(), Cactus::new().child(3));
    }

    #[test]
    fn test_eq() {
        let c1 = Cactus::new().child(1).child(2);
        assert_eq!(c1, c1);
        let c1_1 = c1.child(4);
        let c1_2 = c1.child(4);
        assert_eq!(c1_1, c1_2);
        let c2 = Cactus::new().child(1).child(2);
        assert_eq!(c1, c2);
        assert!(!(c1 != c2));
        let c3 = Cactus::new().child(2).child(2);
        assert_ne!(c1, c3);
        assert!(!(c1 == c3));
    }

    #[test]
    fn test_debug() {
        let c = Cactus::new().child(3).child(2).child(1);
        assert_eq!(format!("{:?}", c), "Cactus[1, 2, 3]");
    }

    #[test]
    fn test_try_unwrap() {
        let c = Cactus::new().child(4).child(3);
        let c1 = c.child(2);
        let c2 = c.child(1);
        assert_eq!(c2.try_unwrap(), Ok(1));
        assert_eq!(c.try_unwrap().unwrap_or_else(|c| c.val().unwrap().clone()), 3);
        assert_eq!(c1.try_unwrap(), Ok(2));
    }

    #[test]
    fn test_hash() {
        fn calculate_hash<T: Hash>(t: &T) -> u64 {
            let mut s = DefaultHasher::new();
            t.hash(&mut s);
            s.finish()
        }

        let c1 = Cactus::new().child(4).child(3);
        let c2 = Cactus::new().child(4).child(3);
        assert_eq!(calculate_hash(&c1), calculate_hash(&c2));
        // The next test is fragile in theory although probably not in practise. Since there's no
        // guarantee that two distinct things will map to distinct hashes, it's perfectly possible
        // that a hasher returns the same value for two distinct cactuses. But this isn't hugely
        // likely to happen and, if it does, it'll be easy to work out what happened.
        let c3 = Cactus::new().child(3).child(4);
        assert_ne!(calculate_hash(&c1), calculate_hash(&c3));

        let mut s = HashSet::new();
        s.insert(c1.clone());
        s.insert(c2.clone());
        assert_eq!(s.len(), 1);
        assert_eq!(*s.iter().nth(0).unwrap(), c1);
        assert_eq!(*s.iter().nth(0).unwrap(), c2);
    }
}
