use super::*;

use std::fmt::*;

impl<T: ?Sized + MetaData, I: Debug + Delta> Pointer for RelPtr<T, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:p}({:?})", self, self.0)
    }
}

impl<T: ?Sized + MetaData, I: Debug + Delta> Debug for RelPtr<T, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("RelPtr")
            .field("ptr", &(self as *const Self))
            .field("offset", &self.0)
            .finish()
    }
}
