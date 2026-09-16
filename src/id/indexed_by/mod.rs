use super::*;

mod trait_impls;

// #[derive(Debug)]
pub struct IndexedBy<Ind, T> where Ind: ID {
    inner: Vec<T>,
    index: PhantomData<Ind>
}

impl<Ind, T> IndexedBy<Ind, T> where Ind: ID {
    /// # Safety
    /// Each inner's element index must match its ID. Failure to ensure this violates this struct's invariants and may lead to runtime panics.
    pub unsafe fn new(inner: Vec<T>) -> Self {
        Self {
            inner,
            index: PhantomData
        }
    }

    // pub const unsafe fn new<N: const usize>(inner: [T; N]) -> 

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
        IdIterator { current: 0.into(), last: self.inner.len().into() }
    }

    pub fn into_inner(self) -> Vec<T> {
        self.inner
    }

    // pub fn clone_inner(&self) -> Vec<T> where T: Clone {
    //     self.inner.clone()
    // }

    pub fn get(&self, index: Ind) -> Option<&T> {
        self.inner.get(index.into())
    }

    pub fn get_mut(&mut self, index: Ind) -> Option<&mut T> {
        self.inner.get_mut(index.into())
    }

    pub fn into_iter_enumerated(self) -> std::iter::Zip<IdIterator<Ind>, std::vec::IntoIter<T>> {
        let upper = self.inner.len().into();
        IdIterator::<Ind>::new(0.into(), upper).zip(self.into_iter())
    }

    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.inner.iter_mut()
    }

    pub fn iter_enumerated(&self) -> std::iter::Zip<IdIterator<Ind>, core::slice::Iter<'_, T>> {
        let upper = self.inner.len().into();
        IdIterator::<Ind>::new(0.into(), upper).zip(self.iter())
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn chunks_mut(&mut self, chunk_size: usize) -> core::slice::ChunksMut<'_, T> {
        self.inner.chunks_mut(chunk_size)
    }
}