use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response, Result};
use rocket::serde::json::{Value, json};
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
pub struct ApiResponse {
    pub json: Value,
    pub status: Status,
}

impl<'r> Responder<'r, 'r> for ApiResponse {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'static> {
        Response::build_from(self.json.respond_to(req)?).status(self.status).header(ContentType::JSON).ok()
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ApiError {
    pub message: String,
    pub code: u16,
}

impl<'r> Responder<'r, 'r> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'static> {
        let status = Status { code: self.code };
        let body = json!({ "message": self.message, "code": self.code });
        Response::build_from(body.respond_to(req)?).status(status).header(ContentType::JSON).ok()
    }
}
