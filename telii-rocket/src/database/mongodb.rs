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
    pub temporal_relation_col: Collection<Document>,
    pub data_col: Collection<Document>,
}

impl MongoRepo {
    pub fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("MONGOURI") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error loading env variable"),
        };
        let client = Client::with_uri_str(uri).unwrap();
        let db = client.database("optum_covid19_elii_tree_20220120");
        // let event_col: Collection<Event> = db.collection("event_v3_g90s_1");
        let temporal_relation_col: Collection<Document> = db.collection("tree_v3_g90s_1");
        // let data_col: Collection<Document> = db.collection("tree_tii_v3_g90s_1");
        let event_col: Collection<Event> = db.collection("event_vv_gall_1");
        let data_col: Collection<Document> = db.collection("telii_v4_gall_1");
        MongoRepo { db,event_col,temporal_relation_col,data_col }
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