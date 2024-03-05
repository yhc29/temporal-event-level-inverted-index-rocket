use std::result;

use crate::{models::event::Event, database::mongodb::MongoRepo};
use mongodb::{bson::{doc, Document,Bson}, Collection, results::{self, InsertOneResult}};
use rocket::{http::Status, serde::json::Json, State};
use std::collections::HashSet;

// non-temporal query using elii: event list1 and event list2
// input: event list1: vec of event ids, event list2: vec of event ids
// output: vec of pt ids
#[get("/elii?<event_id_list1>&<event_id_list2>")]
pub fn elii(db: &State<MongoRepo>, event_id_list1: &str, event_id_list2: &str) -> Result<Json<Vec<String>>, Status> {
  let event_id_list1: Vec<i32> = event_id_list1.split(',')
      .filter_map(|s| s.parse().ok())
      .collect();
  let event_id_list2: Vec<i32> = event_id_list2.split(',')
      .filter_map(|s| s.parse().ok())
      .collect();

  let pipeline1 = vec![
    doc! {"$match": {"id": {"$in": event_id_list1}}}
  ];
  let mut cursor = db.elii_col.aggregate(pipeline1, None).unwrap();
  let mut ptid_list1: Vec<String> = Vec::new();
  while let Some(result) = cursor.next() {
    match result {
      Ok(document) => {
        // println!("document: {:?}", document);
        // let ptid = document.get_str("_id").unwrap();
        // results.push(ptid.to_string());
        let new_ptid_list = document.get_array("ptid_list").unwrap();
        for ptid in new_ptid_list {
          ptid_list1.push(ptid.as_str().unwrap().to_string());
        }
      }
      Err(e) => {
        println!("Error getting ptid list");
        return Err(Status::InternalServerError);
      }
    }
  }
  // println!("ptid_list1: {:?}", ptid_list1);

  let pipeline2 = vec![
    doc! {"$match": {"id": {"$in": event_id_list2}}}
  ];
  cursor = db.elii_col.aggregate(pipeline2, None).unwrap();
  let mut ptid_list2: Vec<String> = Vec::new();
  while let Some(result) = cursor.next() {
    match result {
      Ok(document) => {
        // println!("document: {:?}", document);
        // let ptid = document.get_str("_id").unwrap();
        // results.push(ptid.to_string());
        let new_ptid_list = document.get_array("ptid_list").unwrap();
        for ptid in new_ptid_list {
          ptid_list2.push(ptid.as_str().unwrap().to_string());
        }
      }
      Err(e) => {
        println!("Error getting ptid list");
        return Err(Status::InternalServerError);
      }
    }
  }
  // println!("ptid_list2: {:?}", ptid_list2);

  // Convert the Vec<String> to HashSet<String>
  let ptid_set1: HashSet<_> = ptid_list1.into_iter().collect();
  let ptid_set2: HashSet<_> = ptid_list2.into_iter().collect();

  // Find the intersection
  let ptid_list: Vec<_> = ptid_set1.intersection(&ptid_set2).cloned().collect();

  let results = ptid_list;
  match results.len() {
    _ => Ok(Json(results)),
    // 0 => Err(Status::NotFound),
  }
}



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
      let _stmt = doc! {"e": event_id2,"b": { "$in": _tmp_event_id1s}};
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
      let _stmt = doc! {"e": event_id1,"a": { "$in": _tmp_event_id2s}};
      or_stmt.push(_stmt);
    }
  }

  let node_query = doc! {"$or": or_stmt};
  let mut pipeline = vec![
    doc! {"$match": node_query},
    // doc! {"$group": {"_id": "$pg", "ptid_list": {"$addToSet": "$PTID"}}}
    // doc! {"$group": {"_id": "$pg", "ptid_list": {"$sum": 1}}}
  ];
  if return_type == "list" {
    // push group statement
    pipeline.push(doc! {"$group": {"_id": "$pg", "ptid_list": {"$addToSet": "$PTID"}}});
  }else if return_type == "num"{
    pipeline.push(doc! {"$group": {"_id": "$PTID"}});
    pipeline.push(doc! {"$group": {"_id": Bson::Null, "n": {"$sum": 1}}});
  }
  let mut cursor = db.telii_col.aggregate(pipeline, None).unwrap();
  let mut ptid_list: Vec<String> = Vec::new();
  while let Some(result) = cursor.next() {
    match result {
      Ok(document) => {
        // println!("document: {:?}", document);
        if return_type == "list" {
          let new_ptid_list = document.get_array("ptid_list").unwrap();
          for ptid in new_ptid_list {
            ptid_list.push(ptid.as_str().unwrap().to_string());
          }
        }else if return_type == "num" {
          let n = document.get_i32("n").unwrap();
          ptid_list.push(n.to_string());
        }
      }
      Err(e) => {
        println!("Error getting ptid list");
        return Err(Status::InternalServerError);
      }
    }
  }

  let results = ptid_list;





  // let mut before_nodes: Vec<i32> = Vec::new();
  // let mut after_nodes: Vec<i32> = Vec::new();
  // while let Some(result) = cursor.next() {
  //   match result {
  //     Ok(document) => {
  //       // println!("document: {:?}", document);
  //       let bflag = document.get_i32("_id").unwrap();
  //       if bflag == 1 {
  //         let node_list = document.get_array("node_list").unwrap();
  //         for node in node_list {
  //           before_nodes.push(node.as_i32().unwrap());
  //         }
  //       } else {
  //         let node_list = document.get_array("node_list").unwrap();
  //         for node in node_list {
  //           after_nodes.push(node.as_i32().unwrap());
  //         }
  //       }
  //     }
  //     Err(e) => {
  //       println!("Error getting node_list");
  //       return Err(Status::InternalServerError);
  //     }
  //   }
  // }
  // println!("before_nodes: {:?}", before_nodes);
  // println!("after_nodes: {:?}", after_nodes);

  // // ap_stmt2 = [
  // //     { "$match" : { "$or": [{ "event": {"$in": event_id_list1},"before_nodes": { "$in": before_node_list } }, { "event": {"$in": event_id_list1},"after_nodes": { "$in": after_node_list } }] }},
  // //     { "$project": { "_id":0, 'PTID':1}},
  // //     { "$group": {
  // //       "_id":"$PTID"
  // //       }
  // //     }
  // //   ]
  // let pipeline2 = vec![
  //   doc! {"$match": {"$or": [{ "event": {"$in": &event_id_list1},"before_nodes": { "$in": &before_nodes } }, { "event": {"$in": &event_id_list1},"after_nodes": { "$in": &after_nodes } }]}},
  //   doc! {"$project": {"_id":0, "PTID":1}},
  //   doc! {"$group": {"_id": "$PTID"}}
  // ];
  // let mut cursor2 = db.data_col.aggregate(pipeline2, None).unwrap();
  // let mut results: Vec<String> = Vec::new();
  // while let Some(result) = cursor2.next() {
  //   match result {
  //     Ok(document) => {
  //       // println!("document: {:?}", document);
  //       let ptid = document.get_str("_id").unwrap();
  //       results.push(ptid.to_string());
  //     }
  //     Err(e) => {
  //       println!("Error getting ptid");
  //       return Err(Status::InternalServerError);
  //     }
  //   }
  // }
  match results.len() {
    _ => Ok(Json(results)),
    // 0 => Err(Status::NotFound),
  }
}

// relative temporal query with time interval: event list1 before event list2
// input: event list1: vec of event ids, event list2: vec of event ids, gt: i32 time interval greater than in days, lt: i32 time interval less than in days
// output: vec of pt ids
// #[get("/rtqti_telii?<event_id_list1>&<event_id_list2>&<gt>&<lt>")]
// pub fn rtqti_telii(db: &State<MongoRepo>, event_id_list1: &str, event_id_list2: &str, gt: i32, lt: i32) -> Result<Json<Vec<String>>, Status> {
//   let results = vec![];
//   // let ptid_list_json = rtq_telii(db, event_id_list1, event_id_list2);
//   // // covert ptid_list_json to Vec<String>
//   // let ptid_list: Vec<String> = match ptid_list_json {
//   //   Ok(val) => val.0,
//   //   Err(_) => vec![],
//   // };
//   // println!("ptid_list: {:?}", ptid_list.len());

//   match results.len() {
//     _ => Ok(Json(results)),
//     // 0 => Err(Status::NotFound),
//   }
// }
