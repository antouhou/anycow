//! # AnyCow - A Supercharged Container for Read-Heavy, Update-Light Data
//!
//! `AnyCow` is a versatile, high-performance container that extends the concept of `Cow` 
//! (Clone-on-Write) with multiple storage strategies optimized for different use cases. 
//! It's perfect for scenarios where you need to read values frequently but update them 
//! only occasionally.
//!
//! ## Features
//!
//! - **Multiple Storage Strategies**: Choose the right storage for your use case
//! - **Lock-Free Updates**: Atomic updates using `arc-swap` for the `Updatable` variant
//! - **Thread-Safe Options**: Share data safely across threads
//! - **Zero-Cost Abstractions**: Minimal overhead for common operations
//!
//! ## Storage Variants
//!
//! - [`AnyCow::Borrowed`] - Zero-cost references for temporary data
//! - [`AnyCow::Owned`] - Heap-allocated owned data via `Box<T>` 
//! - [`AnyCow::Shared`] - `Arc<T>` for shared immutable data across threads
//! - [`AnyCow::Updatable`] - Lock-free atomic updates using `arc-swap`
//!
//! ## Quick Example
//!
//! ```rust
//! use anycow::AnyCow;
//!
//! // Create from different sources
//! let borrowed = AnyCow::borrowed(&"hello");
//! let owned = AnyCow::owned(String::from("world"));
//! let updatable = AnyCow::updatable(vec![1, 2, 3]);
//!
//! // Read values efficiently
//! println!("{}", *borrowed.borrow()); // "hello"
//! println!("{}", *owned.borrow());    // "world"
//!
//! // Atomic updates (lock-free!)
//! updatable.try_replace(vec![4, 5, 6]).unwrap();
//! ```

use arc_swap::{ArcSwap, Guard};
use std::sync::Arc;
use std::ops::Deref;

