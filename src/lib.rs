//! `OptionVec<T>`; an abstraction over `Vec<Option<T>>`
//!
//! An element in an `OptionVec<T>` can be accessed by index and maintains
//! its position when elements are removed from the container.
//!
//! An element inserted into an `OptionVec<T>` will occupy the first available
//! position in the container.

#![deny(missing_docs)]

use std::cmp::Ordering;
use std::fmt;
use std::iter::FromIterator;
use std::ops;
use std::slice;
use std::vec;

/// An abstraction over `Vec<Option<T>>`
///
/// An element in an `OptionVec<T>` can be accessed by index and maintains
/// its position when elements are removed from the container.
///
/// An element inserted into an `OptionVec<T>` will occupy the first available
/// position in the container.
pub struct OptionVec<T> {
    vec: Vec<Option<T>>,
}

impl<T> OptionVec<T> {
    /// Creates an empty `OptionVec<T>`.
    #[inline]
    pub fn new() -> OptionVec<T> {
        OptionVec::with_capacity(0)
    }

    /// Creates an empty `OptionVec<T>` with capacity for `n` elements.
    #[inline]
    pub fn with_capacity(n: usize) -> OptionVec<T> {
        OptionVec{
            vec: Vec::with_capacity(n),
        }
    }

    /// Returns a borrowed reference to the internal `Vec<Option<T>>`.
    #[inline]
    pub fn inner(&self) -> &Vec<Option<T>> {
        &self.vec
    }

    /// Returns a mutable reference to the internal `Vec<Option<T>>`.
    ///
    /// Modification of this internal container is safe, but using methods
    /// such as `Vec::insert` or `Vec::remove` will invalidate existing indices.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut Vec<Option<T>> {
        &mut self.vec
    }

    /// Returns the allocated capacity for elements.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    /// Returns the number of contained elements.
    ///
    /// This operation is `O(N)`, as all non-`None` elements must be individually
    /// counted.
    #[inline]
    pub fn len(&self) -> usize {
        self.vec.iter().filter(|v| v.is_some()).count()
    }

    /// Returns whether the container is empty.
    ///
    /// This operation is `O(N)` worst-case, as any elements must be searched
    /// for a non-`None` element.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vec.iter().all(|v| v.is_none())
    }

    /// Inserts an element into the first available position, returning the
    /// destination position.
    #[inline]
    pub fn insert(&mut self, t: T) -> usize {
        if let Some(pos) = self.first_vacant() {
            self.vec[pos] = Some(t);
            pos
        } else {
            self.push(t)
        }
    }

    /// Removes an element from the given position, if one exists.
    #[inline]
    pub fn remove(&mut self, idx: usize) -> Option<T> {
        self.vec.get_mut(idx).and_then(|v| v.take())
    }

    /// Reserves capacity for at least `n` more elements.
    pub fn reserve(&mut self, n: usize) {
        let rem_cap = self.capacity() - self.len();

        if rem_cap < n {
            self.vec.reserve(n - rem_cap);
        }
    }

    /// Reserves capacity for exactly `n` more elements.
    pub fn reserve_exact(&mut self, n: usize) {
        let rem_cap = self.capacity() - self.len();

        if rem_cap < n {
            self.vec.reserve_exact(n - rem_cap);
        }
    }

    /// Shrinks the allocation as much as possible.
    ///
    /// Any trailing `None` elements will be truncated. `None` elements in
    /// internal positions are not removed, so as to maintain `Some(_)` element
    /// positions.
    pub fn shrink_to_fit(&mut self) {
        let n = self.end_occupied();

        self.vec.truncate(n);
        self.vec.shrink_to_fit();
    }

    /// Retains only elements specified by the predicate.
    ///
    /// All elements `e` such that `f(&mut e)` returns `false` will be assigned
    /// to `None`.
    pub fn retain<F>(&mut self, mut f: F)
            where F: FnMut(&mut T) -> bool {
        for v in &mut self.vec {
            let retain = match *v {
                Some(ref mut inner) => f(inner),
                None => true
            };

            if !retain {
                *v = None;
            }
        }
    }

    /// Removes and returns the last occupied element.
    #[inline]
    pub fn pop_back(&mut self) -> Option<T> {
        if let Some(pos) = self.last_occupied() {
            self.remove(pos)
        } else {
            None
        }
    }

    /// Removes and returns the first occupied element.
    #[inline]
    pub fn pop_front(&mut self) -> Option<T> {
        if let Some(pos) = self.first_occupied() {
            self.remove(pos)
        } else {
            None
        }
    }

    /// Removes all contained elements.
    #[inline]
    pub fn clear(&mut self) {
        self.vec.clear();
    }

    /// Returns whether an element exists at the given index.
    #[inline]
    pub fn contains(&self, idx: usize) -> bool {
        self.vec.get(idx).map_or(false, |v| v.is_some())
    }

    /// Returns an element at the given position.
    #[inline]
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.vec.get(idx).and_then(|v| v.as_ref())
    }

    /// Returns a mutable reference to an element at the given position.
    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.vec.get_mut(idx).and_then(|v| v.as_mut())
    }

    /// Returns an iterator over contained elements.
    #[inline]
    pub fn iter(&self) -> Iter<T> {
        Iter(self.vec.iter())
    }

    /// Returns an iterator over mutable references to contained elements.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut(self.vec.iter_mut())
    }

    fn first_vacant(&self) -> Option<usize> {
        for (i, v) in self.vec.iter().enumerate() {
            if v.is_none() {
                return Some(i);
            }
        }
        None
    }

    fn first_occupied(&self) -> Option<usize> {
        for (i, v) in self.vec.iter().enumerate() {
            if v.is_some() {
                return Some(i);
            }
        }
        None
    }

    fn end_occupied(&self) -> usize {
        self.last_occupied().map_or(0, |n| n + 1)
    }

    fn last_occupied(&self) -> Option<usize> {
        for (i, v) in self.vec.iter().enumerate().rev() {
            if v.is_some() {
                return Some(i);
            }
        }
        None
    }

    fn push(&mut self, t: T) -> usize {
        let n = self.vec.len();
        self.vec.push(Some(t));
        n
    }
}

