mod models;
mod database;
mod api;

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use rocket::response::content::Html;

use api::event_api::{get_event}; //import the handler here
use database::mongodb::MongoRepo;

#[get("/")]
fn index() -> Html<String> {
    Html(r#"
        <!DOCTYPE html>
        <html>
        <body>
            <form action="/search" method="post">
                <input type="text" id="query" name="query">
                <input type="submit" value="Search">
            </form>
        </body>
        </html>
    "#.to_string())
}

#[post("/search", data = "<search_term>")]
fn search(search_term: String) -> String {
    format!("You searched for: {}", search_term)
}

#[get("/")]
fn hello() -> Result<Json<String>, Status> {
    Ok(Json(String::from("Hello from rust and mongoDB")))
}

#[launch]
fn rocket() -> _ {
    let db = MongoRepo::init();
    rocket::build()
        .manage(db)
        .mount("/", routes![get_event])
}

fn main() {
    rocket::ignite().mount("/", routes![index, search, hello]).launch();
}
