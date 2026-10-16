//! Isolated source/admission diagnostics; no T3 facts or design vectors run.
#[allow(dead_code,unused_imports)]
mod native {
 include!(concat!(env!("AF_G2_NATIVE_SOURCE_ROOT"), "/crates/af-contract/tests/component_contract_conformance.rs"));
 use serde_json::{Value,json};
 fn ceiling(x:af_effect::OccurrenceCeiling)->Value {match x {af_effect::OccurrenceCeiling::Finite(n)=>json!({"finite":n}),af_effect::OccurrenceCeiling::Unbounded=>json!({"unbounded":null})}}
 fn bounds(x:&OccurrenceBounds)->Value {json!({"lower":x.lower,"upper":ceiling(x.upper)})}
 fn constant(x:&PrimitiveConstant)->Value {match x {PrimitiveConstant::Int64(n)=>json!({"int64":n}),PrimitiveConstant::Unit=>json!({"unit":null}),PrimitiveConstant::Bool(b)=>json!({"bool":b}),_=>panic!("diagnostic supports only actually authored primitive inputs")}}
 fn guard(x:&EffectGuard)->Value {match x {EffectGuard::Unconditional=>json!({"unconditional":null}),EffectGuard::InputEquals(e)=>json!({"input_equals":{"port_no":e.port_no,"expected":constant(&e.expected)}})}}
 fn schema(x:&TypeSchemaRef)->Value {json!({"specification_id":x.specification_id,"type_name":x.type_name,"schema_digest":{"algorithm":x.schema_digest.algorithm,"domain":x.schema_digest.domain,"hex":x.schema_digest.hex}})}
 fn declaration(x:&ComponentContract,c:&StaticContractProofContext<'_>)->Value {serde_json::from_slice(&af_contract::canonicalize_contract(x,c).unwrap().ace1_bytes(c).unwrap()).unwrap()}
 fn action(x:&RecoveryActionIdentity)->Value {
  fn atom(a:&EffectAtom)->Value {json!({"operation":{"catalog":a.operation.catalog,"name":a.operation.name},"resource_selector":{"requirement_port_no":a.resource_selector.requirement_port_no,"scope_bindings":a.resource_selector.scope_bindings.iter().map(|s|json!({"key":s.key,"value":s.value})).collect::<Vec<_>>()},"qualifiers":a.qualifiers.iter().map(|q|format!("{q:?}")).collect::<Vec<_>>()})}
  assert!(x.forward_effect.qualifiers.is_empty()&&x.action_effect.qualifiers.is_empty());
  json!({"forward_effect":atom(&x.forward_effect),"action_kind":x.action_kind.as_str(),"action_effect":atom(&x.action_effect)})
 }
 pub fn run()->Value {
  let mut rows=Vec::new();
  for mode in ["positive","typed_countermodel","unit_unknown","zero","zero_missing_port","type_mismatch","missing_usage","missing_forward","shared_bound_conflict","overflow","first_error_U_then_V","missing_action","missing_port","schema_unresolved"] {
   let mut f=FixtureContext::new();
   let primitive=if mode=="first_error_U_then_V"||mode=="schema_unresolved"{Primitive::UInt32}else if mode=="unit_unknown"{Primitive::Unit}else if mode=="type_mismatch"{Primitive::Bool}else{Primitive::Int64};
   let ty=guard_input_type(&mut f,if primitive==Primitive::Unit{"GuardUnit"}else if primitive==Primitive::Bool{"GuardBool"}else{"GuardInt64"},primitive);
   let condition=if mode=="unit_unknown"{PrimitiveConstant::Unit}else{PrimitiveConstant::Int64(0)};
   let ag=if mode=="positive"||mode=="shared_bound_conflict"||mode=="overflow"{EffectGuard::Unconditional}else{EffectGuard::InputEquals(EffectGuardInputEquals{port_no:if mode=="zero_missing_port"||mode=="first_error_U_then_V"||mode=="missing_port"{2}else{1},expected:condition})};
   let fg=if mode=="first_error_U_then_V"{EffectGuard::InputEquals(EffectGuardInputEquals{port_no:1,expected:PrimitiveConstant::Int64(0)})}else{EffectGuard::Unconditional};
   let mut raw=recovery_contract_with_guards(&mut f,fg,ag,vec![(1,ty)]);
   if mode=="zero"||mode=="zero_missing_port" {raw.recovery_usage_bindings[0].attempts_per_activation=OccurrenceBounds::finite(0,0).unwrap();}
   if mode=="missing_usage" {raw.recovery_usage_bindings.clear();}
   if mode=="missing_action" {let EffectRow::Closed(es)=&mut raw.effects.row else{unreachable!()};es.remove(1);}
   if mode=="missing_forward" {let EffectRow::Closed(es)=&mut raw.effects.row else{unreachable!()};es.remove(0);}
   if mode=="shared_bound_conflict" {
    let a=raw.recovery_action_bindings[0].identity.forward_effect.clone();
    raw.recovery_action_bindings[0].identity.action_effect=a.clone();
    raw.recovery_usage_bindings[0].action=raw.recovery_action_bindings[0].identity.clone();
    let EffectRow::Closed(es)=&mut raw.effects.row else{unreachable!()};es.truncate(1);es[0].bounds=OccurrenceBounds::finite(2,2).unwrap();
   }
   if mode=="overflow" {raw.recovery_usage_bindings[0].activations_per_forward=OccurrenceBounds::finite(1,u64::MAX).unwrap();}
   let c=f.view();
   let canonical=af_contract::canonicalize_contract(&raw,&c).unwrap();
   let report=analyze_contract(&raw,&c);let p=&report.action_guard_compatibility[0];
   let w=p.countermodel().map(|w|json!({
    "source_contract":declaration(w.source_contract(),&c),
    "source_component":{"catalog":w.source_component().catalog,"name":w.source_component().name},
    "recovery_action":action(w.recovery_action()),
    "forward_entry_ordinal":w.forward_entry_ordinal(),"action_entry_ordinal":w.action_entry_ordinal(),
    "forward_guard":guard(w.forward_guard()),"declared_action_guard":guard(w.declared_action_guard()),
    "forward_bounds":bounds(w.forward_bounds()),"activation_upper":ceiling(w.activation_upper()),
    "attempt_upper":ceiling(w.attempt_upper()),"action_upper":ceiling(w.action_upper()),
    "assignments":w.assignments().iter().map(|a|json!({"port_no":a.port_no(),"type_ref":schema(a.type_ref()),"value":constant(a.value())})).collect::<Vec<_>>(),
    "reachability":"WellTypedComponentInputDomain","related_rule_id":w.related_rule_id()
   }));
   if let Some(w)=&w {assert_eq!(w.as_object().unwrap().len(),14);}
   rows.push(json!({"diagnostic_id":mode,"canonical_valid":true,"input":declaration(canonical.contract(),&c),
    "native_source_component":{"catalog":p.source_component().catalog,"name":p.source_component().name},"native_recovery_action":action(p.recovery_action()),
    "component_checked":check_contract(&raw,&c).is_ok(),"guard_verdict":p.verdict().as_str(),
    "basis":format!("{:?}",p.basis()),"countermodel":w,
    "forward_entry_ordinal":p.forward_entry_ordinal(),"action_entry_ordinal":p.action_entry_ordinal(),
    "forward_guard":p.forward_guard().map(guard),"declared_action_guard":p.declared_action_guard().map(guard),
    "action_upper":p.action_upper().map(ceiling),"diagnostic_reason":p.reason()}));
  }
  json!(rows)
 }
}

fn main(){println!("{}",serde_json::to_string_pretty(&native::run()).unwrap());}

