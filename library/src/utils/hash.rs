//! Hashing related things.

/// A std::hash::Hasher that does nothing.
/// Useful if we want to use a HashMap and guarantee that they keys are unique.
#[derive(Default)]
pub struct Nothing32Hasher(u64, i32);

impl std::hash::Hasher for Nothing32Hasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!()
    }

    fn write_i32(&mut self, i: i32) {
        self.0 = i as u64;
    }
}

pub type BuildNothingHasher = std::hash::BuildHasherDefault<Nothing32Hasher>;
