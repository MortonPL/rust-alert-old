//! Hashing related things.

/// A [`std::hash::Hasher`] that does nothing.
/// Useful if we want to use a HashMap but you already guarantee that they keys are unique.
#[derive(Default)]
pub struct Nothing32Hasher(u64);

impl std::hash::Hasher for Nothing32Hasher {
    fn finish(&self) -> u64 {
        self.0
    }

    #[cfg(not(tarpaulin_include))]
    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!()
    }

    fn write_i32(&mut self, i: i32) {
        self.0 = i as u64;
    }
}

/// A [`std::hash::BuildHasherDefault`] with [`Nothing32Hasher`].
pub type BuildNothingHasher = std::hash::BuildHasherDefault<Nothing32Hasher>;
