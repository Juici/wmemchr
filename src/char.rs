use core::mem;

mod private {
    pub trait Seal<T = ()> {}
}

/// A trait for types that have a constant size known at compile time.
pub trait Sized: core::marker::Sized {
    /// The number of bytes used to represent the type in memory.
    const SIZE: usize = mem::size_of::<Self>();
    /// The number of bits used to represent the type in memory.
    const BITS: usize = Self::SIZE * 8;
}

impl<T: core::marker::Sized> Sized for T {}

pub type Packed = u128;

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
            // Avoid hardcoding these constants, in the case we change the size
            // of the packed representation.
            const LO: Packed = {
                let lo = 1;

                let mut res = lo;
                let mut i = 0;
                while i < <Self as Pack>::LANES {
                    res = (res << <Self as Sized>::BITS) | lo;
                    i += 1;
                }
                res
            };
            const HI: Packed = {
                let hi = 1;

                let mut res = hi;
                let mut i = 0;
                while i < <Self as Pack>::LANES {
                    res = (res << <Self as Sized>::BITS) | hi;
                    i += 1;
                }
                res
            };

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
pub trait Wide: private::Seal + Pack + Copy + Eq + 'static {}

macro_rules! impl_wide {
    ($($ty:ty),*) => {
        $(
            impl Wide for $ty {}
            impl private::Seal for $ty {}
        )*
    };
}
impl_wide!(u16, u32, i16, i32);

pub(crate) trait PackedWide: Sized {
    /// Returns true if one of the elements in the packed representation is zero.
    fn contains_zero<T: Wide>(self) -> bool;
}

impl PackedWide for Packed {
    #[inline(always)]
    fn contains_zero<T: Wide>(self) -> bool {
        (self.wrapping_sub(<T as Pack>::LO) & !self & <T as Pack>::HI) != 0
    }
}