/// An owned iterator of `OptionVec<T>` elements.
pub struct IntoIter<T>(vec::IntoIter<Option<T>>);

/// An iterator of borrowed `OptionVec<T>` elements.
#[derive(Clone)]
pub struct Iter<'a, T: 'a>(slice::Iter<'a, Option<T>>);

/// An iterator of mutable `OptionVec<T>` elements.
#[derive(Debug)]
pub struct IterMut<'a, T: 'a>(slice::IterMut<'a, Option<T>>);

macro_rules! option_vec_iter {
    ( $name:ident , $r:ty , $pat:pat , $v:ident ) => {
        impl<'a, T: 'a> Iterator for $name<'a, T> {
            type Item = $r;

            fn next(&mut self) -> Option<$r> {
                while let Some(v) = self.0.next() {
                    if let Some($pat) = *v {
                        return Some($v);
                    }
                }

                None
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let (_, max) = self.0.size_hint();
                (0, max)
            }
        }

        impl<'a, T: 'a> DoubleEndedIterator for $name<'a, T> {
            fn next_back(&mut self) -> Option<$r> {
                while let Some(v) = self.0.next_back() {
                    if let Some($pat) = *v {
                        return Some($v);
                    }
                }

                None
            }
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        while let Some(v) = self.0.next() {
            if v.is_some() {
                return v;
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, max) = self.0.size_hint();
        (0, max)
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        while let Some(v) = self.0.next_back() {
            if v.is_some() {
                return v;
            }
        }

        None
    }
}

option_vec_iter!{ Iter, &'a T, ref v, v }
option_vec_iter!{ IterMut, &'a mut T, ref mut v, v }

impl<T: fmt::Debug> fmt::Debug for IntoIter<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("IntoIter")
            .field(&self.0.as_slice())
            .finish()
    }
}

impl<'a, T: 'a + fmt::Debug> fmt::Debug for Iter<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("IntoIter")
            .field(&self.0.as_slice())
            .finish()
    }
}

