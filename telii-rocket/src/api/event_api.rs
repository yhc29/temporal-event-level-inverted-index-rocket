use crate::{models::event::Event, database::mongodb::MongoRepo};
use mongodb::results::InsertOneResult;
use rocket::{http::Status, serde::json::Json, State};
use mongodb::{options::ClientOptions, Client, bson::doc, options::FindOptions};
use mongodb::bson::Regex;

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

#[get("/corpus_search?<term>")]
pub fn corpus_search(db: &State<MongoRepo>, term: &str) -> Result<Json<Vec<String>>, Status> {
    let filter = doc![
        "value": {
            "$regex": Regex {
                pattern: String::from(term),
                options: String::from("i"),
            }
        }
    ];
    let find_options = FindOptions::builder().projection(doc! {"_id": 0}).build();

    let mut cursor = db.corpus_col.find(filter, find_options).unwrap();
    let mut results: Vec<String> = Vec::new();
    while let Some(result) = cursor.next() {
        match result {
            Ok(document) => {
                let document_string = document.to_string();
                // println!("{}", document_string);
                results.push(document_string);
            }
            Err(e) => return Err(Status::InternalServerError),
        }
    }

    Ok((Json(results)))
}