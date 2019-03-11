#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(missing_docs)]

/*!
# rel-ptr

`rel-ptr` a library for relative pointers, which can be used to create
moveable self-referential types. This library was inspired by
Johnathan Blow's work on Jai, where he added relative pointers
as a primitive into Jai.

A relative pointer is a pointer that uses an offset and it's current location to
calculate where it points to.

## Example

take the memory segment below

`[.., 0x3a, 0x10, 0x02, 0xe4, 0x2b ..]`

where `0x3a` has the address `0xff304050` (32-bit system)
then `0x2b` has the address `0xff304054`.

if we have a 1-byte relative pointer (`RelPtr<_, i8>`)
at the address `0xff304052`, then that relative pointer points to
`0x2b` as well, this is because its address `0xff304052`, plus its
offset, `0x02` points to `0x2b`.

There are three interesting things
about this
1) it only took 1 byte to point to another value,
2) a relative pointer cannot access all memory, only memory near it
3) if both the relative pointer and the pointee move together,
   then the relative pointer will not be invalidated

The third point is what makes moveable self-referential structs possible

The type `RelPtr<T, I>` is a relative pointer. `T` is what it points to,
and `I` is what it uses to store its offset. In practice you can ignore `I`,
which is defaulted to `isize`, because that will cover all of your cases for using
relative pointers. But if you want to optimize the size of the pointer, you can use
any type that implements `Delta`. Some types from std that do so are:
`i8`, `i16`, `i32`, `i64`, `i128`, and `isize`. Note that the trade off is that as you
decrease the size of the offset, you decrease the range to which you can point to.
`isize` will cover at least half of addressable memory, so it should work unless you do
something really crazy. For self-referential structs use a type whose max value is atleast
as big as your struct. i.e. `std::mem::size_of::<T>() <= I::max_value()`.

Note on usized types: these are harder to get working 

## Self Referential Type Example

```rust
# fn main() {
# use rel_ptr::RelPtr;
 struct SelfRef {
     value: (String, u32),
     ptr: RelPtr<String, i8>
 }

 impl SelfRef {
     pub fn new(s: String, i: u32) -> Self {
         let mut this = Self {
             value: (s, i),
             ptr: RelPtr::null()
         };
         
         this.ptr.set(&this.value.0).unwrap();
         
         this
     }

     pub fn fst(&self) -> &str {
         unsafe { self.ptr.as_ref_unchecked() }
     }

     pub fn snd(&self) -> u32 {
         self.value.1
     }
 }

 let s = SelfRef::new("Hello World".into(), 10);
 
 assert_eq!(s.fst(), "Hello World");
 assert_eq!(s.snd(), 10);
# }
```

TODO: Finish example
*/

#[cfg(not(feature = "std"))]
extern crate core as std;

use std::marker::PhantomData;
use std::ptr::NonNull;

/**
 * `Delta` trait generalizes differences in
 * memory locations to types like i8 and i16
 * 
 * Note: certain invariants must be upheld to fulfill
 * the unsafe contract of this trait, these invariants
 * are detailed in each function
 * 
 * This trait is intended to be used with `RelPtr`
 */
pub unsafe trait Delta: Copy + Eq {
    /// The value No change in two pointer locations,
    const ZERO: Self;

    /// Error of `Delta::sub`
    type Error;

    /**
     * The difference between two pointers
     * 
     * Note: for all values of `a: *const u8`,
     * you must enforce that `Delta::sub(a, a) == Delta::ZERO`
     * and that the following function does not panic for all values
     * of `a` and `b`
     * 
     * ```ignore
     *  fn for_all_a_b(a: *const u8, b: *const u8) {
     *      if let Some(x) = Self::sub(a, b) {
     *          unsafe { assert_eq!(Self::add(x, b), a) }
     *      }
     *  }
     * ```
    */
    fn sub(a: *const u8, b: *const u8) -> Result<Self, Self::Error>;

    /**
     * Adds the difference (in `self`) to the pointer `a`
     * 
     * Note: for all values of `a: *const u8`,
     * you must enforce that `Delta::add(Delta::ZERO, a) == a`
     * and that the following function does not panic for all values
     * of `a` and `b`
     * 
     * ```ignore
     *  fn for_all_a_b(a: *const u8, b: *const u8) {
     *      if let Some(x) = Self::sub(a, b) {
     *          unsafe { assert_eq!(Self::add(x, b), a) }
     *      }
     *  }
     * ```
     * 
     * # Safety
     * TODO
    */
    unsafe fn add(self, a: *const u8) -> *mut u8;
}

macro_rules! impl_delta {
    ($($type:ty),* $(,)?) => {$(
        unsafe impl Delta for $type {
            const ZERO: Self = 0;
            type Error = IntegerDeltaError;

            fn sub(a: *const u8, b: *const u8) -> Result<Self, Self::Error> {
                let del = match isize::checked_sub(a as usize as _, b as usize as _) {
                    Some(del) => del,
                    None => return Err(IntegerDeltaError(IntegerDeltaErrorImpl::Sub(a as usize, b as usize)))
                };
                
                if std::mem::size_of::<Self>() < std::mem::size_of::<isize>() && (
                    (Self::min_value() as isize) > del ||
                    (Self::max_value() as isize) < del
                )
                {
                    Err(IntegerDeltaError(IntegerDeltaErrorImpl::Conversion(del)))
                } else {
                    Ok(del as _)
                }
            }

            unsafe fn add(self, a: *const u8) -> *mut u8 {
                <*const u8>::offset(a, self as isize) as *mut u8
            }
        }
    )*};
}

