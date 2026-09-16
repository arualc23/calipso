use super::*;

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

impl <Ind, T> Clone for IndexedBy<Ind, T> where Ind: ID, T: Clone {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone(), index: self.index }
    }
}