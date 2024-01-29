use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: i32,
    pub cov_diag: bson::Document,
    pub num_of_patients: i32,
}