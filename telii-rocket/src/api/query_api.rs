use crate::{models::event::Event, database::mongodb::MongoRepo};
use mongodb::{bson::{doc, Document}, Collection, results::InsertOneResult};
use rocket::{http::Status, serde::json::Json, State};

// relative temporal query: event list1 before event list2
// input: event list1: vec of event ids, event list2: vec of event ids
// output: vec of pt ids
#[get("/rtq_telii?<event_id_list1>&<event_id_list2>")]
pub fn rtq_telii(db: &State<MongoRepo>, event_id_list1: &str, event_id_list2: &str) -> Result<Json<Vec<String>>, Status> {
  let event_id_list1: Vec<i32> = event_id_list1.split(',')
      .filter_map(|s| s.parse().ok())
      .collect();
  let event_id_list2: Vec<i32> = event_id_list2.split(',')
      .filter_map(|s| s.parse().ok())
      .collect();
  
  let mut or_stmt: Vec<Document> = Vec::new();
  for event_id2 in &event_id_list2 {
    let mut _tmp_event_id1s: Vec<i32> = Vec::new();
    for event_id1 in &event_id_list1 {
      if event_id1 < event_id2 {
        _tmp_event_id1s.push(*event_id1);
      }
    }
    if _tmp_event_id1s.len() > 0 {
      let _stmt = doc! {"e": event_id2,"b1": { "$in": _tmp_event_id1s}};
      or_stmt.push(_stmt);
    }
  }
  for event_id1 in &event_id_list1 {
    let mut _tmp_event_id2s: Vec<i32> = Vec::new();
    for event_id2 in &event_id_list2 {
      if event_id2 < event_id1 {
        _tmp_event_id2s.push(*event_id2);
      }
    }
    if _tmp_event_id2s.len() > 0 {
      let _stmt = doc! {"e": event_id1,"a1": { "$in": _tmp_event_id2s}};
      or_stmt.push(_stmt);
    }
  }

  let mut ap_stmt: Vec<Document> = Vec::new();
  let node_query = doc! {"$or": or_stmt};
  let pipeline = vec![
    doc! {"$match": node_query},
    doc! {"$project": {"_id":0, "PTID":1 }},
    doc! {"$group": {"_id": "$PTID"}}
  ];
  let mut cursor = db.data_col.aggregate(pipeline, None).unwrap();
  let mut results: Vec<String> = Vec::new();
  while let Some(result) = cursor.next() {
    match result {
      Ok(document) => {
        // println!("document: {:?}", document);
        let ptid = document.get_str("_id").unwrap();
        results.push(ptid.to_string());
      }
      Err(e) => {
        println!("Error getting ptid");
        return Err(Status::InternalServerError);
      }
    }
  }
  match results.len() {
    _ => Ok(Json(results)),
    // 0 => Err(Status::NotFound),
  }
}







#[get("/rtq_telii_v3?<event_id_list1>&<event_id_list2>")]
pub fn rtq_telii_v3(db: &State<MongoRepo>, event_id_list1: &str, event_id_list2: &str) -> Result<Json<Vec<String>>, Status> {
  let event_id_list1: Vec<i32> = event_id_list1.split(',')
      .filter_map(|s| s.parse().ok())
      .collect();
  let event_id_list2: Vec<i32> = event_id_list2.split(',')
      .filter_map(|s| s.parse().ok())
      .collect();
  
  let mut or_stmt: Vec<Document> = Vec::new();
  for event_id2 in &event_id_list2 {
    let mut _tmp_event_id1s: Vec<i32> = Vec::new();
    for event_id1 in &event_id_list1 {
      if event_id1 < event_id2 {
        _tmp_event_id1s.push(*event_id1);
      }
    }
    if _tmp_event_id1s.len() > 0 {
      let _stmt = doc! {"event": event_id2,"before": { "$in": _tmp_event_id1s}};
      or_stmt.push(_stmt);
    }
  }
  for event_id1 in &event_id_list1 {
    let mut _tmp_event_id2s: Vec<i32> = Vec::new();
    for event_id2 in &event_id_list2 {
      if event_id2 < event_id1 {
        _tmp_event_id2s.push(*event_id2);
      }
    }
    if _tmp_event_id2s.len() > 0 {
      let _stmt = doc! {"event": event_id1,"after": { "$in": _tmp_event_id2s}};
      or_stmt.push(_stmt);
    }
  }

  let mut ap_stmt: Vec<Document> = Vec::new();
  let node_query = doc! {"$or": or_stmt};
  let pipeline = vec![
    doc! {"$match": node_query},
    doc! {"$project": {"_id":0, "id":1, "bflag": {"$cond": {"if": {"$in": ["$event", &event_id_list2]}, "then": 1, "else": 0}}}},
    doc! {"$group": {"_id": "$bflag", "node_list": {"$addToSet": "$id"}}}
  ];
  let mut cursor = db.temporal_relation_col.aggregate(pipeline, None).unwrap();
  let mut before_nodes: Vec<i32> = Vec::new();
  let mut after_nodes: Vec<i32> = Vec::new();
  while let Some(result) = cursor.next() {
    match result {
      Ok(document) => {
        // println!("document: {:?}", document);
        let bflag = document.get_i32("_id").unwrap();
        if bflag == 1 {
          let node_list = document.get_array("node_list").unwrap();
          for node in node_list {
            before_nodes.push(node.as_i32().unwrap());
          }
        } else {
          let node_list = document.get_array("node_list").unwrap();
          for node in node_list {
            after_nodes.push(node.as_i32().unwrap());
          }
        }
      }
      Err(e) => {
        println!("Error getting node_list");
        return Err(Status::InternalServerError);
      }
    }
  }
  println!("before_nodes: {:?}", before_nodes);
  println!("after_nodes: {:?}", after_nodes);

  // ap_stmt2 = [
  //     { "$match" : { "$or": [{ "event": {"$in": event_id_list1},"before_nodes": { "$in": before_node_list } }, { "event": {"$in": event_id_list1},"after_nodes": { "$in": after_node_list } }] }},
  //     { "$project": { "_id":0, 'PTID':1}},
  //     { "$group": {
  //       "_id":"$PTID"
  //       }
  //     }
  //   ]
  let pipeline2 = vec![
    doc! {"$match": {"$or": [{ "event": {"$in": &event_id_list1},"before_nodes": { "$in": &before_nodes } }, { "event": {"$in": &event_id_list1},"after_nodes": { "$in": &after_nodes } }]}},
    doc! {"$project": {"_id":0, "PTID":1}},
    doc! {"$group": {"_id": "$PTID"}}
  ];
  let mut cursor2 = db.data_col.aggregate(pipeline2, None).unwrap();
  let mut results: Vec<String> = Vec::new();
  while let Some(result) = cursor2.next() {
    match result {
      Ok(document) => {
        // println!("document: {:?}", document);
        let ptid = document.get_str("_id").unwrap();
        results.push(ptid.to_string());
      }
      Err(e) => {
        println!("Error getting ptid");
        return Err(Status::InternalServerError);
      }
    }
  }
  match results.len() {
    _ => Ok(Json(results)),
    // 0 => Err(Status::NotFound),
  }
}
