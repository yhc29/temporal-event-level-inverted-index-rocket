

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
use std::time::Instant;
use rocket::response::content::RawHtml;
use rocket::form::Form;
use rocket::local::blocking::Client;

use api::event_api::{get_event, corpus_search};
use api::query_api::{elii, rtq_telii};
use api::eeg_query_api::{eeg_allen_query};
use database::mongodb::{MongoRepo, EegMongoRepo};

#[derive(FromForm)]
struct SearchTerm {
    query1: String,
    query2: String,
}
#[derive(FromForm)]
struct CorpusSearchTerm {
    term: String,
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
    let start = Instant::now();
    let query_response = rtq_telii(db,&search_term.query1,&search_term.query2,None);
    let query_len = match &query_response {
        Ok(val) => val.0.len(), // Get the length of Vec<String>
        Err(_) => 0, // Handle error
    };
    let query_response = match query_response {
        Ok(val) => format!("{:?}", val.0), // Convert Vec<String> to a single String
        Err(_) => String::from("Error occurred"), // Handle error
    };
    // get the elapsed time in seconds
    let duration = start.elapsed().as_secs_f64();
    // // format duration to seconds with 3 digits after the decimal point
    let duration = format!("{:.3}", duration);
    format!("You searched for: {} and {}\nNumber of patients: {}. Response time: {}\nResponse: {}\n", search_term.query1, search_term.query2, query_len, duration, query_response,)
}

#[get("/event_explore")]
fn event_explore() -> RawHtml<String> {
    RawHtml(r#"
        <!DOCTYPE html>
        <html>
        <body>
            <form action="/event_search" method="post">
                <input type="text" id="term" name="term">
                <input type="submit" value="Search">
            </form>
        </body>
        </html>
    "#.to_string())
}

#[post("/event_search", data = "<search_term>")]
fn event_search(db: &State<MongoRepo>,search_term: Form<CorpusSearchTerm>) -> String {
    let start = Instant::now();
    let query_response = corpus_search(db,&search_term.term);
    let query_len = match &query_response {
        Ok(val) => val.0.len(), // Get the length of Vec<String>
        Err(_) => 0, // Handle error
    };
    let query_response = match query_response {
        Ok(val) => val.0, // Convert Vec<String> to a single String
        Err(_) => [String::from("Error occurred")].to_vec(), // Handle error
    };
    // get the elapsed time in seconds
    let duration = start.elapsed().as_secs_f64();
    // format duration to seconds with 3 digits after the decimal point
    let duration = format!("{:.3}", duration);
    let mut result = format!("You searched for: {}\nNumber of terms: {}. Response time: {}\n", search_term.term, query_len, duration);
    // iterate query_response
    for term in &query_response {
        result.push_str(&format!("{}\n", term));
    }
    result
}

#[get("/eeg_before_query_page")]
fn eeg_before_query_page() -> RawHtml<String> {
    RawHtml(r#"
        <!DOCTYPE html>
        <html>
        <body>
            <form action="/eeg_before_result" method="post">
                <input type="text" id="query1" name="query1">
                <input type="text" id="query2" name="query2">
                <input type="submit" value="Search">
            </form>
        </body>
        </html>
    "#.to_string())
}

#[post("/eeg_before_result", data = "<search_term>")]
fn eeg_before_result(eegdb: &State<EegMongoRepo>,search_term: Form<SearchTerm>) -> String {
    let start = Instant::now();
    let relation = "before";
    let query_response = eeg_allen_query(eegdb,relation,&search_term.query1,&search_term.query2);

    let query_response = match query_response {
        Ok(val) => format!("{:?}", val.0), // Convert Vec<String> to a single String
        Err(_) => String::from("Error occurred"), // Handle error
    };

    let duration = start.elapsed().as_secs_f64();
    let duration = format!("{:.3}", duration);
    format!("You searched for: {} and {}\nResponse time: {}\nResponse: {}\n", search_term.query1, search_term.query2, duration, query_response)
}


#[launch]
fn rocket() -> _ {
    let db = MongoRepo::init();
    let eegdb: EegMongoRepo = EegMongoRepo::init();
    rocket::build()
        .manage(db)
        .manage(eegdb)
        .mount("/", routes![index, search, event_explore, event_search,  get_event, elii, rtq_telii, eeg_before_query_page, eeg_before_result,eeg_allen_query])

}
