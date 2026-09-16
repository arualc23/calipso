use std::{fmt::{Debug, Display}, marker::PhantomData, ops::{Index, IndexMut}};

mod indexed_by;
pub use indexed_by::IndexedBy;

pub trait ID: Clone + Copy + PartialEq + PartialOrd + Eq + Ord + Increment + std::hash::Hash + Into<usize> + From<usize> + Display {}

#[macro_export]
macro_rules! id_derives {
    {$vis: vis $item:ident} => {
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Ord, serde::Serialize, serde::Deserialize)]
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

// /// first field is indexe tried, second is length
// pub struct IndexError(usize, usize);
// impl Debug for IndexError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "tried to access index {}, but the length is {}", self.0, self.1)
//     }
// }

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



