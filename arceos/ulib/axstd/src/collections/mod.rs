pub use alloc::collections::*;

use axhal::misc::random;
use siphasher::sip::SipHasher13;
use core::hash::{Hasher, BuildHasher};
use core::ops::{Deref, DerefMut};
use hashbrown::HashMap as HashMapRaw;

pub struct RandomState {
    pub k0: u64,
    pub k1: u64,
}

impl RandomState {
    pub fn new() -> RandomState {
        let rand_u128 = random();
        RandomState {
            k0: (rand_u128 & 0xFFFF_FFFF_FFFF_FFFF) as u64,
            k1: (rand_u128 >> 64) as u64,
        }
    }
}

impl BuildHasher for RandomState {
    type Hasher = SipHasher13;

    fn build_hasher(&self) -> Self::Hasher {
        SipHasher13::new_with_keys(self.k0, self.k1)
    }
}

pub struct HashMap<K, V> {
    inner: HashMapRaw<K, V, RandomState>,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            inner: HashMapRaw::with_hasher(RandomState::new()),
        }
    }
}

impl<K, V> Deref for HashMap<K, V> {
    type Target = HashMapRaw<K, V, RandomState>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K, V> DerefMut for HashMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}