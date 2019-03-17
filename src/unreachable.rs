pub trait UncheckedOptionExt {
    type T;

    unsafe fn unchecked_unwrap(self) -> Self::T;
}

impl<T> UncheckedOptionExt for Option<T> {
    type T = T;
    
    unsafe fn unchecked_unwrap(self) -> T {
        match self {
            Some(value) => value,
            None => std::hint::unreachable_unchecked()
        }
    }
}