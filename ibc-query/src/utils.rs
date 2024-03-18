use tonic::{Request, Response, Status};

use crate::error::QueryError;

pub trait TryIntoDomain<T> {
    fn try_into_domain(self) -> Result<T, Status>;
}

pub trait IntoDomain<T> {
    fn into_domain(self) -> T;
}

impl<T, Raw> TryIntoDomain<T> for Request<Raw>
where
    T: TryFrom<Raw, Error = QueryError>,
{
    fn try_into_domain(self) -> Result<T, Status> {
        Ok(self.into_inner().try_into()?)
    }
}

impl<T, Raw> IntoDomain<T> for Request<Raw>
where
    T: From<Raw>,
{
    fn into_domain(self) -> T {
        self.into_inner().into()
    }
}

pub trait IntoResponse<Raw>: Sized
where
    Self: Into<Raw>,
{
    fn into_response(self) -> Result<Response<Raw>, Status> {
        Ok(Response::new(self.into()))
    }
}

impl<T, Raw> IntoResponse<Raw> for T where T: Into<Raw> {}
