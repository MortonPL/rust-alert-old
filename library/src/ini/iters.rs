//! Iterators for [`IniFile`] and [`IniSection`].

use indexmap::map::{Drain as IndexMapDrain, IntoIter as IndexMapIntoIter, Iter as IndexMapIter};

use crate::ini::{IniSection, IniEntry};

/// An iterator over [`IniFile`][ini] sections. This struct can be created
/// by [`iter`][iter] method of an IniFile.
///
/// [ini]: crate::ini::IniFile
/// [iter]: crate::ini::IniFile::iter
///
/// # Examples
///
/// ```ignore
/// use rust_alert::ini::IniFile;
/// 
/// let ini = IniFile::default();
/// let mut iter = ini.iter();
/// ```
pub struct Iter<'a> {
    iter: IndexMapIter<'a, String, IniSection>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a String, &'a IniSection);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> From<IndexMapIter<'a, String, IniSection>> for Iter<'a> {
    fn from(iter: IndexMapIter<'a, String, IniSection>) -> Self {
        Self { iter }
    }
}

/// An owning iterator over [`IniFile`][ini] sections. This struct can be
/// created by [`into_iter`][into_iter] method of an `IniFile`
/// (which is provided by [`IntoIterator`] trait).
///
/// [ini]: crate::ini::IniFile
/// [into_iter]: crate::ini::IniFile::iter
///
/// # Examples
///
/// ```ignore
/// use rust_alert::ini::IniFile;
/// 
/// let ini = IniFile::default();
/// let mut iter = ini.into_iter();
/// ```
pub struct IntoIter {
    iter: IndexMapIntoIter<String, IniSection>,
}

impl Iterator for IntoIter {
    type Item = (String, IniSection);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl From<IndexMapIntoIter<String, IniSection>> for IntoIter {
    fn from(iter: IndexMapIntoIter<String, IniSection>) -> Self {
        Self { iter }
    }
}

/// A draining iterator over [`IniFile`][ini] sections. This struct can be
/// created by [`drain`][drain] method of a `IniFile`.
///
/// [ini]: crate::ini::IniFile
/// [drain]: crate::ini::IniFile::drain
///
/// # Examples
///
/// ```ignore
/// use rust_alert::ini::IniFile;
/// 
/// let mut ini = IniFile::default();
/// let mut iter = ini.drain();
/// ```
pub struct Drain<'a> {
    drain: IndexMapDrain<'a, String, IniSection>,
}

impl<'a> Iterator for Drain<'a> {
    type Item = (String, IniSection);

    fn next(&mut self) -> Option<Self::Item> {
        self.drain.next()
    }
}

impl<'a> From<IndexMapDrain<'a, String, IniSection>> for Drain<'a> {
    fn from(drain: IndexMapDrain<'a, String, IniSection>) -> Self {
        Self { drain }
    }
}

/// An iterator over [`IniSection`][ini] entries. This struct can be created
/// by [`iter`][iter] method of an IniSection.
///
/// [ini]: crate::ini::IniSection
/// [iter]: crate::ini::IniSection::iter
///
/// # Examples
///
/// ```ignore
/// use rust_alert::ini::IniSection;
/// 
/// let ini = IniSection::default();
/// let mut iter = ini.iter();
/// ```
pub struct SectionIter<'a> {
    iter: indexmap::map::Iter<'a, String, IniEntry>,
}

impl<'a> Iterator for SectionIter<'a> {
    type Item = (&'a String, &'a IniEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> From<IndexMapIter<'a, String, IniEntry>> for SectionIter<'a> {
    fn from(iter: IndexMapIter<'a, String, IniEntry>) -> Self {
        Self { iter }
    }
}

/// An owning iterator over [`IniSection`][ini] entries. This struct can be
/// created by [`into_iter`][into_iter] method of an `IniSection`
/// (which is provided by [`IntoIterator`] trait).
///
/// [ini]: crate::ini::IniSection
/// [into_iter]: crate::ini::IniSection::iter
///
/// # Examples
///
/// ```ignore
/// use rust_alert::ini::IniSection;
/// 
/// let ini = IniSection::default();
/// let mut iter = ini.into_iter();
/// ```
pub struct SectionIntoIter {
    iter: IndexMapIntoIter<String, IniEntry>,
}

impl Iterator for SectionIntoIter {
    type Item = (String, IniEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl From<IndexMapIntoIter<String, IniEntry>> for SectionIntoIter {
    fn from(iter: IndexMapIntoIter<String, IniEntry>) -> Self {
        Self { iter }
    }
}

/// A draining iterator over [`IniSection`][ini] entries. This struct can be
/// created by [`drain`][drain] method of a `IniSection`.
///
/// [ini]: crate::ini::IniSection
/// [drain]: crate::ini::IniSection::drain
///
/// # Examples
///
/// ```ignore
/// use rust_alert::ini::IniSection;
/// 
/// let mut ini = IniSection::default();
/// let mut iter = ini.drain();
/// ```
pub struct SectionDrain<'a> {
    iter: indexmap::map::Drain<'a, String, IniEntry>,
}

impl<'a> Iterator for SectionDrain<'a> {
    type Item = (String, IniEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> From<IndexMapDrain<'a, String, IniEntry>> for SectionDrain<'a> {
    fn from(iter: IndexMapDrain<'a, String, IniEntry>) -> Self {
        Self { iter }
    }
}

#[cfg(test)]
mod examples {
    use crate as rust_alert;

    #[test]
    fn iter() {
        use rust_alert::ini::IniFile;

        let ini = IniFile::default();
        let mut _iter = ini.iter();
    }

    #[test]
    fn into_iter() {
        use rust_alert::ini::IniFile;

        let ini = IniFile::default();
        let mut _iter = ini.into_iter();
    }

    #[test]
    fn drain() {
        use rust_alert::ini::IniFile;

        let mut ini = IniFile::default();
        let mut _iter = ini.drain();
    }

    #[test]
    fn section_iter() {
        use rust_alert::ini::IniSection;

        let ini = IniSection::default();
        let mut _iter = ini.iter();
    }

    #[test]
    fn section_into_iter() {
        use rust_alert::ini::IniSection;

        let ini = IniSection::default();
        let mut _iter = ini.into_iter();
    }

    #[test]
    fn section_drain() {
        use rust_alert::ini::IniSection;

        let mut ini = IniSection::default();
        let mut _iter = ini.drain();
    }
}

#[cfg(test)]
mod coverage {
    use crate::ini::{IniFile, IniSection};

    #[test]
    fn into_iter() {
        let ini = IniFile::default();
        let mut iter = ini.into_iter();
        iter.next();
    }

    #[test]
    fn section_into_iter() {
        let ini = IniSection::default();
        let mut iter = ini.into_iter();
        iter.next();
    }
}
