
use std::raw::TraitObject as TORepr;

use super::MetaData;

union Repr<T: Copy> {
    t: T,
    repr: TORepr
}

unsafe impl<T: ?Sized> MetaData for TraitObject<T> {
    type Data = Ptr;
    type This = T;

    #[inline]
    fn decompose(t: &T) -> (*const u8, Self::Data) {
        let repr = Repr { t };

        unsafe {
            let Repr { repr: TORepr { data, vtable } } = repr;

            (data as _, Ptr(vtable))
        }
    }

    #[inline]
    unsafe fn compose(ptr: *const u8, data: Self::Data) -> *mut T {
        let repr = Repr {
            repr: TORepr {
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

pub struct TraitObject<T: ?Sized>([*const T]);
