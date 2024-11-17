use super::ViewId;

/// A response from a view
///
/// This contains the view id and anything returned from its [`ui.show()`](crate::view::Ui::show) closure.
///
/// This type implements both [`Deref`](std::ops::Deref) and
/// [`DerefMut`](std::ops::DerefMut) which allows you to access the inner data
/// via the dot operator
#[derive(Debug)]
pub struct Response<T = ()> {
    id: ViewId,
    inner: T,
}

/// When the response is a tuple, some convenience functions are provided
impl<L, R> Response<(L, R)> {
    /// 'Flatten' the response to the 'left' side
    pub fn flatten_left(self) -> Response<L> {
        self.map(|(l, _)| l)
    }
    /// 'Flatten' the response to the 'right' side
    pub fn flatten_right(self) -> Response<R> {
        self.map(|(_, r)| r)
    }
    /// Split the response into 2 separate responses
    ///
    /// This uses the same id for both parts
    pub fn split(self) -> (Response<L>, Response<R>) {
        let (left, right) = self.inner;
        (Response::new(self.id, left), Response::new(self.id, right))
    }
}

impl<T> Response<T> {
    pub(in crate::view) const fn new(id: ViewId, inner: T) -> Self {
        Self { id, inner }
    }

    /// Map the inner response data to some new data
    pub fn map<U>(self, map: impl FnOnce(T) -> U) -> Response<U> {
        Response {
            id: self.id,
            inner: map(self.inner),
        }
    }

    /// Get the id which produced this response
    pub const fn id(&self) -> ViewId {
        self.id
    }

    /// Consume the response, returning the inner data
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

impl<T> std::ops::DerefMut for Response<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
