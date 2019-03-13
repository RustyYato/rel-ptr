pub(super) union MaybeUninit<T: Copy> {
    value: T,
    null: ()
}

impl<T: Copy> Copy for MaybeUninit<T> {}
impl<T: Copy> Clone for MaybeUninit<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Copy> MaybeUninit<T> {
    pub fn null() -> Self {
        Self { null: () }
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
    }

    pub unsafe fn get(self) -> T {
        self.value
    }
}