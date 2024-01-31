

mod models;
mod database;
mod api;

#[macro_use] extern crate rocket;
use rocket::{get, http::Status, serde::json::Json, State};
use rocket::http::ContentType;
use rocket::response::Responder;
use rocket::Request;
use rocket::response::Response;
use std::io::Cursor;
use rocket::response::content::RawHtml;
use rocket::form::Form;
use rocket::local::blocking::Client;

use api::event_api::get_event;
use api::query_api::rtq_telii;
use database::mongodb::MongoRepo;

#[derive(FromForm)]
struct SearchTerm {
    query1: String,
    query2: String,
}

#[get("/")]
fn index() -> RawHtml<String> {
    RawHtml(r#"
        <!DOCTYPE html>
        <html>
        <body>
            <form action="/search" method="post">
                <input type="text" id="query1" name="query1">
                <input type="text" id="query2" name="query2">
                <input type="submit" value="Search">
            </form>
        </body>
        </html>
    "#.to_string())
}

#[post("/search", data = "<search_term>")]
fn search(db: &State<MongoRepo>,search_term: Form<SearchTerm>) -> String {
    let query_response = rtq_telii(db,&search_term.query1,&search_term.query2);
    let query_len = match &query_response {
        Ok(val) => val.0.len(), // Get the length of Vec<String>
        Err(_) => 0, // Handle error
    };
    let query_response = match query_response {
        Ok(val) => format!("{:?}", val.0), // Convert Vec<String> to a single String
        Err(_) => String::from("Error occurred"), // Handle error
    };

    format!("You searched for: {} and {}\nNumber of results: {}\nResponse: {}\n", query_response, search_term.query1, search_term.query2, query_len)
}


#[launch]
fn rocket() -> _ {
    let db = MongoRepo::init();
    rocket::build()
        .manage(db)
        .mount("/", routes![index, search, get_event, rtq_telii])

}
