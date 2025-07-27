use arc_swap::ArcSwap;
use std::sync::Arc;

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

    pub fn borrow(&self) -> &T {
        match self {
            AnyCow::Borrowed(value) => value,
            AnyCow::Boxed(value) => &*value,
            AnyCow::Shared(value) => &*value,
            AnyCow::Updatable(value) => {

            },
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
