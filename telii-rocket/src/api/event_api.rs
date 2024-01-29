use crate::{models::event::Event, database::mongodb::MongoRepo};
use mongodb::results::InsertOneResult;
use rocket::{http::Status, serde::json::Json, State};

#[get("/event/<path>")]
pub fn get_event(db: &State<MongoRepo>, path: &str) -> Result<Json<Event>, Status> {
    let id = path;
    if id.is_empty() {
        return Err(Status::BadRequest);
    };
    let event_detail = db.get_event(&id);
    match event_detail {
        Ok(event) => Ok(Json(event)),
        Err(_) => Err(Status::InternalServerError),
    }
}