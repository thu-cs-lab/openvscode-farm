use actix::fut::Ready;
use actix_web::{FromRequest, HttpMessage, ResponseError};
use failure::Fail;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LoginState {
    pub user_name: String,
}

#[derive(Debug, Fail)]
pub enum LoginError {
    #[fail(display = "Not logged-in")]
    NotLoggedIn,
}

impl ResponseError for LoginError {
    fn status_code(&self) -> actix_http::StatusCode {
        actix_http::StatusCode::FORBIDDEN
    }
}

impl FromRequest for LoginState {
    type Error = LoginError;
    type Future = Ready<Result<Self, LoginError>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_http::Payload) -> Self::Future {
        let ext = req.extensions().get::<Self>().cloned();
        futures::future::ready(ext.ok_or(LoginError::NotLoggedIn))
    }
}