/**
 * If an integer's range is too small to store an offset, then
 * this error is generated
 */
#[derive(Debug)]
pub struct IntegerDeltaError(IntegerDeltaErrorImpl);

#[derive(Debug)]
enum IntegerDeltaErrorImpl {
    Conversion(isize),
    Sub(usize, usize)
}

#[cfg(feature = "std")]
impl std::error::Error for IntegerDeltaError {}

mod fmt {
    use super::*;
    use std::fmt;

    impl fmt::Display for IntegerDeltaError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self.0 {
                IntegerDeltaErrorImpl::Conversion(del) =>
                    write!(f, "Offset could not be stored (offset of {} is too large)", del),
                IntegerDeltaErrorImpl::Sub(a, b) =>
                    write!(f, "Difference is beween {} and {} overflows `isize`", a, b),
            }
        }
    }
}

impl_delta! { i8, i16, i32, i64, i128, isize }

// unsafe trait MetaData {
//     type Data;

//     fn decompose(this: *const Self) -> (*const (), Self::Data);

//     fn compose(ptr: *const (), data: Self::Data) -> *const Self;
// }

// unsafe impl<T> MetaData for T {
//     type Data = ();

//     fn decompose(this: *const Self) -> (*const u8)
// }

/**
 * This represents a relative pointers
 * 
 * A relative pointer stores an offset, and uses its
 * that in combination with its current position in memory
 * to point to a value
 * 
 * See crate documentation for more information
*/
#[repr(transparent)]
pub struct RelPtr<T: ?Sized, I: Delta = isize>(I, PhantomData<*mut T>);

impl<T: ?Sized, I: Delta> Clone for RelPtr<T, I> { fn clone(&self) -> Self { *self } }
impl<T: ?Sized, I: Delta> Copy for RelPtr<T, I> {}
impl<T: ?Sized, I: Delta> PartialEq for RelPtr<T, I> {
    fn eq(&self, other: &Self) -> bool { std::ptr::eq(self, other) }
}
impl<T: ?Sized, I: Delta> Eq for RelPtr<T, I> {}

impl<T, I: Delta> RelPtr<T, I> {
    /**
     * A null relative pointer has an offset of 0, (points to itself)
    */
    #[inline(always)]
    pub fn null() -> Self {
        Self(I::ZERO, PhantomData)
    }

    /**
     * Check if relative pointer is null
     */
    #[inline(always)]
    pub fn is_null(&self) -> bool {
        self.0 == I::ZERO
    }

    /**
     * set the offset of a relative pointer,
     * if the offset cannot be calculated using the given
     * `Delta`, then `None` will be returned, and there will be
     * **no** change to the offset
     */
    #[inline]
    pub fn set(&mut self, value: &T) -> Result<(), I::Error> {
        self.0 = I::sub(value as *const T as _, self as *mut Self as _)?;

        Ok(())
    }

    /**
     * Converts the relative pointer into a normal raw pointer
     * 
     * Note: if `self.is_null()` then a null pointer will be returned
     * 
     * # Safety
     * 
     * You must ensure that if `RelPtr::set` was called successfully before
     * calling this function and that the value pointed to does not change it's 
     * offset relative to `RelPtr`
     * 
     * if `RelPtr::set` was never called successfully, this function is safe and returns
     * `None` because the only way to construct a `RelPtr` is to make a null ptr and change it
     * through `RelPtr::set`
    */
    #[inline]
    pub unsafe fn as_raw(&self) -> *mut T {
        if self.is_null() {
            std::ptr::null_mut()
        } else {
            self.as_raw_unchecked()
        }
    }

    /**
     * Converts the relative pointer into a normal raw pointer
     * 
     * # Safety
     * 
     * You must ensure that `RelPtr::set` was called successfully before
     * calling this function and that the value pointed to does not change it's 
     * offset relative to `RelPtr`
     * 
     * if `RelPtr::set` was never called successfully, this function is UB
    */
    #[inline]
    pub unsafe fn as_raw_unchecked(&self) -> *mut T {
        self.0.add(self as *const Self as _) as _
    }

    /**
     * Converts the relative pointer into a normal raw pointer
     * 
     * # Safety
     * 
     * Same as `RelPtr::as_raw`
    */
    #[inline]
    pub unsafe fn as_non_null(&self) -> Option<NonNull<T>> {
        self.as_ref().map(NonNull::from)
    }

    /**
     * Converts the relative pointer into a normal raw pointer
     * 
     * # Safety
     * 
     * Same as `RelPtr::as_raw_unchecked`
    */
    #[inline]
    pub unsafe fn as_non_null_unchecked(&self) -> NonNull<T> {
        NonNull::new_unchecked(self.as_raw_unchecked())
    }

