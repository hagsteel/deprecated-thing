use std::mem::replace;
use std::ops::{Index, IndexMut};

use errors::{Result, Error};

#[derive(Debug)]
enum Entry<T> {
    Empty(usize),
    Occupied(T)
}

// -----------------------------------------------------------------------------
// 		- PreVec -
// -----------------------------------------------------------------------------
/// `PreVec`: a collection that allows inserts at a specific index with an optional offsets.
/// A `PreVec` can be created with either [`capacity`] or [`capacity_with_offset`].
///
/// This is useful with multiple collections where each index has to be unique, e.g having 
/// two collections of connections driven by the same [`Poll`] instance where the index represents
/// the connection token.
///
/// [`capacity`]: struct.PreVec.html#method.with_capacity
/// [`capacity_with_offset`]: struct.PreVec.html#method.with_capacity_and_offset
/// [`prevent_growth`]: struct.PreVec.html#method.prevent_growth
/// [`poll`]: ../struct.Poll.html
///
/// By default the collection can grow; To prevent overlap with another collection
/// call [`prevent_growth`]. 
/// This will result in an error if an insert occurrs outside the allocated space.
///
/// # Example
///
/// Creating two PreVecs sharing a unique range:
/// ```
/// use sonr::PreVec;
///
/// let mut lower = PreVec::with_capacity(1000);
/// let mut upper = PreVec::with_capacity_and_offset(1000, 1000);
///
/// let lower_index = lower.insert("foo").unwrap();
/// let upper_index = upper.insert("bar").unwrap();
///
/// assert_eq!(lower_index, 0);
/// assert_eq!(upper_index, 1000);
/// ```
///
/// Note that setting the offset to `std::usize::MAX` would mean only one 
/// element can be stored in the PreVec, and any attempt to insert more
/// would result in an error
#[derive(Debug)]
pub struct PreVec<T> {
    inner: Vec<Entry<T>>,
    capacity: usize,
    next: usize,
    offset: usize,
    length: usize,
    can_grow: bool,
}


