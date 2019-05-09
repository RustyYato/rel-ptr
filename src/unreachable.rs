
/// Adds an unchecked unwrap, this unwrap is UB if self is None
pub(crate) trait UncheckedOptionExt {
    type T;

    unsafe fn unchecked_unwrap(self) -> Self::T;
}

impl<T> UncheckedOptionExt for Option<T> {
    type T = T;
    
    #[inline]
    #[allow(clippy::assertions_on_constants)]
    unsafe fn unchecked_unwrap(self) -> T {
        match self {
            Some(value) => value,
            None if cfg!(debug_assertions) =>
                panic!("Tried to use an unset relative pointer, this is UB in release mode!"),
            None => std::hint::unreachable_unchecked()
        }
    }
}
