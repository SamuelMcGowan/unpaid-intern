![Crates.io Version](https://img.shields.io/crates/v/unpaid-intern)
![docs.rs](https://img.shields.io/docsrs/unpaid-intern)

A string interner.

A string interner is a data structure commonly used in compilers and other contexts that need to
cheaply store and compare many often identical strings. "Interning" a string returns a pointer (or in
this implementation, an ID) that is cheap to copy and to perform string equality checks on. This is
achieved by deduplicating strings using an internal hash table.

This string interner also stores all strings in a single bump-allocated arena, courtesy of [`bumpalo`](https://docs.rs/bumpalo/latest/bumpalo/),
avoiding excessive allocation.

I decided to represent interned strings with an integer ID (`NonZeroUsize` by default) instead of a reference to avoid introducing lifetimes.
This does mean that accessing the underlying string requires calling a method on the interner, but this is a
single array lookup. You can also specify the backing type as `u32`, `u64`, `usize`, `NonZeroU32` or `NonZeroU64`.

# Example
```rust
use unpaid_intern::Interner;

let interner = Interner::new();

let hello = interner.intern("hello");
let hello2 = interner.intern("hello");
let world = interner.intern("world");

// Interned strings can be compared cheaply.
assert_eq!(hello, hello2);
assert_ne!(hello, world);

// Getting the associated string for an interned string.
assert_eq!(interner.get_str(hello), Some("hello"));
```

# Other `Istr` backing types
```rust
use unpaid_intern::Interner;

let interner: Interner<u64> = Interner::with_istr_repr();

let foo = interner.intern("hiya");

assert_eq!(std::mem::size_of_val(&foo), 8);
assert_eq!(interner.get_str(foo), Some("hiya"));
```
