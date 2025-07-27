use arc_swap::{ArcSwap, Guard};
use std::sync::Arc;
use std::ops::Deref;

pub enum AnyCow<'a, T>
where
    T: 'a + ToOwned,
{
    Borrowed(&'a T),
    Boxed(Box<T>),
    Shared(Arc<T>),
    Updatable(ArcSwap<T>),
}

impl<'a, T> AnyCow<'a, T>
where
    T: 'a + ToOwned<Owned = T>,
{
    pub fn updatable(value: T) -> Self {
        AnyCow::Updatable(ArcSwap::from(Arc::new(value)))
    }

    pub fn into_owned(self) -> T {
        match self {
            AnyCow::Borrowed(value) => value.to_owned(),
            AnyCow::Boxed(value) => *value,
            AnyCow::Shared(value) => (*value).to_owned(),
            AnyCow::Updatable(value) => value.load().as_ref().to_owned(),
        }
    }

    pub fn borrow(&self) -> AnyCowRef<T> {
        match self {
            AnyCow::Borrowed(value) => AnyCowRef::Direct(value),
            AnyCow::Boxed(value) => AnyCowRef::Direct(&*value),
            AnyCow::Shared(value) => AnyCowRef::Direct(&*value),
            AnyCow::Updatable(value) => AnyCowRef::Guarded(value.load()),
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
            AnyCow::Boxed(value) => Arc::new(value.as_ref().to_owned()),
            AnyCow::Shared(value) => value.clone(),
            AnyCow::Updatable(value) => value.load().to_owned(),
        }
    }
}

impl<T> From<T> for AnyCow<'_, T>
where
    T: ToOwned,
{
    fn from(value: T) -> Self {
        AnyCow::Boxed(Box::new(value))
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
