//! Getter-only actual compiler identity receipt, no checked construction.
use serde_json::json;
fn main(){
 let c=af_computation::current_compiler_semantics_identity();
 println!("{}",serde_json::to_string_pretty(&json!({
  "analysis_semantics_version":c.analysis_semantics_version,
  "registry_hex":hex::encode(c.registry_content),"rule_registry_hex":hex::encode(c.rule_registry_content),
  "specifications":c.specifications.iter().map(|s|json!({"specification_id":s.specification_id,"logical_path":s.logical_path,"content_sha256":s.content_sha256,"content_hex":hex::encode(&s.content)})).collect::<Vec<_>>(),
  "rules":c.rules.iter().map(|r|json!({"rule_id":r.rule_id,"normative_source":r.normative_source,"source_anchor":r.source_anchor,"short_statement":r.short_statement})).collect::<Vec<_>>()
 })).unwrap());
}
