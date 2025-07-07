use std::{cell::RefCell, hash::BuildHasher, num::NonZeroUsize, ops::Index};

use hashbrown::{HashTable, hash_table::Entry};
use rustc_hash::FxBuildHasher;

use crate::{
    arena::InternerArena,
    istr::{Istr, IstrRepr},
};

#[derive(Clone, Copy)]
struct Metadata<I: IstrRepr> {
    interned: Istr<I>,
    hash: u64,
}

struct Lookup<I: IstrRepr> {
    random_state: FxBuildHasher,
    table: HashTable<Metadata<I>>,
}

/// Storage for interned strings.
pub struct Interner<I: IstrRepr = NonZeroUsize> {
    lookup: RefCell<Lookup<I>>,
    arena: InternerArena,
}

impl<I: IstrRepr> Default for Interner<I> {
    fn default() -> Self {
        Self {
            lookup: RefCell::new(Lookup {
                random_state: FxBuildHasher::default(),
                table: HashTable::default(),
            }),
            arena: InternerArena::default(),
        }
    }
}

impl Interner {
    /// Create a new interner.
    ///
    /// Uses [`NonZeroUsize`](std::num::NonZeroUsize) as the [`Istr`] backing type.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<I: IstrRepr> Interner<I> {
    /// Create a new interner with the inferred [`Istr`] backing type.
    #[inline]
    pub fn with_istr_repr() -> Self {
        Self::default()
    }

    /// Intern a string, returning an interned string that it is cheap to copy and
    /// perform equality checks on. Strings are only stored in the interner once, no
    /// matter how many times they are interned.
    ///
    ///
    /// ```rust
    /// # use unpaid_intern::Interner;
    /// #
    /// # fn main() {
    /// let interner = Interner::new();
    ///
    /// let hello = interner.intern("hello");
    /// let hello2 = interner.intern("hello");
    /// let world = interner.intern("world");
    ///
    /// assert_eq!(hello, hello2);
    /// assert_ne!(hello, world);
    /// # }
    /// ```
    ///
    /// # Panics
    /// Panics if there are no more available IDs.
    #[inline]
    pub fn intern(&self, key: &str) -> Istr<I> {
        self.try_intern(key).expect("too many interned strings")
    }

    /// Like [`Interner::intern`], but non-panicking in the case that there are no
    /// more available IDs.
    pub fn try_intern(&self, key: &str) -> Option<Istr<I>> {
        let mut lookup = self.lookup.borrow_mut();

        let hash = lookup.random_state.hash_one(key);

        let entry = lookup.table.entry(
            hash,
            |metadata| self.arena.get(metadata.interned.repr.to_index()) == Some(key),
            |metadata| metadata.hash,
        );

        let interned = match entry {
            Entry::Occupied(entry) => entry.get().interned,
            Entry::Vacant(entry) => {
                let index = self.arena.push_str(key);
                let interned = Istr {
                    repr: I::from_index(index)?,
                };

                entry.insert(Metadata { interned, hash });

                interned
            }
        };

        Some(interned)
    }

    /// Get an interned string if this string is interned, otherwise return `None`.
    ///
    /// ```rust
    /// # use unpaid_intern::{Interner, Istr};
    /// #
    /// # fn main() {
    /// let interner = Interner::new();
    ///
    /// let hello: Istr = interner.intern("hello");
    ///
    /// assert_eq!(interner.get_interned("hello"), Some(hello));
    /// assert_eq!(interner.get_interned("world"), None);
    /// # }
    /// ```
    pub fn get_interned(&self, key: &str) -> Option<Istr<I>> {
        let lookup = self.lookup.borrow();

        let hash = lookup.random_state.hash_one(key);

        lookup
            .table
            .find(hash, |metadata| {
                self.arena.get(metadata.interned.repr.to_index()) == Some(key)
            })
            .map(|metadata| metadata.interned)
    }

    /// Look up an interned string to get the associated string.
    ///
    /// Note that if the interned string was created by another interner
    /// this method will return an arbitrary string or `None`.
    ///
    /// If you know that an interned string was created by this interner, you can index into
    /// the interner instead. This is the same as calling this method, but panics if the
    /// interned string is not found in this interner instead of returning `None`.
    ///
    /// ```rust
    /// # use unpaid_intern::Interner;
    /// #
    /// # fn main() {
    /// let interner = Interner::new();
    /// let interned = interner.intern("hello");
    ///
    /// assert_eq!(interner.get_str(interned), Some("hello"));
    /// # }
    /// ```
    #[inline]
    pub fn get_str(&self, interned: Istr<I>) -> Option<&str> {
        self.arena.get(interned.repr.to_index())
    }
}

impl<I: IstrRepr> Index<Istr<I>> for Interner<I> {
    type Output = str;

    #[inline]
    fn index(&self, interned: Istr<I>) -> &Self::Output {
        self.get_str(interned).expect("string not in interner")
    }
}

#[test]
fn test_interner() {
    let interner = Interner::new();

    for n in 0..100 {
        let s = n.to_string();

        let a = interner.intern(&s);
        let b = interner.intern(&s);

        assert_eq!(a, b);
        assert_eq!(interner.get_str(a), Some(s.as_str()));
    }
}
