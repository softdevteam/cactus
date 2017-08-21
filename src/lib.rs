// Copyright (c) 2017 King's College London
// created by the Software Development Team <http://soft-dev.org/>
//
// The Universal Permissive License (UPL), Version 1.0
//
// Subject to the condition set forth below, permission is hereby granted to any person obtaining a
// copy of this software, associated documentation and/or data (collectively the "Software"), free
// of charge and under any and all copyright rights in the Software, and any and all patent rights
// owned or freely licensable by each licensor hereunder covering either (i) the unmodified
// Software as contributed to or provided by such licensor, or (ii) the Larger Works (as defined
// below), to deal in both
//
// (a) the Software, and
// (b) any piece of software and/or hardware listed in the lrgrwrks.txt file
// if one is included with the Software (each a "Larger Work" to which the Software is contributed
// by such licensors),
//
// without restriction, including without limitation the rights to copy, create derivative works
// of, display, perform, and distribute the Software and make, use, sell, offer for sale, import,
// export, have made, and have sold the Software and the Larger Work(s), and to sublicense the
// foregoing rights on either these or other terms.
//
// This license is subject to the following condition: The above copyright notice and either this
// complete permission notice or at a minimum a reference to the UPL must be included in all copies
// or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
// BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
// DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

//! An immutable cactus stuck (also called a spaghetti stack or parent pointer tree). A cactus
//! stack is a (possibly empty) node with a (possibly null) pointer to a parent node. Any given
//! node has a unique path back to the root node. Rather than mutably updating the stack, one
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
            node: Some(Rc::new(Node{val, parent: self.node.clone()}))
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

    /// Consume this Cactus node and return its data. If the cactus node has no children, its data
    /// is returned without cloning; if the node has children then the internal data is cloned
    /// (hence why `T` must implement the `Clone` trait).
    ///
    /// # Examples
    /// ```
    /// use cactus::Cactus;
    /// let c = Cactus::new().child(1).child(2);
    /// let p = c.parent().unwrap();
    /// assert_eq!(c.take_or_clone_val(), Some(2));
    /// // At this point c has been consumed and can no longer be referenced.
    /// assert_eq!(p.val(), Some(&1));
    /// ```
    pub fn take_or_clone_val(self) -> Option<T> where T: Clone {
        self.node.map(|x| match Rc::try_unwrap(x) {
            Ok(n) => n.val,
            Err(c) => c.val.clone()
        })
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
        if self.len() != other.len() {
            return false;
        }
        self.vals()
            .zip(other.vals())
            .all(|(x, y)| x == y)
    }
}

impl<T: Eq> Eq for Cactus<T> {}

impl<T: fmt::Debug> fmt::Debug for Cactus<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Cactus["));
        for (i, x) in self.vals().enumerate() {
            if i > 0 {
                try!(write!(f, ", "));
            }
            try!(write!(f, "{:?}", x));
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
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
    fn test_take_or_clone_val() {
        let c = Cactus::new().child(4).child(3);
        let c1 = c.child(2);
        let c2 = c.child(1);
        assert_eq!(c2.take_or_clone_val(), Some(1));
        assert_eq!(c.take_or_clone_val(), Some(3));
        assert_eq!(c1.take_or_clone_val(), Some(2));
    }
}