impl<T: fmt::Debug> fmt::Debug for OptionVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entries(self.vec.iter()
                .enumerate().filter(|&(_idx, v)| v.is_some()))
            .finish()
    }
}

impl<T: Clone> Clone for OptionVec<T> {
    fn clone(&self) -> OptionVec<T> {
        let end = self.end_occupied();

        OptionVec::from(self.vec[..end].to_vec())
    }

    fn clone_from(&mut self, other: &OptionVec<T>) {
        let end = other.end_occupied();

        self.vec.truncate(end);
        let len = self.vec.len();

        self.vec.clone_from_slice(&other.vec[..len]);
        self.vec.extend_from_slice(&other.vec[len..end]);
    }
}

impl<T> Default for OptionVec<T> {
    fn default() -> OptionVec<T> {
        OptionVec::new()
    }
}

impl<T> From<Vec<Option<T>>> for OptionVec<T> {
    fn from(v: Vec<Option<T>>) -> OptionVec<T> {
        OptionVec{vec: v}
    }
}

impl<T> Into<Vec<Option<T>>> for OptionVec<T> {
    fn into(self) -> Vec<Option<T>> {
        self.vec
    }
}

impl<T> Extend<T> for OptionVec<T> {
    fn extend<I>(&mut self, iter: I) where I: IntoIterator<Item=T> {
        let iter = iter.into_iter();

        let (low, _) = iter.size_hint();
        self.reserve(low);

        for v in iter {
            self.insert(v);
        }
    }
}

impl<T> FromIterator<T> for OptionVec<T> {
    fn from_iter<I>(iter: I) -> OptionVec<T> where I: IntoIterator<Item=T> {
        OptionVec{vec: iter.into_iter().map(Some).collect()}
    }
}

macro_rules! impl_eq {
    ( $rhs:ty ) => {
        impl<'b, A, B> PartialEq<$rhs> for OptionVec<A> where A: PartialEq<B> {
            #[inline]
            fn eq(&self, rhs: &$rhs) -> bool { self.iter().eq(rhs.iter()) }
            #[inline]
            fn ne(&self, rhs: &$rhs) -> bool { self.iter().ne(rhs.iter()) }
        }
    }
}

impl_eq!{ OptionVec<B> }
impl_eq!{ Vec<B> }
impl_eq!{ &'b [B] }

impl<T> Eq for OptionVec<T> where T: Eq {}

impl<T> PartialOrd for OptionVec<T> where T: PartialOrd {
    #[inline]
    fn partial_cmp(&self, rhs: &OptionVec<T>) -> Option<Ordering> {
        self.iter().partial_cmp(rhs.iter())
    }

    #[inline]
    fn lt(&self, rhs: &OptionVec<T>) -> bool { self.iter().lt(rhs.iter()) }
    #[inline]
    fn le(&self, rhs: &OptionVec<T>) -> bool { self.iter().le(rhs.iter()) }
    #[inline]
    fn gt(&self, rhs: &OptionVec<T>) -> bool { self.iter().gt(rhs.iter()) }
    #[inline]
    fn ge(&self, rhs: &OptionVec<T>) -> bool { self.iter().ge(rhs.iter()) }
}

impl<T> Ord for OptionVec<T> where T: Ord {
    #[inline]
    fn cmp(&self, rhs: &OptionVec<T>) -> Ordering {
        self.iter().cmp(rhs.iter())
    }
}

impl<T> ops::Index<usize> for OptionVec<T> {
    type Output = T;

    #[inline]
    fn index(&self, idx: usize) -> &T {
        self.vec[idx].as_ref().unwrap_or_else(|| panic!("index {} is empty", idx))
    }
}

impl<T> ops::IndexMut<usize> for OptionVec<T> {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut T {
        self.vec[idx].as_mut().unwrap_or_else(|| panic!("index {} is empty", idx))
    }
}

impl<T> IntoIterator for OptionVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> IntoIter<T> {
        IntoIter(self.vec.into_iter())
    }
}

