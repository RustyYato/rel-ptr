use std::raw::TraitObject as TORepr;
use core::num::*;

use super::{MetaData, IntegerDeltaError, IntegerDeltaErrorImpl, Delta};

union Trans<T: Copy, U: Copy> {
    t: T,
    u: U,
}

unsafe impl<T: ?Sized> MetaData for TraitObject<T> {
    type Data = Ptr;

    #[inline]
    fn decompose(t: &mut Self) -> (*mut u8, Self::Data) {
        let repr = Trans { t: t as *mut Self };

        unsafe {
            let Trans {
                u: TORepr { data, vtable },
            } = repr;

            (data as _, Ptr(vtable))
        }
    }

    #[inline]
    unsafe fn compose(ptr: *mut u8, data: Self::Data) -> *mut Self {
        let repr = Trans {
            u: TORepr {
                data: ptr as _,
                vtable: data.0,
            },
        };

        repr.t
    }
}

/**
 * A opaque pointer that implements default
 *
 * for internal use only
 */
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ptr(*mut ());

impl Default for Ptr {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}

trait Trait<T: ?Sized> {}

/// `TraitObject` represents a trait object generically
#[repr(transparent)]
pub struct TraitObject<T: ?Sized>(dyn Trait<T>);

impl<T: ?Sized> TraitObject<T> {
    /**
     * make a new `TraitObject` for use in `RelPtr`
     * 
     * # Safety
     * 
     * This is only safe if `T` is a trait object
     */
    pub unsafe fn from(t: *mut T) -> *mut Self {
        Trans { t }.u
    }

    /**
     * convert a `TraitObject` into the underlying trait object
     * 
     * # Safety
     * 
     * This is only safe if `T` is a trait object
     */
    pub unsafe fn into(t: *mut Self) -> *mut T {
        Trans { t }.u
    }
}

macro_rules! impl_delta_nonzero {
    ($($type:ident $base:ident),* $(,)?) => {$(
        unsafe impl Delta for $type {
            type Error = IntegerDeltaError;

            fn sub(a: *mut u8, b: *mut u8) -> Result<Self, Self::Error> {
                let del = match isize::checked_sub(a as usize as _, b as usize as _) {
                    None => return Err(IntegerDeltaError(IntegerDeltaErrorImpl::Sub(a as usize, b as usize))),
                    Some(0) => return Err(IntegerDeltaError(IntegerDeltaErrorImpl::InvalidNonZero)),
                    Some(del) => del,
                };

                if std::mem::size_of::<Self>() < std::mem::size_of::<isize>() && (
                    ($base::min_value() as isize) > del ||
                    ($base::max_value() as isize) < del
                )
                {
                    Err(IntegerDeltaError(IntegerDeltaErrorImpl::Conversion(del)))
                } else {
                    // 0 case was checked in match before hand, so this is guarenteed ot be non zero
                    unsafe { Ok(Self::new_unchecked(del as _)) }
                }
            }

            unsafe fn sub_unchecked(a: *mut u8, b: *mut u8) -> Self {
                use unreachable::UncheckedOptionExt;

                Self::new_unchecked(isize::checked_sub(a as usize as _, b as usize as _).unchecked_unwrap() as _)
            }

            unsafe fn add(self, a: *const u8) -> *mut u8 {
                <*mut u8>::offset(a as _, self.get() as isize) as *mut u8
            }
        }
    )*};
}

impl_delta_nonzero! { NonZeroI8 i8, NonZeroI16 i16, NonZeroI32 i32, NonZeroI64 i64, NonZeroI128 i128, NonZeroIsize isize }