use core::mem;

mod private {
    pub trait Sealed {}
}

/// A trait for types that have a constant size known at compile time.
pub trait Sized: core::marker::Sized {
    /// The number of bytes used to represent the type in memory.
    const SIZE: usize = mem::size_of::<Self>();
    /// The number of bits used to represent the type in memory.
    const BITS: usize = Self::SIZE * 8;
}

impl<T: core::marker::Sized> Sized for T {}

// TODO: Specialise each wide character type for different pointer widths.
cfg_if::cfg_if! {
    if #[cfg(any(target_pointer_width = "8", target_pointer_width = "16"))] {
        // If usize is less than 32 bits, use a u32.
        type _Packed = u32;
        type _NonZeroPacked = core::num::NonZeroU32;
    } else {
        type _Packed = usize;
        type _NonZeroPacked = core::num::NonZeroUsize;
    }
}

/// An integer holding a packed vector of wide characters.
pub type Packed = _Packed;
/// An non-zero bitmask the same size as [`Packed`].
pub type NonZeroPacked = _NonZeroPacked;

/// A trait for types that can be packed into a [`Packed`].
pub trait Pack: Sized {
    /// The number of lanes that the packed representation can hold.
    const LANES: usize = Packed::SIZE / Self::SIZE;

    /// A packed representation of the the lowest bit.
    const LO: Packed;
    /// A packed representation of the the highest bit.
    const HI: Packed;

    /// Broadcasts the value across a [`Packed`].
    fn broadcast(self) -> Packed;
}

macro_rules! impl_pack {
    ($ity:ty, $uty:ty) => {
        impl Pack for $ity {
            const LO: Packed = <$uty as Pack>::LO;
            const HI: Packed = <$uty as Pack>::HI;

            #[inline(always)]
            fn broadcast(self) -> Packed {
                // The method by which we broadcast relies on unsigned arithmetic.
                <$uty as Pack>::broadcast(self as $uty)
            }
        }

        impl Pack for $uty {
            const LO: Packed = Packed::MAX / (<$uty>::MAX as Packed);
            const HI: Packed = <$uty as Pack>::LO << (<$uty as Sized>::BITS - 1);

            #[inline(always)]
            fn broadcast(self) -> Packed {
                (self as Packed) * <$uty as Pack>::LO
            }
        }
    };
}
impl_pack!(i16, u16);
impl_pack!(i32, u32);

/// A trait for wide character types.
pub trait Wide: private::Sealed + Pack + Copy + Eq + 'static {}

macro_rules! impl_wide {
    ($($ty:ty),*) => {
        $(
            impl Wide for $ty {}
            impl private::Sealed for $ty {}
        )*
    };
}
impl_wide!(u16, u32, i16, i32);

#[inline(always)]
pub(crate) fn simd_eq<T: Wide>(a: Packed, b: Packed) -> Packed {
    let xor = a ^ b;
    xor.wrapping_sub(<T as Pack>::LO) & !xor & <T as Pack>::HI
}
