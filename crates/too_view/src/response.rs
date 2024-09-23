use crate::{NoResponse, ViewId};

pub type UserResponse<R> = Response<NoResponse, R>;

#[derive(Debug)]
pub struct Response<T = (), R = ()> {
    resp: T,
    view_id: ViewId,
    inner: R,
}

impl<R> Response<R, ()> {
    // TODO this is ugly
    pub(crate) fn map_output<T>(self, output: T) -> Response<R, T> {
        Response {
            resp: self.resp,
            view_id: self.view_id,
            inner: output,
        }
    }
}

impl<T, R> Response<T, R> {
    pub(crate) fn new(view_id: ViewId, resp: T, inner: R) -> Self {
        Self {
            resp,
            view_id,
            inner,
        }
    }

    pub fn resp(&self) -> &T {
        &self.resp
    }

    pub fn inner(&self) -> &R {
        &self.inner
    }

    pub fn into_resp(self) -> T {
        self.resp
    }

    pub fn into_inner(self) -> R {
        self.inner
    }

    pub fn view_id(&self) -> ViewId {
        self.view_id
    }
}

impl<T, R> std::ops::Deref for Response<T, R> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.resp
    }
}
