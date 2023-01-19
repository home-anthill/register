use super::rocket;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::{Client, LocalRequest, LocalResponse};

use serde_json::{Value};

#[rocket::async_test]
async fn error_catcher_not_found() {
    let client: Client = Client::tracked(rocket()).await.unwrap();

    let req: LocalRequest = client.get("/unknownpath");
    let res: LocalResponse = req.dispatch().await;
    assert_eq!(res.status(), Status::NotFound);
    assert_eq!(
        res.into_string().await.unwrap(),
        String::from("Not found")
    );
}

#[rocket::async_test]
async fn error_catcher_bad_request() {
    let client: Client = Client::tracked(rocket()).await.unwrap();

    let req: LocalRequest = client
        .post("/sensors/register/temperature")
        .header(ContentType::JSON)
        .body("bad-input");
    let res: LocalResponse = req.dispatch().await;
    assert_eq!(res.status(), Status::BadRequest);
    assert_eq!(
        res.into_string().await.unwrap(),
        String::from("Bad request")
    );
}
