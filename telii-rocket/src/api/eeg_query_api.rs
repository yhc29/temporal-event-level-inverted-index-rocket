use std::result;

use crate::{models::event::Event, database::mongodb::EegMongoRepo};
use mongodb::{bson::{doc, Document,Bson}, Collection, results::{self, InsertOneResult}};
use rocket::{http::Status, serde::json::Json, State};
use std::collections::HashSet;
use std::collections::HashMap;
use maplit::hashmap;

pub struct TelExp {
	pub operator: String,
	pub t: String,
	pub event: String,
	pub events: Vec<String>,
	pub delta: f32,
	pub s: String,
	pub e: String,
}

impl TelExp {
	// Constructor
	pub fn init(operation: &str, t: &str, event: &str, events: Option<Vec<&str>>, delta: Option<f32>, s: Option<&str>, e: Option<&str> ) -> Self {
		let events = events.unwrap_or(vec![]).iter().map(|&s| s.to_string()).collect();
		let delta = delta.unwrap_or(0.0);
		let s = s.unwrap_or("");
		let e = e.unwrap_or("");
		TelExp {
			operator: operation.to_string(),
			t: t.to_string(),
			event: event.to_string(),
			events: events,
			delta: delta,
			s: s.to_string(),
			e: e.to_string(),
		}
	}

	// Method
	pub fn print(&self) {
			println!("operator: {}, t: {}, event: {}, events: {:?}, delta: {}, s: {}, e: {}", self.operator, self.t, self.event, self.events, self.delta, self.s, self.e);
	}
}

#[get("/eeg_allen_query?<relation>&<event_id_list1>&<event_id_list2>")]
pub fn eeg_allen_query(db: &State<EegMongoRepo>, relation: &str, event_id_list1: &str, event_id_list2: &str) -> Result<Json<Vec<Document>>, Status> {
	// valid operations: before, after, overlap, contain, start, end
	let relation = relation.to_lowercase();
	if relation != "before" && relation != "after" && relation != "overlap" && relation != "contain" && relation != "start" && relation != "end" {
		return Err(Status::NotFound);
	}
	let event_id_list1: Vec<i32> = event_id_list1.split(',')
			.filter_map(|s| s.parse().ok())
			.collect();
	let event_id_list2: Vec<i32> = event_id_list2.split(',')
			.filter_map(|s| s.parse().ok())
			.collect();
	

	// let mut events = HashMap::new();
	// events.insert("e1", event_id_list1);
	// events.insert("e2", event_id_list2);
	let events = hashmap!{
    "e1" => event_id_list1,
    "e2" => event_id_list2,
	};
	let ts = hashmap!{
		"t" => "e1",
	};
	let exps = vec![
		TelExp::init("box_t_phi", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None)
	];

	let pipeline = construct_query(events, ts, exps);

	let results = Vec::new();
	match results.len() {
	_ => Ok(Json(results)),
	// 0 => Err(Status::NotFound),
	}
}

pub fn construct_query(events: HashMap<&str,Vec<i32>>,ts:HashMap<&str,&str>,exps:Vec<TelExp>) -> Vec<Document> {
	let mongo_stmt = Vec::new();

	// get all event ids from values of events
	let mut event_ids: HashSet<i32>= HashSet::new();
	for (_k,v) in events.iter() {
		event_ids.extend(v);
	}
	// get a set of unique event ids
	


	let tel_cond_stmt = construct_tel_cond(exps);
	print!("{:?}", tel_cond_stmt);

	return mongo_stmt;

}

pub fn construct_tel_cond(exps:Vec<TelExp>) -> Document {
	let mut and_stmt: Vec<Document> = Vec::new();
	for exp in exps {
		let mut mongo_exp = doc!{};
		if exp.operator == "box_t_phi" {
			mongo_exp = box_t_phi(exp);
		} else {
			mongo_exp = doc! {};
		}
		
		and_stmt.push(mongo_exp);
	}
	let mongo_stmt = doc!{ "$cond": [{"$and": and_stmt}, true, false] };

	return mongo_stmt;

}

pub fn box_t_phi(exp: TelExp) -> Document {
	let t = exp.t;
	let event = exp.event;
	let delta = exp.delta;

	let mut mongo_stmt = doc!{};
	if exp.s != "" {
		mongo_stmt = doc!{ "$and": [ { "$gte": [ exp.s, format!("$min_{}", event) ]}, { "$lte": [ { "$add": [ format!("${}", t), delta ] }, format!("$max_{}", event) ] } ] };
	} else {
		let mut min_vec = vec![];
		for e in exp.events {
			min_vec.push(format!("$min_{}", e));
		}
		mongo_stmt = doc!{ "$and": [ { "$gte": [ { "$min": min_vec }, format!("$min_{}", event) ]}, { "$lte": [ { "$add": [ format!("${}", t), delta ] }, format!("$max_{}", event) ] } ] };
	}


	return mongo_stmt;
}