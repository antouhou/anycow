use std::sync::Arc;
use arc_swap::ArcSwap;

pub enum AnyCow<'a, T>
where T: 'a + ToOwned
{
    Borrowed(&'a T),
    Boxed(Box<T>),
    Shared(Arc<T>),
    Updatable(ArcSwap<T>),
}

impl<'a, T> AnyCow<'a, T>
where T: 'a + ToOwned
{
    pub fn updatable(value: T) -> Self {
        AnyCow::Updatable(ArcSwap::from(Arc::new(value)))
    }

    pub fn into_owned(self) -> T {
        match self {
            AnyCow::Borrowed(value) => value.to_owned(),
            AnyCow::Boxed(value) => *value,
            AnyCow::Shared(value) => (*value).to_owned(),
            AnyCow::Updatable(value) => {
                value.load().as_ref().to_owned()
            }
        }
    }

    pub fn replace(&self, new_val: T) {
        if let AnyCow::Updatable(a) = self {
            a.store(Arc::new(new_val));
        }
    }

    pub fn borrow(&self) -> AnyBorrow<'a, T>; // like your StyleRef; Deref<Target=T>
    pub fn try_replace(&self, new_val: T) -> Result<(), NotUpdatable>;
    pub fn try_into_owned(self) -> Result<T, Self>;
    pub fn to_arc(&self) -> std::sync::Arc<T>;

    // If you want true CoW semantics:
    pub fn to_mut(&mut self) -> &mut T where T: Clone;
}

impl<T> From<T> for AnyCow<'_, T> {
    fn from(value: T) -> Self {
        AnyCow::Boxed(Box::new(value))
    }
}

impl<'a, T> From<&'a T> for AnyCow<'a, T> {
    fn from(value: &'a T) -> Self {
        AnyCow::Borrowed(value)
    }
}

impl<T> From<Arc<T>> for AnyCow<'_, T> {
    fn from(value: Arc<T>) -> Self {
        AnyCow::Shared(value)
    }
}
