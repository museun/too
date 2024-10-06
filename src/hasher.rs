use std::hash::BuildHasherDefault;

#[derive(Default)]
pub struct IntHasher(u64);

impl std::hash::Hasher for IntHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.0 = i
    }

    #[inline(never)]
    fn write(&mut self, _: &[u8]) {
        unreachable!()
    }
}

#[derive(Default)]
pub struct BuildIntHasher;

impl std::hash::BuildHasher for BuildIntHasher {
    type Hasher = IntHasher;

    fn build_hasher(&self) -> Self::Hasher {
        IntHasher(0)
    }
}

pub type DefaultIntHasher = BuildHasherDefault<IntHasher>;

#[inline(always)]
pub(crate) const fn hash_fnv_1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        index += 1;
    }
    hash
}
