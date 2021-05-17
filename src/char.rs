use core::mem;

mod private {
    pub trait Sealed {}
}

/// A trait for wide character types.
pub trait Wide: private::Sealed + Copy + Eq + 'static {
    /// An unsigned type of equal size.
    type Unsigned: UnsignedWide;

    /// The number of bytes used to represent the type in memory.
    const SIZE: usize = mem::size_of::<Self>();
    /// The number of bits used to represent the type in memory.
    const BITS: usize = Self::SIZE * 8;
    /// The number of this wide that can fit in a usize.
    const USIZE_WIDES: usize = mem::size_of::<usize>() / Self::SIZE;
    /// The number of this wide per loop.
    const LOOP_SIZE: usize = Self::USIZE_WIDES * 2;

    /// Transmute into an unsigned wide character.
    fn unsigned(self) -> Self::Unsigned;
}

/// A trait for unsigned wide character types.
pub trait UnsignedWide: Wide {
    const LO_USIZE: usize = {
        let lo = 1;

        let mut res = lo;
        let mut i = 0;
        while i < Self::USIZE_WIDES {
            res = (res << Self::BITS) | lo;
            i += 1;
        }

        res
    };
    const HI_USIZE: usize = {
        let hi = 1 << (Self::BITS - 1);

        let mut res = hi;
        let mut i = 0;
        while i < Self::USIZE_WIDES {
            res = (res << Self::BITS) | hi;
            i += 1;
        }

        res
    };

    /// Broadcast into a usize.
    fn broadcast_usize(self) -> usize;

    /// Cast into a usize.
    fn cast_usize(self) -> usize;
}

macro_rules! impl_wide {
    ($ity:ty, $uty:ty) => {
        impl private::Sealed for $ity {}
        impl private::Sealed for $uty {}

        impl Wide for $ity {
            type Unsigned = $uty;

            #[inline(always)]
            fn unsigned(self) -> $uty {
                self as $uty
            }
        }

        impl Wide for $uty {
            type Unsigned = $uty;

            #[inline(always)]
            fn unsigned(self) -> $uty {
                self
            }
        }

        impl UnsignedWide for $uty {
            #[inline(always)]
            fn broadcast_usize(self) -> usize {
                const FACTOR: usize = usize::MAX / <$uty>::MAX as usize;
                self.cast_usize() * FACTOR
            }

            #[inline(always)]
            fn cast_usize(self) -> usize {
                self as usize
            }
        }
    };
}
impl_wide!(i16, u16);
impl_wide!(i32, u32);
