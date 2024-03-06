

mod models;
mod database;
mod api;

#[macro_use] extern crate rocket;
use std::env;
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
use mongodb::{bson::{doc, Document,Bson}};

#[derive(FromForm)]
struct SearchTerm {
    query1: String,
    query2: String,
}
#[derive(FromForm)]
struct CorpusSearchTerm {
    term: String,
}

#[derive(FromForm)]
struct EegSearchParams {
    event1: String,
    event2: String,
    relation: String,
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
    // create eeg_allen_query api query uri with server ip and port
    let server_address = env::var("SERVER_ADDRESS");
    let server_port = env::var("SERVER_PORT");
    let query_uri = match (server_address, server_port) {
        (Ok(address), Ok(port)) => format!("http://{}:{}/eeg_allen_query?relation={}&query1={}&query2={}",address,port,relation,&search_term.query1,&search_term.query2),
        _ => String::from("Error getting server address and port"),
    };



    let query_response = match query_response {
        Ok(val) => val.0, // Convert Vec<String> to a single String
        Err(_) => doc!{"error":String::from("Error occurred")}, // Handle error
    };

    let duration = start.elapsed().as_secs_f64();
    let duration = format!("{:.3}", duration);
    let mut output = format!("You searched for: {} and {}\nResponse time: {}\n", search_term.query1, search_term.query2, duration);
    let latex = query_response.get_str("exp_latex").unwrap();
    output.push_str(&format!("Latex: {}\n", latex));
    let mongo_query = query_response.get_document("tel_cond").unwrap().to_string();
    output.push_str(&format!("Mongo query: {}\n", mongo_query));
    output.push_str(&format!("Results(up to 10):\n"));
    let mut pattern_n = 0;
    let mut subject_set = std::collections::HashSet::new();
    for _doc in query_response.get_array("results").unwrap().iter().map(|doc| doc.as_document().unwrap()) {
        pattern_n += 1;
        let _doc_reuslt = _doc.get_document("_id").unwrap();
        let subjectid = _doc_reuslt.get_str("subjectid").unwrap();
        subject_set.insert(subjectid);
        if pattern_n <= 10 {
            output.push_str(&format!("subject:{},event1:[{},{}],event2:[{},{}]\n", _doc_reuslt.get_str("subjectid").unwrap(),_doc_reuslt.get_datetime("min_e1").unwrap(),_doc_reuslt.get_datetime("max_e1").unwrap(),_doc_reuslt.get_datetime("min_e2").unwrap(),_doc_reuslt.get_datetime("max_e2").unwrap() ) );
        }
    }

    output.push_str(&format!("Number of subjects: {}\n", subject_set.len()));
    output.push_str(&format!("Number of patterns: {}\n", pattern_n));
    output.push_str(&format!("See full API response: {}\n", query_uri));
    return output;
}

#[get("/eeg_query_page")]
fn eeg_query_page() -> RawHtml<String> {
    RawHtml(r#"
        <!DOCTYPE html>
        <html>
        <body>
            <form action="/eeg_query_result" method="post">
                <select id="event1" name="event1">
                    <option value="53" selected>EEG Seizure</option>
                    <option value="79">Clinical Seizure</option>
                    <option value="497">Tonic Phase</option>
                    <option value="1122">Jittery Phase</option>
                    <option value="214">Clonic Phase</option>
                    <option value="941">EEG Suppression</option>
                    <option value="1279">Intermittent Slow</option>
                    <option value="1104">Continuous Slow</option>
                    <option value="652">Ictal</option>
                </select>
                <select id="relation" name="relation">
                    <option value="equal">Equal</option>
                    <option value="before" selected>Before</option>
                    <option value="meet">Meet</option>
                    <option value="contain">Contain</option>
                    <option value="start">Start With</option>
                    <option value="end">End With</option>
                    <option value="overlap">Overlap</option>
                </select>
                <select id="event2" name="event2">
                    <option value="53">EEG Seizure</option>
                    <option value="79">Clinical Seizure</option>
                    <option value="497">Tonic Phase</option>
                    <option value="1122">Jittery Phase</option>
                    <option value="214">Clonic Phase</option>
                    <option value="941" selected>EEG Suppression</option>
                    <option value="1279">Intermittent Slow</option>
                    <option value="1104">Continuous Slow</option>
                    <option value="652">Ictal</option>
                </select>
                <input type="submit" value="Search">
            </form>
        </body>
        </html>
    "#.to_string())
}


#[post("/eeg_query_result", data = "<eeg_search_params>")]
fn eeg_query_result(eegdb: &State<EegMongoRepo>,eeg_search_params: Form<EegSearchParams>) -> String {
    let start = Instant::now();
    let query_response = eeg_allen_query(eegdb,&eeg_search_params.relation,&eeg_search_params.event1,&eeg_search_params.event2);
    // create eeg_allen_query api query uri with server ip and port
    let server_address = env::var("SERVER_ADDRESS");
    let server_port = env::var("SERVER_PORT");
    let query_uri = match (server_address, server_port) {
        (Ok(address), Ok(port)) => format!("http://{}:{}/eeg_allen_query?relation={}&event_id_list1={}&event_id_list2={}",address,port,&eeg_search_params.relation,&eeg_search_params.event1,&eeg_search_params.event2),
        _ => String::from("Error getting server address and port"),
    };



    let query_response = match query_response {
        Ok(val) => val.0, // Convert Vec<String> to a single String
        Err(_) => doc!{"error":String::from("Error occurred")}, // Handle error
    };

    let duration = start.elapsed().as_secs_f64();
    let duration = format!("{:.3}", duration);
    let mut output = format!("You searched for: {} {} {}\nResponse time: {}\n", eeg_search_params.event1, eeg_search_params.relation, eeg_search_params.event2, duration);
    let latex = query_response.get_str("exp_latex").unwrap();
    output.push_str(&format!("Latex: {}\n", latex));
    let mongo_query = query_response.get_document("tel_cond").unwrap().to_string();
    output.push_str(&format!("Mongo query: {}\n", mongo_query));
    output.push_str(&format!("Results(up to 10):\n"));
    let mut pattern_n = 0;
    let mut subject_set = std::collections::HashSet::new();
    for _doc in query_response.get_array("results").unwrap().iter().map(|doc| doc.as_document().unwrap()) {
        pattern_n += 1;
        let _doc_reuslt = _doc.get_document("_id").unwrap();
        let subjectid = _doc_reuslt.get_str("subjectid").unwrap();
        subject_set.insert(subjectid);
        if pattern_n <= 10 {
            output.push_str(&format!("subject:{},event1:[{},{}],event2:[{},{}]\n", _doc_reuslt.get_str("subjectid").unwrap(),_doc_reuslt.get_datetime("min_e1").unwrap(),_doc_reuslt.get_datetime("max_e1").unwrap(),_doc_reuslt.get_datetime("min_e2").unwrap(),_doc_reuslt.get_datetime("max_e2").unwrap() ) );
        }
    }

    output.push_str(&format!("Number of subjects: {}\n", subject_set.len()));
    output.push_str(&format!("Number of patterns: {}\n", pattern_n));
    output.push_str(&format!("See full API response: {}\n", query_uri));
    return output;
}

#[launch]
fn rocket() -> _ {
    let db = MongoRepo::init();
    let eegdb: EegMongoRepo = EegMongoRepo::init();
    rocket::build()
        .manage(db)
        .manage(eegdb)
        .mount("/", routes![index, search, event_explore, event_search,  get_event, elii, rtq_telii, eeg_before_query_page, eeg_query_page, eeg_before_result, eeg_query_result, eeg_allen_query])

}