    /**
     * Gets a reference from the relative pointer,
     * if the relative pointer is null, then `None` is
     * returned
     * 
     * # Safety
     * 
     * Same as `RelPtr::as_raw`
     */
    #[inline]
    pub unsafe fn as_ref(&self) -> Option<&T> {
        <*const T>::as_ref(self.as_raw())
    }

    /**
     * Gets a mutable reference from the relative pointer,
     * if the relative pointer is null, then `None` is
     * returned
     * 
     * # Safety
     * 
     * Same as `RelPtr::as_raw`
     */
    #[inline]
    pub unsafe fn as_mut(&mut self) -> Option<&mut T> {
        <*mut T>::as_mut(self.0.add(self as *const Self as _) as _)
    }
    
    /**
     * Gets a reference from the relative pointer
     * 
     * # Safety
     * 
     * Same as `RelPtr::as_raw_unchecked`
     */
    #[inline]
    pub unsafe fn as_ref_unchecked(&self) -> &T {
        &*self.as_raw_unchecked()
    }
    
    
    /**
     * Gets a mutable reference from the relative pointer
     * 
     * # Safety
     * 
     * Same as `RelPtr::as_raw_unchecked`
     */
    #[inline]
    pub unsafe fn as_mut_unchecked(&mut self) -> &mut T {
        &mut *self.as_raw_unchecked()
    }
}

#[cfg(test)]
mod tests {
    use super::{RelPtr, Delta};

    struct SelfRef<T> {
        t_ref: RelPtr<T, i8>,
        t: T,
    }

    impl<T> SelfRef<T> {
        pub fn new<U: Into<T>>(t: U) -> Self {
            let mut this = Self {
                t: t.into(), t_ref: RelPtr::null()
            };

            this.t_ref.set(&this.t);

            this
        }

        pub fn t(&self) -> &T {
            &self.t
        }

        pub fn t_mut(&mut self) -> &mut T {
            &mut self.t
        }

        pub fn t_ref(&self) -> &T {
            unsafe { self.t_ref.as_ref_unchecked() }
        }

        pub fn t_ref_mut(&mut self) -> &mut T {
            unsafe { self.t_ref.as_mut_unchecked() }
        }
    }

    #[inline(never)]
    fn block_opt<T>(x: T) -> T { x }

    #[test]
    fn simple_test() {
        let mut s = SelfRef {
            t: "Hello World", t_ref: RelPtr::null()
        };

        s.t_ref.set(&s.t);
        
        assert_eq!(s.t(), s.t_ref());
        assert_eq!(*s.t(), "Hello World");
        assert_eq!(*s.t_ref(), "Hello World");
    }

    #[test]
    fn simple_move() {
        let mut s = SelfRef {
            t: "Hello World", t_ref: RelPtr::null()
        };

        s.t_ref.set(&s.t);

        assert_eq!(s.t(), s.t_ref());
        assert_eq!(*s.t(), "Hello World");
        assert_eq!(*s.t_ref(), "Hello World");
        
        let s = block_opt(s);

        assert_eq!(s.t(), s.t_ref());
        assert_eq!(*s.t(), "Hello World");
        assert_eq!(*s.t_ref(), "Hello World");
    }

    #[test]
    fn simple_move_after_init() {
        let mut s = SelfRef::<&str>::new("Hello World");

        assert_eq!(s.t(), s.t_ref());
        assert_eq!(*s.t(), "Hello World");
        assert_eq!(*s.t_ref(), "Hello World");
        
        let s = block_opt(s);

        assert_eq!(s.t(), s.t_ref());
        assert_eq!(*s.t(), "Hello World");
        assert_eq!(*s.t_ref(), "Hello World");
    }

    #[test]
    fn swap() {
        let mut s = SelfRef::<&str>::new("Hello World");
        let mut x = SelfRef::<&str>::new("Killer Move");

        assert_eq!(*s.t(), "Hello World");
        assert_eq!(*x.t(), "Killer Move");

        assert_eq!(*s.t_ref(), "Hello World");
        assert_eq!(*x.t_ref(), "Killer Move");
        
        std::mem::swap(&mut s, &mut x);

        assert_eq!(*s.t(), "Killer Move");
        assert_eq!(*x.t(), "Hello World");

        assert_eq!(*s.t_ref(), "Killer Move");
        assert_eq!(*x.t_ref(), "Hello World");
    }

    #[test]
    fn aliasing() {
        let mut s = SelfRef::<&str>::new("Hello World");
        
        assert_eq!(s.t(), s.t_ref());
        
        *s.t_mut() = "Killer Move";
        
        assert_eq!(*s.t(), "Killer Move");
        assert_eq!(*s.t_ref(), "Killer Move");
    }

    #[test]
    fn check_copy() {
        struct NoCopy;

        let mut s = SelfRef::<NoCopy>::new(NoCopy);
        
        let x = s.t_ref;
        let x = s.t_ref;
    }
}