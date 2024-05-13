//! Iterators for [`CsfStringtable`].

use std::collections::hash_set::{
    Drain as HashSetDrain, IntoIter as HashSetIntoIter, Iter as HashSetIter,
};

use crate::csf::CsfLabel;

/// An iterator over [`CsfStringtable`][csf] labels. This struct can be created
/// by [`iter`][iter] method of a CsfStringtable.
///
/// [csf]: crate::csf::CsfStringtable
/// [iter]: crate::csf::CsfStringtable::iter
///
/// # Examples
///
/// ```ignore
/// use rust_alert::csf::CsfStringtable;
///
/// let csf = CsfStringtable::default();
/// let mut iter = csf.iter();
/// ```
pub struct Iter<'a> {
    iter: HashSetIter<'a, CsfLabel>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a CsfLabel;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> From<HashSetIter<'a, CsfLabel>> for Iter<'a> {
    fn from(iter: HashSetIter<'a, CsfLabel>) -> Self {
        Self { iter }
    }
}

/// An owning iterator over [`CsfStringtable`][csf] labels. This struct can be
/// created by [`into_iter`][into_iter] method of a `CsfStringtable`
/// (which is provided by [`IntoIterator`] trait).
///
/// [csf]: crate::csf::CsfStringtable
/// [into_iter]: crate::csf::CsfStringtable::iter
///
/// # Examples
///
/// ```ignore
/// use rust_alert::csf::CsfStringtable;
///
/// let csf = CsfStringtable::default();
/// let mut iter = csf.into_iter();
/// ```
pub struct IntoIter {
    iter: HashSetIntoIter<CsfLabel>,
}

impl Iterator for IntoIter {
    type Item = CsfLabel;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl From<HashSetIntoIter<CsfLabel>> for IntoIter {
    fn from(iter: HashSetIntoIter<CsfLabel>) -> Self {
        Self { iter }
    }
}

/// A draining iterator over [`CsfStringtable`][csf] labels. This struct can be
/// created by [`drain`][drain] method of a `CsfStringtable`.
///
/// [csf]: crate::csf::CsfStringtable
/// [drain]: crate::csf::CsfStringtable::drain
///
/// # Examples
///
/// ```ignore
/// use rust_alert::csf::CsfStringtable;
///
/// let mut csf = CsfStringtable::default();
/// let mut iter = csf.drain();
/// ```
pub struct Drain<'a> {
    drain: HashSetDrain<'a, CsfLabel>,
}

impl<'a> Iterator for Drain<'a> {
    type Item = CsfLabel;

    fn next(&mut self) -> Option<Self::Item> {
        self.drain.next()
    }
}

impl<'a> From<HashSetDrain<'a, CsfLabel>> for Drain<'a> {
    fn from(drain: HashSetDrain<'a, CsfLabel>) -> Self {
        Self { drain }
    }
}

#[cfg(test)]
mod examples {
    mod iter {
        use crate as rust_alert;

        #[test]
        fn next() {
            use rust_alert::csf::CsfStringtable;

            let csf = CsfStringtable::default();
            let mut _iter = csf.iter();
        }
    }

    mod into_iter {
        use crate as rust_alert;

        #[test]
        fn next() {
            use rust_alert::csf::CsfStringtable;

            let csf = CsfStringtable::default();
            let mut _iter = csf.into_iter();
        }
    }

    mod drain {
        use crate as rust_alert;

        #[test]
        fn next() {
            use rust_alert::csf::CsfStringtable;

            let mut csf = CsfStringtable::default();
            let mut _iter = csf.drain();
        }
    }
}
