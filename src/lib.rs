use arc_swap::{ArcSwap, Guard};
use std::sync::Arc;
use std::ops::Deref;

pub enum AnyCow<'a, T>
where
    T: 'a + ToOwned,
{
    Borrowed(&'a T),
    Owned(T),
    Boxed(Box<T>),
    Shared(Arc<T>),
    Updatable(ArcSwap<T>),
}

impl<'a, T> AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T>,
{
    pub fn borrowed(value: &'a T) -> Self {
        AnyCow::Borrowed(value)
    }

    pub fn owned(value: T) -> Self {
        AnyCow::Owned(value)
    }

    pub fn boxed(value: Box<T>) -> Self {
        AnyCow::Boxed(value)
    }

    pub fn shared(value: Arc<T>) -> Self {
        AnyCow::Shared(value)
    }

    pub fn updatable(value: T) -> Self {
        AnyCow::Updatable(ArcSwap::from(Arc::new(value)))
    }

    pub fn is_borrowed(&self) -> bool {
        matches!(self, AnyCow::Borrowed(_))
    }

    pub fn is_owned(&self) -> bool {
        matches!(self, AnyCow::Owned(_))
    }

    pub fn is_boxed(&self) -> bool {
        matches!(self, AnyCow::Boxed(_))
    }

    pub fn to_mut(&mut self) -> &mut T {
        match self {
            AnyCow::Borrowed(value) => {
                *self = AnyCow::Owned(value.to_owned());
                match self {
                    AnyCow::Owned(value) => value,
                    _ => unreachable!(),
                }
            }
            AnyCow::Owned(value) => value,
            AnyCow::Shared(value) => {
                *self = AnyCow::Owned(value.as_ref().to_owned());
                match self {
                    AnyCow::Owned(value) => value,
                    _ => unreachable!(),
                }
            }
            AnyCow::Updatable(value) => {
                let owned = value.load().as_ref().to_owned();
                *self = AnyCow::Owned(owned);
                match self {
                    AnyCow::Owned(value) => value,
                    _ => unreachable!(),
                }
            }
            AnyCow::Boxed(value) => {
                *self = AnyCow::Owned((**value).to_owned());
                match self {
                    AnyCow::Owned(value) => value,
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn into_owned(self) -> T {
        match self {
            AnyCow::Borrowed(value) => value.to_owned(),
            AnyCow::Owned(value) => value,
            AnyCow::Shared(value) => Arc::try_unwrap(value).unwrap_or_else(|arc| arc.as_ref().to_owned()),
            AnyCow::Updatable(value) => value.load().as_ref().to_owned(),
            AnyCow::Boxed(value) => *value,
        }
    }

    pub fn borrow(&self) -> AnyCowRef<T> {
        match self {
            AnyCow::Borrowed(value) => AnyCowRef::Direct(value),
            AnyCow::Owned(value) => AnyCowRef::Direct(&*value),
            AnyCow::Shared(value) => AnyCowRef::Direct(&*value),
            AnyCow::Updatable(value) => AnyCowRef::Guarded(value.load()),
            AnyCow::Boxed(value) => AnyCowRef::Direct(&**value),
        }
    }

    pub fn try_replace(&self, new_val: T) -> Result<(), ()> {
        if let AnyCow::Updatable(a) = self {
            a.store(Arc::new(new_val));
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn to_arc(&self) -> Arc<T> {
        match self {
            AnyCow::Borrowed(value) => Arc::new((*value).to_owned()),
            AnyCow::Owned(value) => Arc::new(value.to_owned()),
            AnyCow::Shared(value) => value.clone(),
            AnyCow::Updatable(value) => value.load().to_owned(),
            AnyCow::Boxed(value) => Arc::new((**value).to_owned()),
        }
    }
}

impl<T> From<T> for AnyCow<'_, T>
where
    T: ToOwned,
{
    fn from(value: T) -> Self {
        AnyCow::Owned(value)
    }
}

impl<'a, T> From<&'a T> for AnyCow<'a, T>
where
    T: 'a + ToOwned,
{
    fn from(value: &'a T) -> Self {
        AnyCow::Borrowed(value)
    }
}

impl<T> From<Arc<T>> for AnyCow<'_, T>
where
    T: ToOwned,
{
    fn from(value: Arc<T>) -> Self {
        AnyCow::Shared(value)
    }
}

impl<T> From<Box<T>> for AnyCow<'_, T>
where
    T: ToOwned,
{
    fn from(value: Box<T>) -> Self {
        AnyCow::Boxed(value)
    }
}

pub enum AnyCowRef<'a, T>
where
    T: 'a + ToOwned,
{
    Direct(&'a T),
    Guarded(Guard<Arc<T>>),
}

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


impl<'a, T> Clone for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + Clone,
{
    fn clone(&self) -> Self {
        match self {
            AnyCow::Borrowed(value) => AnyCow::Borrowed(value),
            AnyCow::Owned(value) => AnyCow::Owned(value.clone()),
            AnyCow::Shared(value) => AnyCow::Shared(value.clone()),
            AnyCow::Updatable(value) => AnyCow::Shared(value.load().clone()),
            AnyCow::Boxed(value) => AnyCow::Owned((**value).clone()),
        }
    }
}

impl<'a, T> std::fmt::Debug for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyCow::Borrowed(value) => f.debug_tuple("Borrowed").field(value).finish(),
            AnyCow::Owned(value) => f.debug_tuple("Owned").field(value).finish(),
            AnyCow::Shared(value) => f.debug_tuple("Shared").field(value).finish(),
            AnyCow::Updatable(value) => f.debug_tuple("Updatable").field(&*value.load()).finish(),
            AnyCow::Boxed(value) => f.debug_tuple("Boxed").field(&**value).finish(),
        }
    }
}

impl<'a, T> PartialEq for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.borrow().deref() == other.borrow().deref()
    }
}

impl<'a, T> Eq for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + Eq,
{
}

impl<'a, T> std::hash::Hash for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.borrow().deref().hash(state)
    }
}

impl<'a, T> PartialOrd for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.borrow().deref().partial_cmp(other.borrow().deref())
    }
}

impl<'a, T> Ord for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.borrow().deref().cmp(other.borrow().deref())
    }
}

impl<'a, T> std::fmt::Display for AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T> + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.borrow().deref().fmt(f)
    }
}