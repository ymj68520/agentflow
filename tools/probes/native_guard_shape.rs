//! Non-authoritative metadata validation of actual diagnostic presentations.
use serde_json::{Value,json};
fn main(){
 let path=std::env::args().nth(1).expect("actual diagnostic presentation path");
 let rows:Value=serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
 let mut out=Vec::new();
 for r in rows.as_array().unwrap(){
  af_computation::validate_registered_model_shape(r["metadata_type"].as_str().unwrap(),&r["metadata_value"]).unwrap();
  out.push(json!({"diagnostic_id":r["diagnostic_id"],"canonical_valid":r["canonical_valid"],"component_checked":r["component_checked"],"complete_native_record_shape_validated":true,"capture":false,"capability":false}));
 }
 assert!(!af_computation::current_registered_analysis_edition().can_issue_capability());
 println!("{}",serde_json::to_string_pretty(&out).unwrap());
}
