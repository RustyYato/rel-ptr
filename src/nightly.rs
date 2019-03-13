use std::raw::TraitObject as TORepr;
use core::num::*;

use super::{MetaData, IntegerDeltaError, IntegerDeltaErrorImpl, Delta, Ptr};

union Trans<T: Copy, U: Copy> {
    t: T,
    u: U,
}

unsafe impl<T: ?Sized> MetaData for TraitObject<T> {
    type Data = *mut ();

    #[inline]
    fn data(t: &Self) -> Self::Data {
        unsafe { Trans::<&Self, TORepr> { t }.u.vtable }
    }

    #[inline]
    unsafe fn compose(ptr: Ptr<u8>, vtable: Self::Data) -> Ptr<Self> {
        Trans {
            u: TORepr {
                data: ptr?.as_ptr() as *mut (),
                vtable,
            },
        }.t
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
    pub unsafe fn new(t: &mut T) -> &mut Self {
        &mut *(Trans::<*mut T, *mut Self> { t: t as _ }.u)
    }

    /// convert a `TraitObject` into the underlying trait object
    pub fn as_ref(&self) -> &T {
        unsafe { &*(Trans::<*const Self, *const T> { t: self as _ }.u) }
    }

    /// convert a `TraitObject` into the underlying trait object
    pub fn as_ref_mut(&mut self) -> &mut T {
        unsafe { &mut *(Trans::<*mut Self, *mut T> { t: self as _ }.u) }
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