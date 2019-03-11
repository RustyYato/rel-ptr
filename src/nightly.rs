
use std::raw::TraitObject as TORepr;

use super::MetaData;

union Trans<T: Copy, U: Copy> {
    t: T,
    u: U
}

unsafe impl<T: ?Sized> MetaData for TraitObject<T> {
    type Data = Ptr;

    #[inline]
    fn decompose(t: &Self) -> (*const u8, Self::Data) {
        let repr = Trans { t };

        unsafe {
            let Trans { u: TORepr { data, vtable } } = repr;

            (data as _, Ptr(vtable))
        }
    }

    #[inline]
    unsafe fn compose(ptr: *const u8, data: Self::Data) -> *mut Self {
        let repr = Trans {
            u: TORepr {
                data: ptr as _,
                vtable: data.0
            }
        };
        
        repr.t
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ptr(*mut ());

impl Default for Ptr {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}

trait Trait<T: ?Sized> {}

/**
 * `TraitObject` represents a trait object generically
 * 
 * # Safety
 * 
 * The only types that are safe to use with `TraitObject`
 * are trait objects, using anything else is UB
 */
#[repr(transparent)]
pub struct TraitObject<T: ?Sized>(dyn Trait<T>);

impl<T: ?Sized> TraitObject<T> {
    pub unsafe fn new(t: &T) -> &Self {
        Trans { t }.u
    }

    pub unsafe fn into(&self) -> &T {
        Trans { t: self }.u
    }
}