/// A supercharged container that can hold data in multiple storage formats,
/// optimized for read-heavy, occasionally-updated scenarios.
///
/// `AnyCow` extends the concept of `Cow` (Clone-on-Write) by providing multiple
/// storage strategies, each optimized for different use cases:
///
/// - **Borrowed**: Zero-cost references to existing data
/// - **Owned**: Heap-allocated owned data via `Box<T>`
/// - **Shared**: Reference-counted sharing via `Arc<T>`
/// - **Updatable**: Atomic, lock-free updates via `arc-swap`
///
/// # Examples
///
/// ```rust
/// use anycow::AnyCow;
/// use std::sync::Arc;
///
/// // Different ways to create AnyCow
/// let borrowed = AnyCow::borrowed(&"hello");
/// let owned = AnyCow::owned(String::from("world"));
/// let shared = AnyCow::shared(Arc::new(42));
/// let updatable = AnyCow::updatable(vec![1, 2, 3]);
///
/// // All variants can be read the same way
/// assert_eq!(*borrowed.borrow(), "hello");
/// assert_eq!(*owned.borrow(), "world");
/// assert_eq!(*shared.borrow(), 42);
/// assert_eq!(*updatable.borrow(), vec![1, 2, 3]);
/// ```
pub enum AnyCow<'a, T>
where
    T: 'a + ToOwned,
{
    /// A borrowed reference to the data with zero allocation cost.
    /// 
    /// This variant is ideal for temporary references and hot code paths
    /// where you want to avoid any allocation overhead.
    Borrowed(&'a T),
    
    /// Heap-allocated owned data stored in a `Box<T>`.
    /// 
    /// This variant gives you ownership of the data stored on the heap
    /// and allows for direct mutation via [`to_mut()`](AnyCow::to_mut).
    /// Useful for data that needs to be owned and potentially large.
    Owned(Box<T>),
    
    /// Reference-counted shared data via `Arc<T>`.
    /// 
    /// Perfect for sharing immutable data across multiple threads
    /// or when you need multiple owners of the same data.
    Shared(Arc<T>),
    
    /// Atomically updatable data using lock-free operations.
    /// 
    /// This variant uses `arc-swap` to provide lock-free, atomic updates
    /// while allowing multiple concurrent readers. Ideal for configuration
    /// data, caches, or any shared state that needs occasional updates.
    Updatable(ArcSwap<T>),
}

impl<'a, T> AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T>,
{
    /// Creates a new `AnyCow` with a borrowed reference to the data.
    /// 
    /// This is the most efficient variant as it involves no allocation
    /// and provides zero-cost access to the underlying data.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use anycow::AnyCow;
    /// 
    /// let data = "hello world";
    /// let cow = AnyCow::borrowed(&data);
    /// assert!(cow.is_borrowed());
    /// ```
    pub fn borrowed(value: &'a T) -> Self {
        AnyCow::Borrowed(value)
    }

    /// Creates a new `AnyCow` with owned data stored in a `Box<T>`.
    /// 
    /// The data is moved into a heap-allocated box and can be mutated
    /// via [`to_mut()`](Self::to_mut).
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use anycow::AnyCow;
    /// 
    /// let cow = AnyCow::owned(String::from("hello"));
    /// assert!(cow.is_owned());
    /// ```
    pub fn owned(value: T) -> Self {
        AnyCow::Owned(Box::new(value))
    }

    /// Creates a new `AnyCow` with reference-counted shared data.
    /// 
    /// Perfect for sharing immutable data across multiple threads
    /// or when you need multiple owners.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use anycow::AnyCow;
    /// use std::sync::Arc;
    /// 
    /// let data = Arc::new(String::from("shared data"));
    /// let cow = AnyCow::shared(data);
    /// ```
    pub fn shared(value: Arc<T>) -> Self {
        AnyCow::Shared(value)
    }

    /// Creates a new `AnyCow` with atomically updatable data.
    /// 
    /// This variant uses `arc-swap` for lock-free, atomic updates
    /// while allowing concurrent reads. Perfect for configuration
    /// data, caches, or shared state with infrequent updates.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use anycow::AnyCow;
    /// 
    /// let cow = AnyCow::updatable(vec![1, 2, 3]);
    /// 
    /// // Read the current value
    /// assert_eq!(*cow.borrow(), vec![1, 2, 3]);
    /// 
    /// // Atomically update the value
    /// cow.try_replace(vec![4, 5, 6]).unwrap();
    /// assert_eq!(*cow.borrow(), vec![4, 5, 6]);
    /// ```
    pub fn updatable(value: T) -> Self {
        AnyCow::Updatable(ArcSwap::from(Arc::new(value)))
    }

    /// Returns `true` if this `AnyCow` contains a borrowed reference.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use anycow::AnyCow;
    /// 
    /// let data = "hello";
    /// let cow = AnyCow::borrowed(&data);
    /// assert!(cow.is_borrowed());
    /// 
    /// let cow = AnyCow::owned(String::from("hello"));
    /// assert!(!cow.is_borrowed());
    /// ```
    pub fn is_borrowed(&self) -> bool {
        matches!(self, AnyCow::Borrowed(_))
    }

    /// Returns `true` if this `AnyCow` contains owned data.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use anycow::AnyCow;
    /// 
    /// let cow = AnyCow::owned(String::from("hello"));
    /// assert!(cow.is_owned());
    /// 
    /// let data = "hello";
    /// let cow = AnyCow::borrowed(&data);
    /// assert!(!cow.is_owned());
    /// ```
    pub fn is_owned(&self) -> bool {
        matches!(self, AnyCow::Owned(_))
    }

    /// Returns a mutable reference to the owned data.
    /// 
    /// If the data is not already owned, this method will clone it
    /// (following Clone-on-Write semantics) and convert the container
    /// to the `Owned` variant.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use anycow::AnyCow;
    /// 
    /// let hello = String::from("hello");
    /// let mut cow = AnyCow::borrowed(&hello);
    /// assert!(cow.is_borrowed());
    /// 
    /// // This will clone the data and make it owned
    /// let mutable_ref = cow.to_mut();
    /// *mutable_ref = String::from("world");
    /// 
    /// assert!(cow.is_owned());
    /// assert_eq!(*cow.borrow(), "world");
    /// ```
    pub fn to_mut(&mut self) -> &mut T {
        match self {
            AnyCow::Borrowed(value) => {
                *self = AnyCow::Owned(Box::new(value.to_owned()));
                match self {
                    AnyCow::Owned(value) => value,
                    _ => unreachable!(),
                }
            }
            AnyCow::Owned(value) => value,
            AnyCow::Shared(value) => {
                *self = AnyCow::Owned(Box::new(value.as_ref().to_owned()));
                match self {
                    AnyCow::Owned(value) => value,
                    _ => unreachable!(),
                }
            }
            AnyCow::Updatable(value) => {
                let owned = value.load().as_ref().to_owned();
                *self = AnyCow::Owned(Box::new(owned));
                match self {
                    AnyCow::Owned(value) => value,
                    _ => unreachable!(),
                }
            }
        }
    }

    /// Converts this `AnyCow` into owned data.
    /// 
    /// This method consumes the container and returns the owned data,
    /// cloning if necessary. For `Arc` data, it will try to unwrap
    /// the `Arc` if there's only one reference, otherwise it will clone.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use anycow::AnyCow;
    /// use std::sync::Arc;
    /// 
    /// let hello = String::from("hello");
    /// let cow = AnyCow::borrowed(&hello);
    /// let owned: String = cow.into_owned();
    /// assert_eq!(owned, "hello");
    /// 
    /// let cow = AnyCow::shared(Arc::new(42));
    /// let owned: i32 = cow.into_owned();
    /// assert_eq!(owned, 42);
    /// ```
    pub fn into_owned(self) -> T {
        match self {
            AnyCow::Borrowed(value) => value.to_owned(),
            AnyCow::Owned(value) => *value,
            AnyCow::Shared(value) => Arc::try_unwrap(value).unwrap_or_else(|arc| arc.as_ref().to_owned()),
            AnyCow::Updatable(value) => value.load().as_ref().to_owned(),
        }
    }

    /// Returns a reference to the contained data.
    /// 
    /// This method provides unified access to the data regardless of
    /// the storage variant. For the `Updatable` variant, this returns
    /// a guard that ensures the data remains valid during access.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use anycow::AnyCow;
    /// use std::sync::Arc;
    /// 
    /// let borrowed = AnyCow::borrowed(&"hello");
    /// let owned = AnyCow::owned(String::from("world"));
    /// let shared = AnyCow::shared(Arc::new(42));
    /// 
    /// assert_eq!(*borrowed.borrow(), "hello");
    /// assert_eq!(*owned.borrow(), "world");
    /// assert_eq!(*shared.borrow(), 42);
    /// ```
    pub fn borrow(&self) -> AnyCowRef<T> {
        match self {
            AnyCow::Borrowed(value) => AnyCowRef::Direct(value),
            AnyCow::Owned(value) => AnyCowRef::Direct(&**value),
            AnyCow::Shared(value) => AnyCowRef::Direct(&*value),
            AnyCow::Updatable(value) => AnyCowRef::Guarded(value.load()),
        }
    }

    /// Attempts to atomically replace the value in an `Updatable` variant.
    /// 
    /// This method only succeeds if the container is of the `Updatable` variant.
    /// The replacement is atomic and lock-free, making it perfect for
    /// concurrent scenarios.
    /// 
    /// # Returns
    /// 
    /// - `Ok(())` if the replacement was successful
    /// - `Err(())` if this container is not the `Updatable` variant
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use anycow::AnyCow;
    /// 
    /// let updatable = AnyCow::updatable(vec![1, 2, 3]);
    /// assert_eq!(*updatable.borrow(), vec![1, 2, 3]);
    /// 
    /// // Atomic replacement
    /// assert!(updatable.try_replace(vec![4, 5, 6]).is_ok());
    /// assert_eq!(*updatable.borrow(), vec![4, 5, 6]);
    /// 
    /// // This will fail for non-updatable variants
    /// let owned = AnyCow::owned(vec![1, 2, 3]);
    /// assert!(owned.try_replace(vec![4, 5, 6]).is_err());
    /// ```
    pub fn try_replace(&self, new_val: T) -> Result<(), ()> {
        if let AnyCow::Updatable(a) = self {
            a.store(Arc::new(new_val));
            Ok(())
        } else {
            Err(())
        }
    }

    /// Converts this `AnyCow` to an `Arc<T>`.
    /// 
    /// This method will clone the data if necessary to create an `Arc`.
    /// If the container already holds an `Arc` (in the `Shared` or `Updatable` 
    /// variants), it may reuse the existing `Arc`.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use anycow::AnyCow;
    /// use std::sync::Arc;
    /// 
    /// let cow = AnyCow::owned(String::from("hello"));
    /// let arc: Arc<String> = cow.to_arc();
    /// assert_eq!(*arc, "hello");
    /// ```
    pub fn to_arc(&self) -> Arc<T> {
        match self {
            AnyCow::Borrowed(value) => Arc::new((*value).to_owned()),
            AnyCow::Owned(value) => Arc::new((**value).to_owned()),
            AnyCow::Shared(value) => value.clone(),
            AnyCow::Updatable(value) => value.load().to_owned(),
        }
    }
}

/// Automatic conversion from owned values.
/// 
/// This implementation allows any owned value to be automatically
/// converted into an `AnyCow::Owned` variant.
/// 
/// # Examples
/// 
/// ```rust
/// use anycow::AnyCow;
/// 
/// let cow: AnyCow<String> = String::from("hello").into();
/// assert!(cow.is_owned());
/// ```
impl<T> From<T> for AnyCow<'_, T>
where
    T: ToOwned,
{
    fn from(value: T) -> Self {
        AnyCow::Owned(Box::new(value))
    }
}

/// Automatic conversion from borrowed references.
/// 
/// This implementation allows borrowed references to be automatically
/// converted into an `AnyCow::Borrowed` variant.
/// 
/// # Examples
/// 
/// ```rust
/// use anycow::AnyCow;
/// 
/// let data = String::from("hello");
/// let cow: AnyCow<String> = (&data).into();
/// assert!(cow.is_borrowed());
/// ```
impl<'a, T> From<&'a T> for AnyCow<'a, T>
where
    T: 'a + ToOwned,
{
    fn from(value: &'a T) -> Self {
        AnyCow::Borrowed(value)
    }
}

/// Automatic conversion from `Arc<T>`.
/// 
/// This implementation allows `Arc<T>` values to be automatically
/// converted into an `AnyCow::Shared` variant.
/// 
/// # Examples
/// 
/// ```rust
/// use anycow::AnyCow;
/// use std::sync::Arc;
/// 
/// let arc = Arc::new(String::from("hello"));
/// let cow: AnyCow<String> = arc.into();
/// ```
impl<T> From<Arc<T>> for AnyCow<'_, T>
where
    T: ToOwned,
{
    fn from(value: Arc<T>) -> Self {
        AnyCow::Shared(value)
    }
}

/// A reference to data contained in an `AnyCow`.
/// 
/// This enum provides unified access to data regardless of how it's stored
/// in the `AnyCow`. The `Guarded` variant is used for the `Updatable` storage
/// to ensure the data remains valid during access through lock-free mechanisms.
/// 
/// # Examples
/// 
/// ```rust
/// use anycow::AnyCow;
/// 
/// let cow = AnyCow::owned(String::from("hello"));
/// let cow_ref = cow.borrow();
/// assert_eq!(&*cow_ref, "hello");
/// ```
pub enum AnyCowRef<'a, T>
where
    T: 'a + ToOwned,
{
    /// A direct reference to the data.
    /// 
    /// Used for `Borrowed`, `Owned`, `Shared`, and `Boxed` variants
    /// where we can provide a direct reference to the data.
    Direct(&'a T),
    
    /// A guarded reference to atomically-managed data.
    /// 
    /// Used for the `Updatable` variant to ensure the data remains
    /// valid during access through the `arc-swap` guard mechanism.
    Guarded(Guard<Arc<T>>),
}

/// Provides transparent access to the contained data.
/// 
/// This implementation allows `AnyCowRef` to be used transparently
/// as if it were a direct reference to the contained data.
impl<'a, T> Deref for AnyCowRef<'a, T>
where
    T: 'a + ToOwned,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            AnyCowRef::Direct(value) => value,
            AnyCowRef::Guarded(guard) => guard.as_ref(),
        }
    }
}