// -----------------------------------------------------------------------------
// 		- Impl PreVec -
// -----------------------------------------------------------------------------
impl<T> PreVec<T> {
    /// Create a `PreVec` with a set capacity.
    /// Inserting above capacity will either allocate more space
    /// or return an error depending on wheter `enable_growth` or `prevent_growth` is called,
    /// 
    /// By default growth is enabled.
    ///
    /// # Example
    ///
    /// ```
    /// # use sonr::PreVec;
    /// let v = PreVec::<u32>::with_capacity(10);
    /// assert_eq!(v.capacity(), 10);
    /// ```
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
            capacity: cap,
            next: 0,
            offset: 0,
            length: 0,
            can_grow: true,
        }
    }

    /// Create a `PreVec` with a set capacity and offset.
    ///
    /// # Example
    ///
    /// ```
    /// # use sonr::PreVec;
    /// let mut v = PreVec::<u32>::with_capacity_and_offset(10, 5);
    /// assert_eq!(v.offset(), 5);
    ///
    /// let index = v.insert(5).unwrap();
    /// assert_eq!(index, 5);
    /// ```
    pub fn with_capacity_and_offset(cap: usize, offset: usize) -> Self {
        let mut p = Self::with_capacity(cap);
        p.set_offset(offset);
        p
    }

    /// Return the capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Return the offset
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Prevent inserting above the capacity.
    pub fn prevent_growth(&mut self) {
        self.can_grow = false;
    }

    /// Enable the collection to grow and allocate more space.
    pub fn enable_growth(&mut self) {
        self.can_grow = true;
    }

    /// Check if the index is within the range of the collection 
    /// (between capacity and offset)
    pub fn in_range(&self, index: usize) -> bool {
        index >= self.offset && index < self.capacity
    }

    /// Set the offset of the collection.
    pub fn set_offset(&mut self, new_offset: usize) {
        self.offset = new_offset;
    }

    /// Get an entry at a specific index
    ///
    /// # Example
    ///
    /// ```
    /// # use sonr::PreVec;
    /// let mut v = PreVec::with_capacity_and_offset(2, 10);
    /// v.insert(1u32);
    /// assert_eq!(v.get(10), Some(&1));
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        match self.inner.get(index - self.offset) {
            Some(Entry::Empty(_)) | None => None,
            Some(Entry::Occupied(ref v)) => Some(v)
        }
    }

    /// Get a mutable entry at a specific index
    ///
    /// # Example
    ///
    /// ```
    /// # use sonr::PreVec;
    /// let mut v = PreVec::with_capacity_and_offset(2, 10);
    /// v.insert("original".to_string());
    /// v.get_mut(10).map(|s| *s = "changed".to_string()).unwrap();
    /// assert_eq!(&v[10], "changed");
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        match self.inner.get_mut(index - self.offset) {
            Some(Entry::Empty(_)) | None => None,
            Some(Entry::Occupied(ref mut v)) => Some(v)
        }
    }

    fn grow_if_required(&mut self, index: usize) -> Result<()> {
        // Grow the inner vec up to the index
        if index >= self.inner.len() {
            // If the PreVec is not allowed to grow then 
            // return a NoCapacity error
            if index >= self.capacity && !self.can_grow {
                return Err(Error::NoCapacity);
            }

            // If it's less than capacity
            // then grow to accomodate the index
            let len = self.inner.len();
            if index < self.capacity {
                let mut inner: Vec<Entry<T>> = (len..self.capacity)
                    .map(|i| Entry::Empty(i+1))
                    .collect();
                self.inner.append(&mut inner);
            } else {
                self.capacity *= 2;
                let mut inner: Vec<Entry<T>> = (len..self.capacity)
                    .map(|i| Entry::Empty(i+1))
                    .collect();
                self.inner.append(&mut inner);
            }
        }

        Ok(())
    }

    /// Insert a value in the next available slot and return the slot index.
    /// Inserting above capacity will cause reallocation if and only if the `PreVec` can grow.
    ///
    /// # Example
    ///
    /// ```
    /// # use sonr::PreVec;
    /// let mut v = PreVec::with_capacity_and_offset(2, 10);
    /// let index = v.insert(1).unwrap();
    /// assert_eq!(index, 10);
    /// ```
    pub fn insert(&mut self, v: T) -> Result<usize> {
        let index = self.next;
        
        self.grow_if_required(index)?;

        let empty = replace(&mut self.inner[index], Entry::Occupied(v));
        match empty {
            Entry::Empty(next) => {
                self.length += 1;
                self.next = next;
            }
            Entry::Occupied(_) => panic!("attempted to insert into occupied slot: {}", index),
        }
        Ok(index + self.offset)
    }

    /// Remove at index (inserting an empty entry)
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if let Entry::Empty(_) = self.inner[index - self.offset] {
            return None;
        }

        let prev = replace(
            &mut self.inner[index - self.offset],
            Entry::Empty(self.next)
        );

        match prev {
            Entry::Occupied(v) => {
                self.length -= 1;
                self.next = index - self.offset;
                Some(v)
            }
            Entry::Empty(_) => None
        }
    }

    /// Number of occupied slots
    pub fn len(&self) -> usize {
        self.length
    }

    /// Returns `true` if the collection has no entries
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove all entries
    pub fn clear(&mut self) {
        self.inner = (0..self.capacity).map(|i| Entry::Empty(i+1)).collect();
        self.next = 0;
        self.length = 0;
    }
}


// -----------------------------------------------------------------------------
// 		- impl Index -
// -----------------------------------------------------------------------------
impl<T> Index<usize> for PreVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        match &self.inner[index - self.offset] {
            Entry::Occupied(v) => v,
            Entry::Empty(_) => panic!("invalid index"),
        }
    }
}

// -----------------------------------------------------------------------------
// 		- impl IndexMut -
// -----------------------------------------------------------------------------
impl<T> IndexMut<usize> for PreVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        match &mut self.inner[index - self.offset] {
            Entry::Occupied(v) => v,
            Entry::Empty(_) => panic!("invalid index: {}", index),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Foo {
        a: usize,
        b: usize,
    }

    fn values(cap: usize) -> Vec<Foo> {
        vec![Foo { a: 1, b: 2}; cap]
    }

