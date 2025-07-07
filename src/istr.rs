use std::num::{NonZeroU32, NonZeroU64, NonZeroUsize};

/// An ID for an interned string. Cheap to copy, and to perform string equality checks on, as
/// internally it is simply an integer ID. By default this is backed by a [`NonZeroUsize`], which
/// allows niche optimization.
///
/// In order to get the associated string, the interned string must be looked up
/// in the interner it was created with.
///
/// Note that performing an equality check on interned strings from different
/// interners will give a nonsensical result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Istr<Repr: IstrRepr = NonZeroUsize> {
    pub(crate) repr: Repr,
}

/// A backing type for an [`Istr`].
pub trait IstrRepr: Copy + sealed::Sealed {
    /// Convert a `usize` index to this backing type.
    ///
    /// Returns `None` if the given index is too large to
    /// fit into this type.
    fn from_index(index: usize) -> Option<Self>;

    /// Convert this backing value back to a `usize`.
    ///
    /// This is allowed to panic or overflow if the value is
    /// out of bounds of a `usize`.
    fn to_index(self) -> usize;
}

impl sealed::Sealed for usize {}
impl sealed::Sealed for u64 {}
impl sealed::Sealed for u32 {}

impl sealed::Sealed for NonZeroUsize {}
impl sealed::Sealed for NonZeroU64 {}
impl sealed::Sealed for NonZeroU32 {}

impl IstrRepr for NonZeroUsize {
    #[inline]
    fn from_index(index: usize) -> Option<Self> {
        NonZeroUsize::new(index.wrapping_add(1))
    }

    #[inline]
    fn to_index(self) -> usize {
        self.get() - 1
    }
}

impl IstrRepr for NonZeroU64 {
    #[inline]
    fn from_index(index: usize) -> Option<Self> {
        let n = u64::try_from(index).ok()?;
        NonZeroU64::new(n.wrapping_add(1))
    }

    #[inline]
    fn to_index(self) -> usize {
        self.get() as usize - 1
    }
}

impl IstrRepr for NonZeroU32 {
    #[inline]
    fn from_index(index: usize) -> Option<Self> {
        let n: u32 = u32::try_from(index).ok()?;
        Self::new(n.wrapping_add(1))
    }

    #[inline]
    fn to_index(self) -> usize {
        self.get() as usize - 1
    }
}

impl IstrRepr for usize {
    #[inline]
    fn from_index(index: usize) -> Option<Self> {
        Some(index)
    }

    #[inline]
    fn to_index(self) -> usize {
        self
    }
}

impl IstrRepr for u64 {
    #[inline]
    fn from_index(index: usize) -> Option<Self> {
        u64::try_from(index).ok()
    }

    #[inline]
    fn to_index(self) -> usize {
        self as usize
    }
}

impl IstrRepr for u32 {
    #[inline]
    fn from_index(index: usize) -> Option<Self> {
        u32::try_from(index).ok()
    }

    #[inline]
    fn to_index(self) -> usize {
        self as usize
    }
}

mod sealed {
    pub trait Sealed {}
}
