use std::{result, vec};

use crate::{models::event::Event, database::mongodb::EegMongoRepo};
use mongodb::{bson::{doc, Document,Bson}, Collection, results::{self, InsertOneResult}};
use rocket::{http::Status, serde::json::{self, Json}, State};
use std::collections::HashSet;
use std::collections::HashMap;
use maplit::hashmap;

#[derive(Clone)]
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
	pub fn init(operator: &str, t: &str, event: &str, events: Option<Vec<&str>>, delta: Option<f32>, s: Option<&str>, e: Option<&str> ) -> Self {
		let events = events.unwrap_or(vec![]).iter().map(|&s| s.to_string()).collect();
		let delta = delta.unwrap_or(0.0);
		let s = s.unwrap_or("");
		let e = e.unwrap_or("");
		TelExp {
			operator: operator.to_string(),
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
	pub fn latex(&self) -> String {
		let exp = self.operator.split("_").collect::<Vec<&str>>();
		let mut op_str = "";
			if exp[0] == "box" {
				op_str = " \\Box ";
			} else if exp[0] == "diamond" {
				op_str = " \\Diamond ";
			}
			let mut t_str = self.t.clone();
			if self.delta != 0.0 {
				t_str = format!("{}+{}", self.t, self.delta);
			}

			let mut latex_str2 = String::new();
			if exp[1] == "t"{
				if exp.contains(&"neg"){
					latex_str2 = format!("{}_{{{}}} \\neg {}", op_str, t_str, self.event);
				}else{
					latex_str2 = format!("{}_{{{}}} {}", op_str, t_str, self.event);
				}

			}else if exp.last().map(|&s| s) == Some("t"){
				if exp.contains(&"neg"){
					latex_str2 = format!("({} \\neg {})_{{{}}}", op_str, self.event, t_str);
				}else{
					latex_str2 = format!("({} {})_{{{}}}", op_str, self.event, t_str);
				}
			}
			latex_str2
		}
}

#[get("/eeg_allen_query?<relation>&<event_id_list1>&<event_id_list2>")]
pub fn eeg_allen_query(db: &State<EegMongoRepo>, relation: &str, event_id_list1: &str, event_id_list2: &str) -> Result<Json<Document>, Status> {
	// valid operations: before, after, overlap, contain, start, end
	let relation = relation.to_lowercase();
	if vec!["before", "after", "overlap", "contain", "start", "end", "meet", "equal"].contains(&&*relation) == false{
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
	let mut ts = hashmap!{
		"t" => "e1",
	};
	let mut exps = Vec::new();
	if relation == "before"{
		exps = vec![
			TelExp::init("box_t_phi", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_t_neg_phi", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("diamond_neg_phi_t", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_neg_phi_t", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_phi_t", "t", "e2", Some(vec!["e1","e2"]), Some(60.0*1000.0), None, None)
		];
	}else if relation == "contain"{
		ts = hashmap!{
			"t" => "e2",
		};
		exps = vec![
			TelExp::init("box_t_phi", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("diamond_t_neg_phi", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_phi_t", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("diamond_neg_phi_t", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None)
		];
	}else if relation == "start"{
		ts = hashmap!{
			"t" => "e2",
		};
		exps = vec![
			TelExp::init("box_t_phi", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_t_phi", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_phi_t", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("diamond_neg_phi_t", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None)
		];
	}else if relation == "end"{
		ts = hashmap!{
			"t" => "e2",
		};
		exps = vec![
			TelExp::init("box_t_phi", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("diamond_t_neg_phi", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_phi_t", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_phi_t", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None)
		];
	}else if relation == "overlap"{
		exps = vec![
			TelExp::init("box_t_phi", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("diamond_t_phi", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("diamond_t_neg_phi", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_phi_t", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("diamond_neg_phi_t", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None)
		];
	}else if relation == "meet"{
		exps = vec![
			TelExp::init("box_t_phi", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_t_neg_phi", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_neg_phi_t", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_phi_t", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None)
		];
	}else if relation == "equal"{
		exps = vec![
			TelExp::init("box_t_phi", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_t_neg_phi", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_neg_phi_t", "t", "e1", Some(vec!["e1","e2"]), Some(0.0), None, None),
			TelExp::init("box_phi_t", "t", "e2", Some(vec!["e1","e2"]), Some(0.0), None, None)
		];
	}


	let pipeline = construct_query(events, ts.clone(), exps.clone());
	let mut tel_cond = doc!{};
	for _step in pipeline.clone(){
		if _step.contains_key("$addFields"){
			// println!("{:?}", _step);
			tel_cond = _step.get_document("$addFields").unwrap().clone();
			tel_cond = tel_cond.get_document("tel_cond").unwrap().clone();
			if let Some(Bson::Document(tel_cond_doc)) = tel_cond.get_array("$cond").unwrap().first() {
				tel_cond = tel_cond_doc.clone();
			}
		}
	}
	let mut results = Vec::new();

	let mut cursor = db.timeline_col.aggregate(pipeline, None).unwrap();
  while let Some(result) = cursor.next() {
    match result {
      Ok(document) => {
        // println!("document: {:?}", document);
        results.push(document);
      }
      Err(e) => {
        println!("Error getting result");
        return Err(Status::InternalServerError);
      }
    }
  }
	let api_result = doc!{"exp_latex": construct_exps_latex(exps.clone(),ts.clone()), "tel_cond": tel_cond,"results": results.clone()};
	match results.len() {
		_ => Ok(Json(api_result)),
		// 0 => Err(Status::NotFound),
	}
}

pub fn construct_exps_latex(exps:Vec<TelExp>,ts:HashMap<&str,&str>) -> String {
	let mut t_set = HashSet::new();
	for (_k,_v) in ts.iter() {
		t_set.insert(format!("{} \\in {}", _k, _v));
	}
	let mut op_set = HashSet::new();
	for exp in exps {
		let exp_latex = exp.latex();
		op_set.insert(exp_latex.clone());
	}
	format!("$\\exists {}, {}$", t_set.into_iter().collect::<Vec<String>>().join(", "), op_set.into_iter().collect::<Vec<String>>().join(" \\land "))
}
pub fn construct_query(events: HashMap<&str,Vec<i32>>,ts:HashMap<&str,&str>,exps:Vec<TelExp>) -> Vec<Document> {
	// get all event ids from values of events
	let mut event_ids: HashSet<i32> = HashSet::new();
	let mut group_stmt = doc!{"_id": "$subjectid"};
	let mut filter_none_time_stmt = doc!{ "_id":1};
	let mut filter = Vec::<Document>::new();

	for (_k,_v) in events.iter() {
		let event_name = _k.to_string(); // Clone the value of event_name

		event_ids.extend(_v);
		group_stmt.insert(*_k, doc!{ "$addToSet": { "$cond": [ { "$in": [ "$e", _v ] }, "$times", None::<i32> ] } });
		filter_none_time_stmt.insert(event_name.clone(), doc!{"$setDifference": [ format!("${}", event_name), [None::<i32>]]}); 
		filter.push(doc!{"$gt": [ {"$size": format!("${}", event_name)}, 0]});
	}
	let event_ids: Vec<i32> = event_ids.into_iter().collect();

	// get tel conditions
	let tel_cond_stmt = construct_tel_cond(exps);
	// print!("{:?}", tel_cond_stmt);

	let mut mongo_stmt = vec![
		doc!{"$match": {"e": {"$in": event_ids}}},
		doc!{"$project": {"_id": 0, "subjectid": 1, "e": 1, "times": 1}},
		doc!{"$unwind": "$times"},
		doc!{"$group": group_stmt},
		doc!{"$project": filter_none_time_stmt},
		doc!{"$match": { "$expr": { "$and": filter } }}
	];
	let mut project_stmt = doc!{"_id": 1};
	for (_k,_v) in ts.iter() {
		project_stmt.insert(_k.to_string(), format!("${}", _v));
	}
	for _k in events.keys() {
		mongo_stmt.push(doc!{"$unwind": format!("${}", _k)});
		project_stmt.insert(format!("min_{}", _k),doc!{"$arrayElemAt": [format!("${}", _k), 0]});
		project_stmt.insert(format!("max_{}", _k),doc!{"$arrayElemAt": [format!("${}", _k), -1]});
	}
	mongo_stmt.push(doc!{"$project": project_stmt});
	for _k in ts.keys() {
		mongo_stmt.push( doc!{"$unwind": format!("${}", _k)} );
	}
	mongo_stmt.push(doc!{"$addFields": {"tel_cond": tel_cond_stmt}});
	mongo_stmt.push(doc!{"$match": {"tel_cond": true}});
	let mut group_fields = doc!{"subjectid": "$_id"};
	for _k in events.keys() {
		group_fields.insert(format!("min_{}", _k), format!("$min_{}", _k));
		group_fields.insert(format!("max_{}", _k), format!("$max_{}", _k));
	}
	mongo_stmt.push(doc!{"$group": {"_id": group_fields}});


	return mongo_stmt;

}

pub fn construct_tel_cond(exps:Vec<TelExp>) -> Document {
	let mut and_stmt: Vec<Document> = Vec::new();
	for exp in exps {
		let mut mongo_exp = doc!{};
		if exp.operator == "box_t_phi" {
			mongo_exp = box_t_phi(exp);
		} else if exp.operator == "box_t_neg_phi" {
			mongo_exp = box_t_neg_phi(exp);
		} else if exp.operator == "box_phi_t" {
			mongo_exp = box_phi_t(exp);
		} else if exp.operator == "box_neg_phi_t" {
			mongo_exp = box_neg_phi_t(exp);
		} else if exp.operator == "diamond_t_phi" {
			mongo_exp = diamond_t_phi(exp);
		} else if exp.operator == "diamond_t_neg_phi" {
			mongo_exp = diamond_t_neg_phi(exp);
		} else if exp.operator == "diamond_phi_t" {
			mongo_exp = diamond_phi_t(exp);
		} else if exp.operator == "diamond_neg_phi_t" {
			mongo_exp = diamond_neg_phi_t(exp);
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
	let s = exp.s;

	let mut mongo_stmt = doc!{};
	if s != "" {
		mongo_stmt = doc!{ "$and": [ { "$gte": [ format!("${}", s), format!("$min_{}", event) ]}, { "$lte": [ { "$add": [ format!("${}", t), delta ] }, format!("$max_{}", event) ] } ] };
	} else {
		let mut s_vec = vec![];
		for x in exp.events {
			s_vec.push(format!("$min_{}", x));
		}
		mongo_stmt = doc!{ "$and": [ { "$gte": [ { "$min": s_vec }, format!("$min_{}", event) ]}, { "$lte": [ { "$add": [ format!("${}", t), delta ] }, format!("$max_{}", event) ] } ] };
	}
	return mongo_stmt;
}

pub fn box_t_neg_phi(exp: TelExp) -> Document {
	let t = exp.t;
	let event = exp.event;
	let delta = exp.delta;
	let s = exp.s;

	let mut mongo_stmt = doc!{};
	if s != "" {
		mongo_stmt = doc!{ "$or": [ { "$gte": [ format!("$min_{}", event), { "$add": [ format!("${}", t) , delta ] }] }, { "$lte": [ format!("$max_{}", event), format!("${}", s)] } ] };
	} else {
		let mut s_vec = vec![];
		for x in exp.events {
			s_vec.push(format!("$min_{}", x));
		}
		mongo_stmt = doc!{ "$or": [ { "$gte": [ format!("$min_{}", event), { "$add": [ format!("${}", t) , delta ] }] }, { "$lte": [ format!("$max_{}", event), { "$min": s_vec }] } ] };
	}
	return mongo_stmt;
}

pub fn box_phi_t(exp: TelExp) -> Document {
	let t = exp.t;
	let event = exp.event;
	let delta = exp.delta;
	let e = exp.e;

	let mut mongo_stmt = doc!{};
	if e != "" {
		mongo_stmt = doc!{ "$and": [ { "$gte": [ { "$add": [ format!("${}", t), delta ] }, format!("$min_{}", event) ] }, { "$gte": [ format!("$max_{}", event), format!("${}", e) ] } ] };
	} else {
		let mut e_vec = vec![];
		for x in exp.events {
			e_vec.push(format!("$max_{}", x));
		}
		mongo_stmt = doc!{ "$and": [ { "$gte": [ { "$add": [ format!("${}", t), delta ] }, format!("$min_{}", event) ] }, { "$gte": [ format!("$max_{}", event), { "$max": e_vec } ] } ] };
	}
	return mongo_stmt;
}

pub fn box_neg_phi_t(exp: TelExp) -> Document {
	let t = exp.t;
	let event = exp.event;
	let delta = exp.delta;
	let e = exp.e;

	let mut mongo_stmt = doc!{};
	if e != "" {
		mongo_stmt = doc!{ "$or": [ { "$gte": [ format!("$min_{}", event), format!("${}", e) ] }, { "$lte": [ format!("$max_{}", event), { "$add": [ format!("${}", t), delta ] }] } ] };
	} else {
		let mut e_vec = vec![];
		for x in exp.events {
			e_vec.push(format!("$max_{}", x));
		}
		mongo_stmt = doc!{ "$or": [ { "$gte": [ format!("$min_{}", event), { "$max": e_vec } ] }, { "$lte": [ format!("$max_{}", event), { "$add": [ format!("${}", t), delta ] }] } ] };
	}
	return mongo_stmt;
}

pub fn diamond_t_phi(exp: TelExp) -> Document {
	let t = exp.t;
	let event = exp.event;
	let delta = exp.delta;
	let s = exp.s;

	let mut mongo_stmt = doc!{};
	if s != "" {
		mongo_stmt = doc!{ "$and": [ { "$gt": [ { "$add": [ format!("${}", t), delta ] },  format!("$min_{}", event) ] }, { "$gt": [ format!("$max_{}", event), format!("${}", s)  ] } ] };
	} else {
		let mut s_vec = vec![];
		for x in exp.events {
			s_vec.push(format!("$min_{}", x));
		}
		mongo_stmt = doc!{ "$and": [ { "$gt": [ { "$add": [ format!("${}", t), delta ] },  format!("$min_{}", event) ] }, { "$gt": [ format!("$max_{}", event), { "$min": s_vec } ] } ] };
	}
	return mongo_stmt;
}

pub fn diamond_t_neg_phi(exp: TelExp) -> Document {
	let t = exp.t;
	let event = exp.event;
	let delta = exp.delta;
	let s = exp.s;

	let mut mongo_stmt = doc!{};
	if s != "" {
		mongo_stmt = doc!{ "$or": [ { "$gt": [ format!("$min_{}", event), format!("${}", s) ] }, { "$lt": [ format!("$max_{}", event), { "$add": [ format!("${}", t), delta ] } ] } ] };
	} else {
		let mut s_vec = vec![];
		for x in exp.events {
			s_vec.push(format!("$min_{}", x));
		}
		mongo_stmt = doc!{ "$or": [ { "$gt": [ format!("$min_{}", event), { "$min": s_vec } ] }, { "$lt": [ format!("$max_{}", event), { "$add": [ format!("${}", t), delta ] } ] } ] };
	}
	return mongo_stmt;
}

pub fn diamond_phi_t(exp: TelExp) -> Document {
	let t = exp.t;
	let event = exp.event;
	let delta = exp.delta;
	let e = exp.e;

	let mut mongo_stmt = doc!{};
	if e != "" {
		mongo_stmt = doc!{ "$and": [ { "$gt": [ format!("${}", e), format!("$min_{}", event)] }, { "$gt": [ format!("$max_{}", event),  { "$add": [ format!("${}", t), delta ] } ] } ] };
	} else {
		let mut e_vec = vec![];
		for x in exp.events {
			e_vec.push(format!("$max_{}", x));
		}
		mongo_stmt = doc!{ "$and": [ { "$gt": [ { "$max": e_vec }, format!("$min_{}", event)] }, { "$gt": [ format!("$max_{}", event),  { "$add": [ format!("${}", t), delta ] } ] } ] };
	}
	return mongo_stmt;
}

pub fn diamond_neg_phi_t(exp: TelExp) -> Document {
	let t = exp.t;
	let event = exp.event;
	let delta = exp.delta;
	let e = exp.e;

	let mut mongo_stmt = doc!{};
	if e != "" {
		mongo_stmt = doc!{ "$and": [{ "$gt": [ format!("${}", e), format!("${}", t)]}, { "$or": [ { "$gt": [ format!("$min_{}", event),  { "$add": [ format!("${}", t), delta ] } ] }, { "$lt": [ format!("$max_{}", event), format!("${}", e) ] }] }]};
	} else {
		let mut e_vec = vec![];
		for x in exp.events {
			e_vec.push(format!("$max_{}", x));
		}
		mongo_stmt = doc!{ "$and": [{ "$gt": [ { "$max": e_vec.clone() }, format!("${}", t)]}, { "$or": [ { "$gt": [ format!("$min_{}", event),  { "$add": [ format!("${}", t), delta ] } ] }, { "$lt": [ format!("$max_{}", event), { "$max": e_vec.clone() } ] }] }]};
	}
	return mongo_stmt;
}