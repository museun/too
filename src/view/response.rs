use super::ViewId;

#[derive(Debug)]
pub struct Response<T = ()> {
    id: ViewId,
    inner: T,
}

impl<L, R> Response<(L, R)> {
    pub fn flatten_left(self) -> Response<L> {
        self.map(|(l, _)| l)
    }

    pub fn flatten_right(self) -> Response<R> {
        self.map(|(_, r)| r)
    }
}

impl<T> Response<T> {
    pub(in crate::view) const fn new(id: ViewId, inner: T) -> Self {
        Self { id, inner }
    }

    pub fn map<U>(self, map: impl FnOnce(T) -> U) -> Response<U> {
        Response {
            id: self.id,
            inner: map(self.inner),
        }
    }

    pub const fn id(&self) -> ViewId {
        self.id
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> std::ops::Deref for Response<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