    #[test]
    fn remove_with_offset() -> Result<()> {
        let mut v = PreVec::with_capacity_and_offset(10, 1);
        assert_eq!(v.insert(0)?, 1);
        assert_eq!(v.insert(1)?, 2);
        assert_eq!(v.insert(2)?, 3);
        assert_eq!(v.remove(1), Some(0));
        assert_eq!(v.remove(1), None);
        assert_eq!(v.insert(1)?, 1);
        Ok(())
    }

    #[test]
    fn insert_get_index() {
        let mut v = PreVec::with_capacity(10);
        assert_eq!(v.insert("foo").unwrap(), 0);
        assert_eq!(v.insert("foo").unwrap(), 1);
    }

    #[test]
    fn get_mut() {
        let mut v = PreVec::with_capacity(1);
        v.insert("foo").unwrap();
        let x = &mut v[0];
        assert_eq!(x, &"foo");
    }

    #[test]
    fn remove() {
        let mut v = PreVec::with_capacity(10);
        assert_eq!(v.insert("foo").unwrap(), 0);
        v.remove(0);
        assert_eq!(v.insert("foo").unwrap(), 0);
    }

    #[test]
    fn insert_many() {
        let cap = 10;
        let values = values(cap);
        let mut store = PreVec::with_capacity(cap);

        // Add twice as many elements forcing 
        // a resize
        for _ in 0..cap * 2 {
            for v in &values {
                store.insert(v);
            }

            for v in &values {
                let _v = store[v.a];
            }
        }

        // Make sure the resize works
        let x = store.remove(15).unwrap();
        assert_eq!(store.insert(x).unwrap(), 15);
    }


    #[test]
    fn insert_with_offset() {
        let mut store = PreVec::with_capacity(2);
        store.set_offset(10);

        let index = store.insert(1u32).unwrap();
        assert_eq!(index, 10);
        let index = store.insert(1u32).unwrap();
        assert_eq!(index, 11);

    }

    #[test]
    fn removing_with_offset_returns_correct_value() {
        let mut store = PreVec::with_capacity(2);
        store.set_offset(10);

        let index = store.insert(1u32).unwrap();
        assert_eq!(index, 10);
        let val = store.remove(index).unwrap();
        assert_eq!(val, 1u32);
    }

    #[test]
    fn get_with_offset() {
        let mut store = PreVec::with_capacity(2);
        store.set_offset(10);

        store.insert(1u32);
        store.insert(2u32);

        assert_eq!(*store.get(10).unwrap(), 1);
        assert_eq!(*store.get(11).unwrap(), 2);
    }

    #[test]
    fn get_mut_with_offset() {
        let mut store = PreVec::with_capacity(2);
        store.set_offset(10);

        store.insert(1u32);
        store.insert(2u32);

        assert_eq!(*store.get_mut(10).unwrap(), 1);
        assert_eq!(*store.get_mut(11).unwrap(), 2);
    }

    #[test]
    fn index_with_offset() {
        let mut store = PreVec::with_capacity(2);
        store.set_offset(10);
        store.insert(1);
        store.insert(2);

        assert_eq!(*store.get(10).unwrap(), 1);
        assert_eq!(*store.get(11).unwrap(), 2);
    }

    #[test]
    fn max_index_with_offset() {
        use std::usize::MAX as MAX_USIZE;
        let mut store = PreVec::with_capacity_and_offset(1, MAX_USIZE);
        let index = store.insert(1u32).unwrap();
        assert_eq!(index, MAX_USIZE);
    }

    #[test]
    fn in_range() {
        let store: PreVec<u32> = PreVec::with_capacity(100);

        assert!(store.in_range(0));
        assert!(store.in_range(99));
        assert!(!store.in_range(100));
    }

    #[test]
    fn len_after_remove() {
        let mut store: PreVec<u32> = PreVec::with_capacity(100);
        store.insert(10);
        store.remove(0);
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn allow_growth() {
        let mut store: PreVec<u32> = PreVec::with_capacity(1);
        store.insert(1);
        store.insert(1);

        assert_eq!(store.capacity, 2);
    }

    #[test]
    fn disable_growth() {
        let mut store: PreVec<u32> = PreVec::with_capacity(1);
        store.prevent_growth();
        assert_eq!(store.insert(1).unwrap(), 0);
        match store.insert(1) {
            Err(Error::NoCapacity) => {}
            _ => panic!("Should return a NoCapacity error")
        }

        assert_eq!(store.capacity, 1);
    }
}
