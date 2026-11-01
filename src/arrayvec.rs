use core::fmt::{self, Debug};
use core::hash::{self, Hash};
use core::hint::assert_unchecked;
use core::mem::{MaybeUninit, transmute};
use core::ops::{Deref, DerefMut};

#[derive(Clone)]
pub(crate) struct ArrayVec<T: Clone + Copy, const CAP: usize> {
    data: [MaybeUninit<T>; CAP],
    len: usize,
}

impl<T: Clone + Copy, const CAP: usize> ArrayVec<T, CAP> {
    pub const fn new() -> Self {
        Self {
            data: [MaybeUninit::uninit(); CAP],
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn as_slice(&self) -> &[T] {
        unsafe { transmute(self.data.split_at(self.len).0) }
    }

    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { transmute(self.data.split_at_mut(self.len).0) }
    }

    pub const fn push(&mut self, value: T) {
        assert!(self.len < CAP, "cannot push into a full ArrayVec");
        unsafe {
            self.push_unchecked(value);
        }
    }

    pub const unsafe fn push_unchecked(&mut self, value: T) {
        unsafe {
            assert_unchecked(self.len < CAP);
        }
        self.data[self.len].write(value);
        self.len += 1;
    }

    pub const fn clear(&mut self) {
        self.len = 0;
    }
}

impl<T: Clone + Copy, const CAP: usize> Deref for ArrayVec<T, CAP> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T: Clone + Copy, const CAP: usize> DerefMut for ArrayVec<T, CAP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T: Clone + Copy + PartialEq, const CAP: usize> PartialEq for ArrayVec<T, CAP> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Clone + Copy + Hash, const CAP: usize> Hash for ArrayVec<T, CAP> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<T: Clone + Copy + Debug, const CAP: usize> Debug for ArrayVec<T, CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}
