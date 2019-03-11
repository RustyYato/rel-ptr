use super::*;

struct SelfRef<T, U: ?Sized + MetaData> {
    t_ref: RelPtr<U, i8>,
    t: T,
}

fn id<T>(t: &T) -> &T {
    t
}

impl<T, U: ?Sized + MetaData> SelfRef<T, U> {
    pub fn new(t: T, f: fn(&T) -> &U) -> Self {
        let mut this = Self {
            t: t.into(),
            t_ref: RelPtr::null(),
        };

        this.t_ref.set(f(&this.t)).unwrap();

        this
    }

    pub fn t(&self) -> &T {
        &self.t
    }

    pub fn t_mut(&mut self) -> &mut T {
        &mut self.t
    }

    pub fn t_ref(&self) -> &U {
        unsafe { self.t_ref.as_ref_unchecked() }
    }

    #[allow(unused)]
    pub fn t_ref_mut(&mut self) -> &mut U {
        unsafe { self.t_ref.as_mut_unchecked() }
    }
}

#[inline(never)]
fn block_opt<T>(x: T) -> T {
    x
}

#[test]
fn simple_test() {
    let mut s = SelfRef {
        t: "Hello World",
        t_ref: RelPtr::null(),
    };

    s.t_ref.set(&s.t).unwrap();

    assert_eq!(s.t(), s.t_ref());
    assert_eq!(*s.t(), "Hello World");
    assert_eq!(*s.t_ref(), "Hello World");
}

#[test]
fn simple_move() {
    let mut s = SelfRef {
        t: "Hello World",
        t_ref: RelPtr::null(),
    };

    s.t_ref.set(&s.t).unwrap();

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
    let s = SelfRef::new("Hello World", id);

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
    let mut s = SelfRef::new("Hello World", id);
    let mut x = SelfRef::new("Killer Move", id);

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
    let mut s = SelfRef::new("Hello World", id);

    assert_eq!(s.t(), s.t_ref());

    *s.t_mut() = "Killer Move";

    assert_eq!(*s.t(), "Killer Move");
    assert_eq!(*s.t_ref(), "Killer Move");
}

#[test]
fn sub_str() {
    #[inline(never)]
    fn get_move(s: SelfRef<[u8; 5], [u8]>) {
        assert_eq!(*s.t(), [0, 1, 2, 3, 4]);
        assert_eq!(*s.t_ref(), [2, 3, 4]);
    }

    let s = SelfRef::new([0, 1, 2, 3, 4], |x| &x[2..]);

    assert_eq!(*s.t(), [0, 1, 2, 3, 4]);
    assert_eq!(*s.t_ref(), [2, 3, 4]);

    get_move(s);
}

#[test]
fn check_copy() {
    fn is_copy<T: Copy>() {}

    fn check<T: ?Sized + MetaData, I: Delta>() {
        is_copy::<RelPtr<T, I>>;
    }
}

#[cfg(feature = "nightly")]
mod nightly {
    use super::*;

    fn check_trait_object_simple() {
        use std::fmt::Display;

        let s = unsafe {
            SelfRef::<[u8; 5], TraitObject<dyn PartialEq<[u8]>>>::new([0, 1, 2, 3, 4], |x| {
                TraitObject::new(x)
            })
        };

        assert_eq!(*s.t(), [0, 1, 2, 3, 4]);

        let eq: &[u8] = &[2, 3, 4];
        assert!(unsafe { s.t_ref().into() } == eq);
    }

    #[cfg(feature = "no_std")]
    fn check_trait_object_after_move() {
        use std::fmt::Display;

        let s = unsafe {
            SelfRef::<[u8; 5], TraitObject<dyn PartialEq<[u8]>>>::new([0, 1, 2, 3, 4], |x| {
                TraitObject::new(x)
            })
        };

        assert_eq!(*s.t(), [0, 1, 2, 3, 4]);

        let eq: &[u8] = &[2, 3, 4];
        assert!(unsafe { s.t_ref().into() } == eq);

        #[inline(never)]
        fn force_move<T>(t: T) -> T {
            t
        }

        let s = force_move(s);

        assert_eq!(*s.t(), [0, 1, 2, 3, 4]);

        let eq: &[u8] = &[2, 3, 4];
        assert!(unsafe { s.t_ref().into() } == eq);
    }

    #[cfg(not(feature = "no_std"))]
    fn check_trait_object_after_move() {
        use std::fmt::Display;

        let s = unsafe {
            SelfRef::<[u8; 5], TraitObject<dyn PartialEq<[u8]>>>::new([0, 1, 2, 3, 4], |x| {
                TraitObject::new(x)
            })
        };

        assert_eq!(*s.t(), [0, 1, 2, 3, 4]);

        let eq: &[u8] = &[2, 3, 4];
        assert!(unsafe { s.t_ref().into() } == eq);

        let s = Box::new(s);

        assert_eq!(*s.t(), [0, 1, 2, 3, 4]);

        let eq: &[u8] = &[2, 3, 4];
        assert!(unsafe { s.t_ref().into() } == eq);
    }
}