/// Cloning support for `AnyCow`.
/// 
/// Cloning behavior varies by variant:
/// - `Borrowed`: Copies the reference (cheap)
/// - `Owned`: Clones the owned data in the box
/// - `Shared`: Clones the `Arc` (cheap reference counting)
/// - `Updatable`: Converts to `Shared` with current data snapshot
impl<'a, T> Clone for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + Clone,
{
    fn clone(&self) -> Self {
        match self {
            AnyCow::Borrowed(value) => AnyCow::Borrowed(value),
            AnyCow::Owned(value) => AnyCow::Owned(Box::new((**value).clone())),
            AnyCow::Shared(value) => AnyCow::Shared(value.clone()),
            AnyCow::Updatable(value) => AnyCow::Shared(value.load().clone()),
        }
    }
}

/// Debug formatting for `AnyCow`.
/// 
/// Shows both the variant type and the contained data for easy debugging.
impl<'a, T> std::fmt::Debug for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyCow::Borrowed(value) => f.debug_tuple("Borrowed").field(value).finish(),
            AnyCow::Owned(value) => f.debug_tuple("Owned").field(&**value).finish(),
            AnyCow::Shared(value) => f.debug_tuple("Shared").field(value).finish(),
            AnyCow::Updatable(value) => f.debug_tuple("Updatable").field(&*value.load()).finish(),
        }
    }
}

/// Equality comparison for `AnyCow`.
/// 
/// Compares the contained data regardless of storage variant.
/// Two `AnyCow` instances are equal if their contained data is equal.
impl<'a, T> PartialEq for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.borrow().deref() == other.borrow().deref()
    }
}

/// Full equality for `AnyCow`.
impl<'a, T> Eq for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + Eq,
{
}

/// Hash implementation for `AnyCow`.
/// 
/// Hashes the contained data regardless of storage variant.
impl<'a, T> std::hash::Hash for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.borrow().deref().hash(state)
    }
}

/// Partial ordering for `AnyCow`.
/// 
/// Compares the contained data regardless of storage variant.
impl<'a, T> PartialOrd for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.borrow().deref().partial_cmp(other.borrow().deref())
    }
}

/// Total ordering for `AnyCow`.
/// 
/// Orders based on the contained data regardless of storage variant.
impl<'a, T> Ord for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.borrow().deref().cmp(other.borrow().deref())
    }
}

/// Display formatting for `AnyCow`.
/// 
/// Displays the contained data regardless of storage variant.
impl<'a, T> std::fmt::Display for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.borrow().deref().fmt(f)
    }
}