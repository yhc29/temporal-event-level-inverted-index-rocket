use std::env;
extern crate dotenv;

use dotenv::dotenv;

use mongodb::{
    bson::{doc, extjson::de::Error, oid::ObjectId, Document},
    results::{InsertOneResult, UpdateResult, DeleteResult},
    sync::{Client, Collection, Database},
};
use crate::models::event::Event;

pub struct MongoRepo {
    db: Database,
    event_col: Collection<Event>,
    pub corpus_col: Collection<Document>,
    pub elii_col: Collection<Document>,
    pub telii_col: Collection<Document>,
    pub telii_common_col: Collection<Document>,
    pub timeline_col: Collection<Document>,
}

pub struct EegMongoRepo {
    db: Database,
    event_col: Collection<Event>,
    pub timeline_col: Collection<Document>,
}

impl MongoRepo {
    pub fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("MONGOURI") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error loading env variable"),
        };
        let client = Client::with_uri_str(uri).unwrap();
        let db = client.database("optum_covid19_telii_20220120");
        let event_col: Collection<Event> = db.collection("event_v4");
        let corpus_col: Collection<Document> = db.collection("term_corpus_v4");
        let elii_col: Collection<Document> = db.collection("elii_v4");
        let telii_col: Collection<Document> = db.collection("telii_v4_diag_gall_7");
        let telii_common_col: Collection<Document> = db.collection("telii_common_v4_diag_gall_7");
        let timeline_col: Collection<Document> = db.collection("pt_timeline_v4_diag_gall_7");
        MongoRepo { db,event_col,corpus_col,elii_col,telii_col,telii_common_col,timeline_col }
    }
    pub fn get_event(&self, id: &str) -> Result<Event, Error> {
        let id = id.parse::<i32>().unwrap();
        print!("id: {}", id);
        let filter = doc! {"id": id};
        let event_detail: Option<Event> = self
          .event_col
          .find_one(filter, None)
          .ok()
          .expect("Error getting event's detail");
        Ok(event_detail.unwrap())
    }

    pub fn search_icd10_diag_of_event_ids(&self, codes: &Vec<String>) -> Result<Vec<i32>, mongodb::error::Error> {
        let filter = doc! {"cov_diag.DIAGNOSIS_CD": {"$in": codes}, "cov_diag.DIAGNOSIS_STATUS": "Diagnosis of", "cov_diag.DIAGNOSIS_CD_TYPE": "ICD10"};
        let mut cursor = self
          .event_col
          .find(filter, None)
          .ok()
          .expect("Error getting event's detail");
        // get id list
        let mut results: Vec<i32> = Vec::new();
        while let Some(result) = cursor.next() {
            match result {
                Ok(document) => {
                    results.push(document.id);
                }
                Err(e) => {
                    println!("Error getting event's detail");
                    return Err(e.into());
                }
            }
        }

        Ok(results)
    }
    pub fn relative_temporal_query_telii<T>(&self, event_id_list1: &Vec<i32>, event_id_list2: &Vec<i32>) -> Result<Vec<String>, mongodb::error::Error> {
        let temporal_relation_col: Collection<T> = self.db.collection("tree_v3_g89__1");

        let mut results: Vec<String> = Vec::new();
        Ok(results)
    }

}

impl EegMongoRepo {
    pub fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("MONGOURI") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error loading env variable"),
        };
        let client = Client::with_uri_str(uri).unwrap();
        let db = client.database("eegdb_telii_amia2024");
        let event_col: Collection<Event> = db.collection("event_v4");
        let timeline_col: Collection<Document> = db.collection("pt_timeline_eeg_v4_7");
        EegMongoRepo { db,event_col,timeline_col }
    }
}

