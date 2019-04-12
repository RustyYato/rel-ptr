use std::raw::TraitObject as TORepr;

use super::{MetaData, IntegerDeltaError, IntegerDeltaErrorImpl, Delta, Ptr};
use crate::unreachable::UncheckedOptionExt as _;

/// Union to reinterpret bits
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

/// This takes the place of any trait, this is to allow
/// generalizing over all trait objects
trait Trait<T: ?Sized> {}

/**
 * `TraitObject` represents a trait object generically
 * 
 * You can use trait objects with `RelPtr` like so,
 * 
 * ```rust
 *  fn main() {
 *      use rel_ptr::{RelPtr, TraitObject};
 *      
 *      type RelPtrTO = RelPtr<TraitObject<dyn std::any::Any>>;
 *      
 *      // value to store in `RelPtr`
 *      let mut value: [u8; 10] = [0; 10];
 *      
 *      // setup `RelPtr`
 *      let mut ptr: RelPtrTO = RelPtr::null();
 *  
 *      // This is safe because `dyn std::any::Any` is a trait object
 *      // make `&mut TraitObject<dyn std::any::Any>`
 *      let to = unsafe { TraitObject::from_mut(
 *          &mut value as &mut dyn std::any::Any
 *      ) };
 *      
 *      // set `RelPtr`
 *      ptr.set(to);
 * 
 *      // ... use `RelPtr`
 *  }
 * ```
 * 
 * # Safety
 * 
 * It is unsafe to use TraitObject with anything other than an actual trait object
 */
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
    pub unsafe fn from_ref(t: &T) -> &Self {
        Trans::<&T, &Self> { t: t as _ }.u
    }

    /**
     * make a new `TraitObject` for use in `RelPtr`
     * 
     * # Safety
     * 
     * This is only safe if `T` is a trait object
     */
    pub unsafe fn from_mut(t: &mut T) -> &mut Self {
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
