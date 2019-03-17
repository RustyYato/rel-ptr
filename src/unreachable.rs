pub trait UncheckedOptionExt {
    type T;

    unsafe fn unchecked_unwrap(self) -> Self::T;
}

impl<T> UncheckedOptionExt for Option<T> {
    type T = T;
    
    #[allow(clippy::assertions_on_constants)]
    unsafe fn unchecked_unwrap(self) -> T {
        match self {
            Some(value) => value,
            None => {
                debug_assert!(false);
                std::hint::unreachable_unchecked()
            }
        }
    }
}