use std::{fmt::{Debug, Display}, marker::PhantomData, ops::{Deref, DerefMut, Index, IndexMut}};

pub trait ID: Clone + Copy + PartialEq + PartialOrd + Eq + Ord + Increment + std::hash::Hash + Into<usize> + From<usize> + Display {}

#[macro_export]
macro_rules! id_derives {
    {$vis: vis $item:ident} => {
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Ord)]
        $vis struct $item(u32);
        impl Into<usize> for $item {
            fn into(self) -> usize {
                self.0 as usize
            }
        }

        impl From<usize> for $item {
            fn from(value: usize) -> Self {
                Self(value as u32)
            }
        }

        impl crate::id::Increment for $item {
            fn increment(&mut self) {
                self.0 += 1;
            }
        }

        impl std::fmt::Display for $item {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl crate::id::ID for $item {}
    };
}

pub trait Increment {
    fn increment(&mut self);
}

pub struct IdIterator<T> where T: ID {
    current: T,
    last: T,
}

impl<T> IdIterator<T> where T: ID {
    pub fn new(start: T, last: T) -> Self {
        Self { current: start, last }
    }
}

impl<T> Iterator for IdIterator<T> where T: ID {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.last { return None; }
        let res = self.current;
        self.current.increment();
        Some(res)
    }
}

pub struct IndexedBy<Ind, T> where Ind: ID {
    inner: Vec<T>,
    index: PhantomData<Ind>
}

impl<Ind, T> IndexedBy<Ind, T> where Ind: ID {
    /// #Safety
    /// Each inner's element index must match its ID. Failure to ensure this violates this struct's invariants and may lead to runtime panics.
    pub unsafe fn new(inner: Vec<T>) -> Self {
        Self {
            inner,
            index: PhantomData
        }
    }

    pub fn empty() -> Self {
        Self {
            inner: Vec::new(),
            index: PhantomData,
        }
    }

    pub fn filled(length: usize, value: T) -> Self where T: Clone {
        Self {
            inner: vec![value; length],
            index: PhantomData,
        }
    }

    pub fn filled_with(length: usize, generator: impl Fn() -> T) -> Self {
        let mut inner = Vec::with_capacity(length);
        for _ in 0..length {
            inner.push(generator());
        }
        Self {
            inner,
            index: PhantomData
        }
    }

    pub fn iter_ids(&self) -> IdIterator<Ind> {
        IdIterator { current: 0.into(), last: self.len().into() }
    }

    pub fn into_inner(self) -> Vec<T> {
        self.inner
    }

    pub fn get(&self, index: Ind) -> Option<&T> {
        self.inner.get(index.into())
    }

    pub fn into_iter_enumerated(self) -> std::iter::Zip<IdIterator<Ind>, std::vec::IntoIter<T>> {
        let upper = self.len().into();
        IdIterator::<Ind>::new(0.into(), upper).zip(self.into_iter())
    }
}

impl<Ind, T> Debug for IndexedBy<Ind, T> where Ind: ID, T: Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<Ind, T> Index<Ind> for IndexedBy<Ind, T> where Ind: ID {
    type Output = T;

    fn index(&self, index: Ind) -> &Self::Output {
        &self.inner[index.into()]
    }
}

impl<Ind, T> IndexMut<Ind> for IndexedBy<Ind, T> where Ind: ID {
    fn index_mut(&mut self, index: Ind) -> &mut Self::Output {
        &mut self.inner[index.into()]
    }
}

impl<Ind, T> Deref for IndexedBy<Ind, T> where Ind: ID {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Ind, T> DerefMut for IndexedBy<Ind, T> where Ind: ID {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<Ind, T> FromIterator<T> for IndexedBy<Ind, T> where Ind: ID {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self { inner: iter.into_iter().collect(), index: PhantomData }
    }
}

impl<Ind, T> IntoIterator for IndexedBy<Ind, T> where Ind: ID {
    type IntoIter = std::vec::IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}