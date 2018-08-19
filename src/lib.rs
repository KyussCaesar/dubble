//! A generic double-buffer.
//!
//! # Description
//!
//! A generic double-buffer for anything that implements `Clone`.
//!
//! # Usage
//!
//! # Notes
//!
//! ## `Default`
//!
//! `DoubleBuffered` implements `Default` so long as the type being buffered 
//! implements it. For example, if you needed a double-buffered `i32`:
//!
//! ```
//! use double_buffered::DoubleBuffered;
//! let db: DoubleBuffered<i32> = DoubleBuffered::default();
//! ```
//!
//! ## `Deref` and `DerefMut`
//!
//! An important note on the `impl`s for each of these is that `Deref` will 
//! dereference to the *read* buffer while `DerefMut` will dereference to the
//! *write* buffer. So, if you did `*my_buf = 3` followed by `assert(*my_buf == 3)`,
//! you'd find that the assertion would fail.
//!
//! In other words, `Deref` behaves as if you had called `my_buf.read()`, and
//! `DerefMut` behaves as if you had called `my_buf.write()`.
//!

use std::ops::{Deref, DerefMut};

/// Represents something that is double-buffered.
pub struct DoubleBuffered<T: Clone>
{
    rbuf: T,
    wbuf: T,
}

impl<T: Clone> DoubleBuffered<T>
{
    /// Uses `constructor` to construct each buffer. It's handy to pass things
    /// like `Vec::new` into here. `DoubleBuffered` also implements default
    /// if the wrapped type does, so you could also do
    /// `DoubleBuffered<Vec<T>>::default()`
    pub fn construct_with<F: Fn() -> T>(constructor: F) -> Self
    {
        Self
        {
            rbuf: constructor(),
            wbuf: constructor(),
        }
    }

    /// Initialises the double-buffer with the value. Both buffers are initialised
    /// with the same value.
    pub fn init_with(value: T) -> Self
    {
        Self
        {
            rbuf: value.clone(),
            wbuf: value.clone(),
        }
    }

    /// Returns an immutable reference to the active buffer.
    pub fn read(&self) -> &T
    {
        &self.rbuf
    }

    /// Returns a mutable reference to the *inactive* buffer.
    /// Note that changes made through this reference will not be reflected
    /// until after `update` is called.
    ///
    /// This might seem a little weird; "why not just go `my_buf.write(stuff)`"?.
    /// The reason is so that you can update the elements of a collection without
    /// having to build a clone of the collection. For example:
    ///
    /// ```rust
    /// # use double_buffered::DoubleBuffered;
    /// let mut my_buf: DoubleBuffered<Vec<i32>> = DoubleBuffered::default();
    /// let mut wb = my_buf.write();
    /// wb.push(4);
    /// ```
    ///
    /// Compared to the other form:
    ///
    /// ```rust,not_run
    /// # use double_buffered::DoubleBuffered;
    /// # let mut my_buf: DoubleBuffered<Vec<i32>> = DoubleBuffered::default();
    /// let mut wb = my_buf.read().clone();
    /// wb.push(5);
    /// // my_buf.write(wb);
    /// ```
    ///
    /// Notice that you have to create a copy and modify it.
    pub fn write(&mut self) -> &mut T
    {
        &mut self.wbuf
    }

    /// Copies the write buffer into the read buffer.
    pub fn update(&mut self)
    {
        self.rbuf = self.wbuf.clone();
    }

    /// Writes the value to the write buffer, and then immediately updates the
    /// read buffer.
    pub fn upsert(&mut self, value: T)
    {
        *self.write() = value;
        self.update();
    }

    /// Returns the read buffer. This does not update the read buffer with the
    /// contents of the write buffer beforehand.
    pub fn unbuffer_read(self) -> T
    {
        self.rbuf
    }

    /// Returns the write buffer.
    pub fn unbuffer_write(self) -> T
    {
        self.wbuf
    }
}

impl<T: Default + Clone> Default for DoubleBuffered<T>
{
    /// Use the default constructor for the type.
    fn default() -> Self
    {
        Self::construct_with(T::default)
    }
}

impl<T: Clone> Deref for DoubleBuffered<T>
{
    type Target = T;

    fn deref(&self) -> &T
    {
        self.read()
    }
}

impl<T: Clone> DerefMut for DoubleBuffered<T>
{
    fn deref_mut(&mut self) -> &mut T
    {
        self.write()
    }
}


#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn basic_int()
    {
        // create a db int
        let mut db = DoubleBuffered::<i32>::default();

        *db.write() = 3;
        // read buffer should not update until told to do so.
        assert!(*db.read() == 0);
        db.update();
        assert!(*db.read() == 3);

        // check the same thing again
        *db.write() = 4;
        assert!(*db.read() == 3);
        db.update();
        assert!(*db.read() == 4);
    }

    #[test]
    fn basic_string()
    {
        let mut db = DoubleBuffered::construct_with(String::new);
        assert!(*db.read() == String::new());
        *db.write() = "hello, world".to_string();
        db.update();
        assert!(*db.read() == String::from("hello, world"));
    }

    #[test]
    fn basic_int_using_deref()
    {
        // the same test as basic_int, but making use of the Deref traits

        // create a db int
        let mut db = DoubleBuffered::<i32>::default();

        *db = 3;
        // read buffer should not update until told to do so.
        assert!(*db == 0);
        db.update();
        assert!(*db == 3);

        // check the same thing again
        *db = 4;
        assert!(*db == 3);
        db.update();
        assert!(*db == 4);
    }
}

