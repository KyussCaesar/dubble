//! A generic double-buffer.
//!
//! # Description
//!
//! A generic double-buffer for anything that implements `Clone`.
//!
//! # Usage
//!
//! `DoubleBuffered` implements `Default` so long as the type being double-buffered
//! does.
//!
//! ```rust
//! use dubble::DoubleBuffered;
//! let mut my_buf = DoubleBuffered::<i32>::default();
//! ```
//!
//! Otherwise, you can use a `Fn() -> T` to construct the value in each buffer.
//! ```rust
//! # use dubble::DoubleBuffered;
//! let mut my_buf = DoubleBuffered::construct_with(Vec::<i32>::new);
//! ```
//! 
//! A longer example usage is shown below.
//!
//! ```rust
//! # use dubble::DoubleBuffered;
//! // creating a buffer using a closure to initialise the buffers
//! let mut my_buf = DoubleBuffered::construct_with(||
//! {
//!     let mut s = String::new();
//!     s.push_str("hello,");
//!     s
//! });
//!
//! // writing to the buffer
//! {
//!     // creates a mutable reference to the write buffer
//!     let mut wb = my_buf.write();
//!     wb.push_str(" world!");
//! }
//!
//! // NB: DoubleBuffer implements DerefMut, so we could also use the
//! // `push_str` method of `String` directly, like so.
//! my_buf.push_str(" Hello again!");
//!
//! // reading from the buffer
//! // note: the read half of the buffer should not have updated yet
//! assert!(my_buf.read() == "hello,");
//!
//! // updating the buffer
//! // other half of the buffer has been updated
//! // NB: DoubleBuffer implements Deref, so we could use the dereference operator
//! // here as well
//! my_buf.update();
//! assert!(*my_buf == "hello, world! Hello again!");
//! ```
//!
//! # Notes
//!
//! ## `Default`
//!
//! `DoubleBuffered` implements `Default` so long as the type being buffered 
//! implements it. For example, if you needed a double-buffered `i32`:
//!
//! ```
//! use dubble::DoubleBuffered;
//! let my_buf: DoubleBuffered<i32> = DoubleBuffered::default();
//! ```
//!
//! ## `Deref` and `DerefMut`
//!
//! An important note on the `impl`s for each of these is that `Deref` will 
//! dereference to the *read* buffer while `DerefMut` will dereference to the
//! *write* buffer. So, if you did `*my_buf = 3` followed by `assert(*my_buf == 3)`,
//! you'd find that the assertion would fail.
//!
//! ```rust,should_panic
//! # use dubble::DoubleBuffered;
//! # let mut my_buf: DoubleBuffered<i32> = DoubleBuffered::default();
//! *my_buf = 3;
//! assert!(*my_buf == 3);
//! ```
//!
//! In other words, `Deref` behaves as if you had called `my_buf.read()`, and
//! `DerefMut` behaves as if you had called `my_buf.write()`.
//!
#![no_std]

use core::ops::
{
    Deref,
    DerefMut,
    Index,
    IndexMut
};

/// Represents something that is double-buffered. The type being buffered must
/// be `Clone`, so that the read buffer can be updated with the contents of the
/// write buffer during the update.
///
/// See the module-level documentation for more information.
pub struct DoubleBuffered<T: Clone>
{
    rbuf: T,
    wbuf: T,
}

impl<T: Clone> DoubleBuffered<T>
{
    /// Initialises the double-buffer with the value. Both buffers are initialised
    /// with the same value.
    pub fn new(value: T) -> Self
    {
        Self
        {
            rbuf: value.clone(),
            wbuf: value.clone(),
        }
    }

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

    /// Returns an immutable reference to the read buffer.
    pub fn read(&self) -> &T
    {
        &self.rbuf
    }

    /// Returns a mutable reference to the write buffer.
    /// Note that changes made through this reference will not be reflected
    /// until after `update` is called.
    ///
    /// This might seem a little weird; "why not just go `my_buf.write(stuff)`"?.
    /// The reason is so that you can update the elements of a collection without
    /// having to build a clone of the collection. For example:
    ///
    /// ```rust
    /// # use dubble::DoubleBuffered;
    /// let mut my_buf: DoubleBuffered<Vec<i32>> = DoubleBuffered::default();
    /// let mut wb = my_buf.write();
    /// wb.push(4);
    /// ```
    ///
    /// Compared to the other potential form:
    ///
    /// ```rust,not_run
    /// # use dubble::DoubleBuffered;
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
    /// contents of the write buffer beforehand. You could think of this like
    /// "quit without saving" in a word processor.
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

impl<T: Default + Clone> Default for DoubleBuffered<T>
{
    /// Use the default constructor for the type.
    fn default() -> Self
    {
        Self::construct_with(T::default)
    }
}

impl<I, T: Index<I> + Clone> Index<I> for DoubleBuffered<T>
{
    type Output = <T as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output
    {
        &self.rbuf[index]
    }
}

impl<I, T: IndexMut<I> + Clone> IndexMut<I> for DoubleBuffered<T>
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output
    {
        &mut self.wbuf[index]
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

    #[test]
    fn vec_i32()
    {
        let mut db = DoubleBuffered::<Vec<i32>>::default();

        // using deref and index
        db.push(0);
        db.update();
        assert!(db[0] == 0);

        // read view should not change
        db[0] = 1;
        assert!(db[0] == 0);

        // should now be updated
        db.update();
        assert!(db[0] == 1);
    }
}

