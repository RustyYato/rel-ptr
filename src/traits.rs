
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
    /// Error of `Delta::sub`
    type Error;

    /**
     * The difference between two pointers
     *
     * Note: for all values of `a: *mut u8`,
     * you must enforce that `Delta::sub(a, a) == Delta::ZERO`
     * and that the following function does not panic for all values
     * of `a` and `b`
     *
     * ```ignore
     *  fn for_all_a_b(a: *mut u8, b: *mut u8) {
     *      if let Some(x) = Self::sub(a, b) {
     *          unsafe { assert_eq!(Self::add(x, b), a) }
     *      }
     *  }
     * ```
     */
    fn sub(a: *mut u8, b: *mut u8) -> Result<Self, Self::Error>;

    /**
     * The difference between two pointers
     *
     * Note: for all values of `a: *mut u8`,
     * you must enforce that `Delta::sub(a, a) == Delta::ZERO`
     * and that the following function does not panic for all values
     * of `a` and `b` if the difference between `a` and `b` is valid
     * 
     * ```ignore
     *  fn for_all_a_b(a: *mut u8, b: *mut u8) {
     *      unsafe { assert_eq!(Self::add(Self::sub_unchecked(a, b), b), a) }
     *  }
     * ```
     * 
     * Safety:
     * 
     * If the difference between `a` and `b` is not
     * representable by `Self` is UB
     */
    unsafe fn sub_unchecked(a: *mut u8, b: *mut u8) -> Self;

    /**
     * Adds the difference (in `self`) to the pointer `a`
     *
     * Note: for all values of `a: *mut u8`,
     * you must enforce that `Delta::add(Delta::ZERO, a) == a`
     * and that the following function does not panic for all values
     * of `a` and `b`
     *
     * ```ignore
     *  fn for_all_a_b(a: *mut u8, b: *mut u8) {
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

/// A index which can contain null
/// 
/// # Safety
/// 
/// ```ignore
/// fn for_all_a(a: *mut u8) {
///     assert_eq!(a, Self::add(Self::NULL, a))
/// }
/// ```
pub trait Nullable: Delta {
    /// The value No change in two pointer locations,
    const NULL: Self;
}

/**
 * A trait to abstract over the sizedness of types,
 * and to access metadata about a type
 *
 * If [Custom DST](https://github.com/rust-lang/rfcs/pull/2594) lands and stablizes,
 * then it will replace `MetaData`
 */
pub unsafe trait MetaData {
    /// the type of meta data a type carries
    type Data: Copy + Eq;

    /// decompose a type into a thin pointer and some metadata
    fn decompose(this: &mut Self) -> (*mut u8, Self::Data);

    /// recompose a type from a thin pointer and some metadata
    ///
    /// it is guarenteed that the metadata is
    /// * `ptr == null` `Self::Data` is undefined
    /// * `ptr != null` generated from `MetaData::decompose`
    unsafe fn compose(ptr: *mut u8, data: Self::Data) -> *mut Self;
}

unsafe impl<T> MetaData for T {
    type Data = ();

    #[inline]
    fn decompose(this: &mut Self) -> (*mut u8, Self::Data) {
        (this as *mut Self as _, ())
    }

    #[inline]
    unsafe fn compose(ptr: *mut u8, (): Self::Data) -> *mut Self {
        ptr as _
    }
}

unsafe impl<T> MetaData for [T] {
    type Data = usize;

    #[inline]
    fn decompose(this: &mut Self) -> (*mut u8, Self::Data) {
        (this.as_ptr() as _, this.len())
    }

    #[inline]
    unsafe fn compose(ptr: *mut u8, data: Self::Data) -> *mut Self {
        std::slice::from_raw_parts_mut(ptr as _, data)
    }
}

unsafe impl MetaData for str {
    type Data = usize;

    #[inline]
    fn decompose(this: &mut Self) -> (*mut u8, Self::Data) {
        (this.as_ptr() as _, this.len())
    }

    #[inline]
    unsafe fn compose(ptr: *mut u8, data: Self::Data) -> *mut Self {
        std::str::from_utf8_unchecked_mut(std::slice::from_raw_parts_mut(ptr as _, data)) as *mut Self
            as _
    }
}