impl<'a, T> IntoIterator for &'a OptionVec<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut OptionVec<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

#[cfg(test)]
mod test {
    use super::OptionVec;

    #[test]
    fn test_len() {
        let v = OptionVec::from(vec![
            None, Some("foo"), None, Some("bar"), None]);

        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_insert() {
        let mut v = OptionVec::from(vec![
            Some(()), None, Some(())]);

        assert_eq!(v.len(), 2);

        assert_eq!(v.insert(()), 1);
        assert_eq!(v.len(), 3);

        assert_eq!(v.insert(()), 3);
        assert_eq!(v.len(), 4);
    }

    #[test]
    fn test_remove() {
        let mut v = OptionVec::from(vec![
            Some(1), Some(2), Some(3)]);

        assert_eq!(v.remove(0), Some(1));
        assert_eq!(v.remove(0), None);
    }

    #[test]
    fn test_retain() {
        let mut v = OptionVec::from(vec![
            Some(1), Some(2), Some(3)]);

        v.retain(|n| *n >= 2);

        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_clone() {
        let a = OptionVec::from(vec![
            Some(1), None, Some(2), None]);

        let b = a.clone();

        let mut c = OptionVec::new();
        c.clone_from(&a);

        let mut d = OptionVec::from(vec![Some(0); 10]);
        d.clone_from(&a);

        assert_eq!(a.len(), 2);
        assert_eq!(b.len(), 2);
        assert_eq!(c.len(), 2);
        assert_eq!(d.len(), 2);

        assert_eq!(a.inner().len(), 4);
        assert_eq!(b.inner().len(), 3);
        assert_eq!(c.inner().len(), 3);
        assert_eq!(d.inner().len(), 3);
    }

    #[test]
    fn test_shrink_to_fit() {
        let mut v = OptionVec::from(vec![
            Some(1), None, Some(2), None, None]);

        assert_eq!(v.len(), 2);
        assert_eq!(v.inner().len(), 5);

        v.shrink_to_fit();

        assert_eq!(v.len(), 2);
        assert_eq!(v.inner().len(), 3);
    }

    #[test]
    fn test_pop_back() {
        let mut v = OptionVec::from(vec![
            Some(1), Some(2)]);

        assert_eq!(v.pop_back(), Some(2));
        assert_eq!(v.pop_back(), Some(1));
        assert_eq!(v.pop_back(), None);
    }

    #[test]
    fn test_pop_front() {
        let mut v = OptionVec::from(vec![
            Some(1), Some(2)]);

        assert_eq!(v.pop_front(), Some(1));
        assert_eq!(v.pop_front(), Some(2));
        assert_eq!(v.pop_front(), None);
    }

    #[test]
    fn test_into_iter() {
        let v = OptionVec::from(vec![
            None, Some(1), Some(2), None, Some(3), None]);

        let mut iter = v.into_iter();

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter() {
        let v = OptionVec::from(vec![
            None, Some(1), Some(2), None, Some(3), None]);

        let mut iter = v.iter();

        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_mut() {
        let mut v = OptionVec::from(vec![
            None, Some(1), Some(2), None, Some(3), None]);

        for i in &mut v {
            *i *= 2;
        }

        let mut iter = v.iter();

        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), Some(&6));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_debug() {
        let mut v = OptionVec::from(vec![Some(1)]);
        let _ = format!("{:?}", v);
        let _ = format!("{:?}", v.iter());
        let _ = format!("{:?}", v.iter_mut());
        let _ = format!("{:?}", v.into_iter());
    }

    #[test]
    fn test_eq() {
        let a = OptionVec::from(vec![Some(1), None, Some(2)]);
        let b = OptionVec::from(vec![None, Some(1), Some(2), None]);

        assert_eq!(a, b);
    }
}
