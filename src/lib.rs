use actix_http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use log::error;
use std::{backtrace::Backtrace, fmt::Display};

pub mod env;
pub mod login;
pub mod routes;

#[derive(Debug, Clone)]
pub struct Error {
    status_code: StatusCode,
    message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error ({}, {})", self.status_code, self.message)
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_http::StatusCode {
        self.status_code
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).body(self.message.as_bytes().to_owned())
    }
}

#[track_caller]
pub fn err<T: Display>(err: T) -> Error {
    let location = std::panic::Location::caller();
    let backtrace = Backtrace::force_capture();
    error!(
        "Error with err: {}, location: {}, backtrace: {}",
        err, location, backtrace
    );
    Error {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("Please contact admin"),
    }
}

pub const LOGIN_SESSION_KEY: &str = "vscode-session";
