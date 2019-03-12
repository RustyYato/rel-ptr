
/**
 * If an integer's range is too small to store an offset, then
 * this error is generated
 */
#[derive(Debug)]
pub struct IntegerDeltaError(pub(crate) IntegerDeltaErrorImpl);

#[derive(Debug)]
pub(crate) enum IntegerDeltaErrorImpl {
    Conversion(isize),
    Sub(usize, usize),

    #[cfg(feature = "nightly")]
    InvalidNonZero
}

#[cfg(not(feature = "no_std"))]
impl std::error::Error for IntegerDeltaError {}

mod fmt {
    use super::*;
    use std::fmt;

    impl fmt::Display for IntegerDeltaError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self.0 {
                IntegerDeltaErrorImpl::Conversion(del) => write!(
                    f,
                    "Offset could not be stored (offset of {} is too large)",
                    del
                ),
                IntegerDeltaErrorImpl::Sub(a, b) => {
                    write!(f, "Difference is beween {} and {} overflows `isize`", a, b)
                },
                
                #[cfg(feature = "nightly")]
                IntegerDeltaErrorImpl::InvalidNonZero => {
                    write!(f, "Difference was zero when a `NonZero*` type was specified")
                }
            }
        }
    }
}