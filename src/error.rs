use actix_web::{HttpResponse, ResponseError};
use failure::Fail;

#[derive(Debug, Fail)]
pub enum ServiceError {
  #[fail(display = "Bad Request")]
  BadRequest,

  #[fail(display = "Unauthorised")]
  Unauthorised,

  #[fail(display = "Payment Required")]
  PaymentRequired,

  #[fail(display = "Internal Server Error")]
  InternalServerError,
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            ServiceError::BadRequest => HttpResponse::BadRequest().json("Bad Request"),
            ServiceError::Unauthorised => HttpResponse::Unauthorized().json("Unauthorised"),
            ServiceError::InternalServerError => HttpResponse::InternalServerError().json("Internal Server Error"),
            ServiceError::PaymentRequired => HttpResponse::PaymentRequired().json("Payment Required"),
        }
    }
}
