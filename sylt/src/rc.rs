use std::fmt;

pub struct Rc<T>(*mut (usize, T));

impl<T> Rc<T> {
    pub fn new(data: T) -> Self {
        Self(Box::into_raw(Box::new((1, data))))
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        unsafe {
            if (*self.0).0 == 1 {
                drop(Box::from_raw(self.0));
                return;
            }
            (*self.0).0 -= 1;
        }
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.0).0 += 1;
        }
        Self(self.0)
    }
}

impl<T> From<T> for Rc<T> {
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

impl<T> AsRef<T> for Rc<T> {
    fn as_ref(&self) -> &T {
        unsafe {
            &(*self.0).1
        }
    }
}

impl<T> std::ops::Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            &(*self.0).1
        }
    }
}

impl<T: std::cmp::PartialEq> std::cmp::PartialEq for Rc<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            (*self.0).1.eq(&(*other.0).1)
        }
    }
}

impl<T: std::cmp::Eq> std::cmp::Eq for Rc<T> {}

impl<T: std::cmp::PartialOrd> std::cmp::PartialOrd for Rc<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        unsafe {
            (*self.0).1.partial_cmp(&(*other.0).1)
        }
    }
}

impl<T: std::cmp::Ord> std::cmp::Ord for Rc<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        unsafe {
            (*self.0).1.cmp(&(*other.0).1)
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Rc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            (*self.0).1.fmt(f)
        }
    }
}

impl<T: fmt::Display> fmt::Display for Rc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            (*self.0).1.fmt(f)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Rc;
    #[test]
    fn new_empty() {
        let rc = Rc::new(());
        unsafe {
            assert_eq!((*rc.0).0, 1);
        }
    }

    #[test]
    fn clone() {
        let rc1 = Rc::new(());
        let rc2 = Rc::clone(&rc1);
        unsafe {
            assert_eq!((*rc1.0).0, 2);
            assert_eq!((*rc2.0).0, 2);
        }
    }

    #[test]
    fn explicit_drop() {
        let rc1 = Rc::new(());
        let rc2 = Rc::clone(&rc1);
        drop(rc1);
        unsafe {
            assert_eq!((*rc2.0).0, 1);
        }
    }

    #[test]
    fn implicit_drop() {
        let rc1 = Rc::new(());
        {
            let rc2 = Rc::clone(&rc1);
            unsafe {
                assert_eq!((*rc1.0).0, 2);
                assert_eq!((*rc2.0).0, 2);
            }
        }
        unsafe {
            assert_eq!((*rc1.0).0, 1);
        }
    }

    #[test]
    fn calls_drop() {
        use std::sync::atomic::{AtomicBool, Ordering};
        struct A<'a>(&'a AtomicBool);

        impl<'a> Drop for A<'a> {
            fn drop(&mut self) {
                // Panics if the value has been dropped before
                assert_eq!(
                    // Store true iff current value is false
                    self.0.compare_exchange(
                        false,
                        true,
                        Ordering::Acquire,
                        Ordering::Relaxed),
                    Ok(false)
                );
            }
        }

        let a = AtomicBool::new(false);
        let rc = Rc::new(A(&a));
        assert_eq!(a.load(Ordering::Relaxed), false);
        drop(rc);
        assert_eq!(a.load(Ordering::Relaxed), true);

        let a = AtomicBool::new(false);
        let rc1 = Rc::new(A(&a));
        let rc2 = Rc::clone(&rc1);
        assert_eq!(a.load(Ordering::Relaxed), false);
        drop(rc2);
        assert_eq!(a.load(Ordering::Relaxed), false);
        unsafe {
            assert_eq!((*rc1.0).0, 1);
        }
        drop(rc1);
        assert_eq!(a.load(Ordering::Relaxed), true);
    }

    #[test]
    fn from() {
        let rc = Rc::from(1usize);
        unsafe {
            assert_eq!((*rc.0).1, 1usize);
        }
    }

    #[test]
    fn deref() {
        let rc = Rc::new(1usize);
        assert_eq!(*rc, 1usize);
    }

    #[derive(Debug, PartialEq)]
    struct A(usize);

    #[test]
    fn partial_eq() {
        let rc = Rc::new(A(1usize));
        assert!(rc == Rc::new(A(1usize)));
    }

    #[test]
    fn as_ref() {
        let rc = Rc::new(A(1usize));
        assert_eq!(rc.as_ref(), &A(1usize));
    }

    #[test]
    fn partial_ord() {
        let rc1 = Rc::new(1usize);
        let rc2 = Rc::new(2usize);
        assert_eq!(rc1.partial_cmp(&rc2), 1usize.partial_cmp(&2usize));
        assert_eq!(rc2.partial_cmp(&rc1), 2usize.partial_cmp(&1usize));
        assert_eq!(rc1.partial_cmp(&rc1), 1usize.partial_cmp(&1usize));

        let f1 = Rc::new(std::f32::NAN);
        let f2 = Rc::new(1.0f32);
        assert_eq!(f1.partial_cmp(&f2), std::f32::NAN.partial_cmp(&1.0f32));
    }

    #[test]
    fn ord() {
        let rc1 = Rc::new(1usize);
        let rc2 = Rc::new(2usize);
        assert_eq!(rc1.cmp(&rc2), 1usize.cmp(&2usize));
        assert_eq!(rc2.cmp(&rc1), 2usize.cmp(&1usize));
        assert_eq!(rc1.cmp(&rc1), 1usize.cmp(&1usize));
    }
}
