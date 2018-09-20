//! The `Poh` module provides an object for generating a Proof of History.
//! It records Hashes items on behalf of its users.

use hash::{hash, hashv, Hash};
use std::time::{Duration, Instant};

pub struct Poh {
    last_hash: Hash,
    num_hashes: u64,
    last_tick: Instant,
    tick_duration: Option<Duration>,
}

#[derive(Debug)]
pub struct PohEntry {
    pub num_hashes: u64,
    pub id: Hash,
    pub mixin: Option<Hash>,
}

impl Poh {
    pub fn new(last_hash: Hash, tick_duration: Option<Duration>) -> Self {
        let last_tick = Instant::now();
        Poh {
            last_hash,
            num_hashes: 0,
            last_tick,
            tick_duration,
        }
    }

    pub fn hash(&mut self) {
        self.last_hash = hash(&self.last_hash.as_ref());
        self.num_hashes += 1;
    }

    pub fn record(&mut self, mixin: Hash) -> PohEntry {
        let num_hashes = self.num_hashes + 1;
        self.num_hashes = 0;

        self.last_hash = hashv(&[&self.last_hash.as_ref(), &mixin.as_ref()]);

        PohEntry {
            num_hashes,
            id: self.last_hash,
            mixin: Some(mixin),
        }
    }

    // emissions of Ticks (i.e. PohEntries without a mixin) allows
    //  validators to parallelize the work of catching up
    pub fn tick(&mut self) -> Option<PohEntry> {
        if let Some(tick_duration) = self.tick_duration {
            if self.last_tick.elapsed() >= tick_duration {
                self.last_tick = Instant::now();
                let entry = PohEntry {
                    num_hashes: self.num_hashes,
                    id: self.last_hash,
                    mixin: None,
                };
                self.num_hashes = 0;
                return Some(entry);
            }
        }
        None
    }
}

pub fn verify(initial: Hash, entries: &[PohEntry]) -> bool {
    let mut last_hash = initial;

    for entry in entries {
        assert!(entry.num_hashes != 0);
        for _ in 1..entry.num_hashes {
            last_hash = hash(&last_hash.as_ref());
        }
        let id = match entry.mixin {
            Some(mixin) => hashv(&[&last_hash.as_ref(), &mixin.as_ref()]),
            None => hash(&last_hash.as_ref()),
        };
        if id != entry.id {
            return false;
        }
        last_hash = id;
    }

    return true;
}

#[cfg(test)]
mod tests {
    use hash::Hash;
    use poh::{self, PohEntry};

    #[test]
    #[should_panic]
    fn test_poh_verify_assert() {
        poh::verify(
            Hash::default(),
            &[PohEntry {
                num_hashes: 0,
                id: Hash::default(),
                mixin: None,
            }],
        );
    }
}
