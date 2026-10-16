//! Actual-only local test projection. This neither computes Expected nor
//! claims to encode model@3 or a complete Scope/Fragment.
use af_contract::guard_observation::*;
use af_contract::{
    ActionGuardCompatibility, ComponentContract, ComponentInputPort, GuardCompatibilityBasis,
    RecoveryActionIdentity, RecoveryUsageBinding, StaticContractProofContext,
    TypedGuardCountermodel,
};
use af_effect::{EffectAtom, EffectEntry, EffectGuard, OccurrenceBounds, OccurrenceCeiling};
use af_idl::CanonicalTypeSchema;
use af_semantic::{PrimitiveConstant, SchemaEntry, StructuralType, TypeSchemaRef};
use serde_json::{Value, json};

fn ceiling(x: OccurrenceCeiling) -> Value {
    match x {
        OccurrenceCeiling::Finite(n) => json!({"finite":n}),
        OccurrenceCeiling::Unbounded => json!({"unbounded":null}),
    }
}
fn bounds(x: &OccurrenceBounds) -> Value {
    json!({"lower":x.lower,"upper":ceiling(x.upper)})
}
fn constant(x: &PrimitiveConstant) -> Value {
    match x {
        PrimitiveConstant::Unit => json!({"unit":null}),
        PrimitiveConstant::Null => json!({"null":null}),
        PrimitiveConstant::Bool(v) => json!({"bool":v}),
        PrimitiveConstant::Int64(v) => json!({"int64":v}),
        PrimitiveConstant::Decimal(v) => json!({"decimal":v.as_str()}),
        PrimitiveConstant::String(v) => json!({"string":v}),
        PrimitiveConstant::Bytes(v) => json!({"bytes":v}),
        PrimitiveConstant::Timestamp(v) => json!({"timestamp":v.as_micros()}),
        PrimitiveConstant::Duration(v) => json!({"duration":v.as_micros()}),
    }
}
fn guard(x: &EffectGuard) -> Value {
    match x {
        EffectGuard::Unconditional => json!({"unconditional":null}),
        EffectGuard::InputEquals(g) => {
            json!({"input_equals":{"port_no":g.port_no,"expected":constant(&g.expected)}})
        }
    }
}
fn atom(x: &EffectAtom) -> Value {
    use af_effect::EffectQualifier;
    json!({"operation":{"catalog":x.operation.catalog,"name":x.operation.name},
        "resource_selector":{"requirement_port_no":x.resource_selector.requirement_port_no,
            "scope_bindings":x.resource_selector.scope_bindings.iter().map(|s|json!({"key":s.key,"value":s.value})).collect::<Vec<_>>()},
            "qualifiers":x.qualifiers.iter().map(|q|match q {EffectQualifier::Idempotency(v)=>json!({"idempotency":match v {af_effect::EffectIdempotencyQualifier::Natural=>"natural",af_effect::EffectIdempotencyQualifier::None=>"none"}})}).collect::<Vec<_>>()})
}
fn identity(x: &RecoveryActionIdentity) -> Value {
    json!({"forward_effect":atom(&x.forward_effect),"action_kind":x.action_kind.as_str(),"action_effect":atom(&x.action_effect)})
}
fn entry(x: &EffectEntry) -> Value {
    json!({"atom":atom(&x.atom),"bounds":bounds(&x.bounds),"guard":guard(&x.guard)})
}
fn schema_ref(x: &TypeSchemaRef) -> Value {
    json!({"specification_id":x.specification_id,"type_name":x.type_name,
        "schema_digest":{"algorithm":x.schema_digest.algorithm,"domain":x.schema_digest.domain,"hex":x.schema_digest.hex}})
}
fn port(x: &ComponentInputPort) -> Value {
    json!({"port_no":x.port_no,"type":schema_ref(&x.type_ref)})
}
fn schema_entry(x: &SchemaEntry) -> Value {
    let actual = CanonicalTypeSchema {
        schema: x.specification_id.clone(),
        name: x.type_name.clone(),
        body: x.body.clone(),
    };
    let bytes = actual.ace1_bytes().unwrap();
    let tree: Value = serde_json::from_slice(&bytes).unwrap();
    json!({"specification_id":x.specification_id,"type_name":x.type_name,
        "digest":{"algorithm":x.digest.alg().as_str(),"domain":x.digest.domain().as_str(),"hex":x.digest.hex()},"body":tree["body"]})
}
fn structural(x: &StructuralType) -> Value {
    match x {
        StructuralType::Primitive(p) => json!({"primitive":p.keyword()}),
        StructuralType::Schema(s) => json!({"schema":schema_ref(s)}),
        StructuralType::List(s) => json!({"list":structural(s)}),
        StructuralType::MapString(s) => json!({"map_string":structural(s)}),
        StructuralType::Option(s) => json!({"option":structural(s)}),
    }
}
fn usage(x: &RecoveryUsageBinding) -> Value {
    use af_usage::UsageUpperBound;
    json!({"action":identity(&x.action),"activations_per_forward":bounds(&x.activations_per_forward),
        "attempts_per_activation":bounds(&x.attempts_per_activation),"per_attempt_usage":{"limits":x.per_attempt_usage.limits.iter().map(|l|json!({
            "meter":{"catalog":l.meter.catalog,"name":l.meter.name},"phase":l.phase.as_str(),
            "upper":match l.upper {UsageUpperBound::Finite(n)=>json!({"finite":n}),UsageUpperBound::Unbounded=>json!({"unbounded":null})}
        })).collect::<Vec<_>>()}})
}
fn resolution<T>(x: &ObservedResolution<T>, f: impl FnOnce(&T) -> Value) -> Value {
    match x {
        ObservedResolution::NotQueried => json!({"not_queried":null}),
        ObservedResolution::Absent => json!({"absent":null}),
        ObservedResolution::Present(x) => json!({"present":f(x)}),
    }
}
fn peer(x: &ObservedPeer) -> Value {
    json!({"binding_ordinal":x.binding_ordinal,"binding":{"identity":identity(&x.binding.identity)},
        "usage":resolution(&x.usage,|(n,u)|json!({"ordinal":n,"binding":usage(u)}))})
}
fn failure(x: &ObservedBoundFailure) -> Value {
    match x {
        ObservedBoundFailure::SameAtomProductOverflow {
            binding_ordinal,
            activation_upper,
            attempt_upper,
        } => {
            json!({"same_atom_product_overflow":{"binding_ordinal":binding_ordinal,"activation_upper":ceiling(*activation_upper),"attempt_upper":ceiling(*attempt_upper)}})
        }
        ObservedBoundFailure::SameAtomSumOverflow {
            processed_binding_ordinals,
            before,
            next,
        } => {
            json!({"same_atom_sum_overflow":{"processed_binding_ordinals":processed_binding_ordinals,"before":ceiling(*before),"next":ceiling(*next)}})
        }
        ObservedBoundFailure::SameAtomDivisorOverflow { sum } => {
            json!({"same_atom_divisor_overflow":{"sum":ceiling(*sum)}})
        }
        ObservedBoundFailure::SameAtomLowerConflict {
            declared_bounds,
            sum,
            divisor,
            effective_upper,
        } => {
            json!({"same_atom_lower_conflict":{"declared_bounds":bounds(declared_bounds),"sum":ceiling(*sum),"divisor":ceiling(*divisor),"effective_upper":ceiling(*effective_upper)}})
        }
        ObservedBoundFailure::ActionUpperOverflow { step, left, right } => {
            json!({"action_upper_overflow":{"step":match step {ObservedMultiplyStep::ForwardTimesActivation=>"forward_times_activation",ObservedMultiplyStep::PartialTimesAttempt=>"partial_times_attempt"},"left":ceiling(*left),"right":ceiling(*right)}})
        }
    }
}
fn bound_trace(x: &ObservedBounds) -> Value {
    json!({"declared_forward_bounds":bounds(&x.declared_forward_bounds),"consulted_peer_bindings":x.consulted_peer_bindings.iter().map(peer).collect::<Vec<_>>(),
        "arithmetic":x.arithmetic.iter().map(|e|json!({"kind":match e.kind {
            ObservedArithmeticKind::PeerProduct=>"peer_product",ObservedArithmeticKind::PeerSum=>"peer_sum",ObservedArithmeticKind::Divisor=>"divisor",
            ObservedArithmeticKind::ForwardTimesActivation=>"forward_times_activation",ObservedArithmeticKind::PartialTimesAttempt=>"partial_times_attempt"},
            "binding_ordinal":e.binding_ordinal,"left":ceiling(e.left),"right":ceiling(e.right),"result":e.result.map(ceiling)})).collect::<Vec<_>>(),
        "sum":resolution(&x.sum,|c|ceiling(*c)),"divisor":resolution(&x.divisor,|c|ceiling(*c)),
        "effective_upper":resolution(&x.effective_upper,|c|ceiling(*c)),"forward_times_activation":resolution(&x.forward_times_activation,|c|ceiling(*c)),
        "partial_times_attempt":resolution(&x.partial_times_attempt,|c|ceiling(*c)),"action_upper":resolution(&x.action_upper,|c|ceiling(*c)),
        "failure":x.failure.as_ref().map(failure),"unisolated":x.unisolated.as_ref().map(|u|match u {
            ObservedUnisolated::ForeignForward(p)=>json!({"foreign_forward":peer(p)}),ObservedUnisolated::MissingPeerUsage(p)=>json!({"missing_peer_usage":peer(p)})})})
}
fn port_result(x: &ObservedPortResult) -> Value {
    let equals = |g: &af_effect::EffectGuardInputEquals| json!({"port_no":g.port_no,"expected":constant(&g.expected)});
    match x {
        ObservedPortResult::UnconditionalNotRequired => json!({"unconditional_not_required":null}),
        ObservedPortResult::Valid {
            port: p,
            guard: g,
            resolved,
            expanded,
        } => {
            json!({"valid":{"port":port(p),"guard":equals(g),"resolved":schema_entry(resolved),"expanded":structural(expanded)}})
        }
        ObservedPortResult::MissingPort {
            guard: g,
            input_declarations,
        } => {
            json!({"missing_port":{"guard":equals(g),"input_declarations":input_declarations.iter().map(port).collect::<Vec<_>>()}})
        }
        ObservedPortResult::UnresolvedSchema {
            port: p,
            guard: g,
            step,
        } => {
            json!({"unresolved_schema":{"port":port(p),"guard":equals(g),"step":match step {ObservedSchemaStep::Resolve=>"resolve",ObservedSchemaStep::Expand=>"expand"}}})
        }
        ObservedPortResult::TypeMismatch {
            port: p,
            guard: g,
            resolved,
            expanded,
        } => {
            json!({"type_mismatch":{"port":port(p),"guard":equals(g),"resolved":schema_entry(resolved),"expanded":structural(expanded)}})
        }
    }
}
fn declaration(x: &ComponentContract, c: &StaticContractProofContext<'_>) -> Value {
    serde_json::from_slice(
        &af_contract::canonicalize_contract(x, c)
            .unwrap()
            .ace1_bytes(c)
            .unwrap(),
    )
    .unwrap()
}
fn countermodel(x: &TypedGuardCountermodel, c: &StaticContractProofContext<'_>) -> Value {
    json!({"source_contract":declaration(x.source_contract(),c),
        "source_component":{"catalog":x.source_component().catalog,"name":x.source_component().name},
        "recovery_action":identity(x.recovery_action()),"forward_entry_ordinal":x.forward_entry_ordinal(),"action_entry_ordinal":x.action_entry_ordinal(),
        "forward_guard":guard(x.forward_guard()),"declared_action_guard":guard(x.declared_action_guard()),"forward_bounds":bounds(x.forward_bounds()),
        "activation_upper":ceiling(x.activation_upper()),"attempt_upper":ceiling(x.attempt_upper()),"action_upper":ceiling(x.action_upper()),
        "assignments":x.assignments().iter().map(|a|json!({"port_no":a.port_no(),"type_ref":schema_ref(a.type_ref()),"value":constant(a.value())})).collect::<Vec<_>>(),
        "reachability":match x.reachability() {af_contract::GuardCountermodelReachability::WellTypedComponentInputDomain=>"well_typed_component_input_domain"},"related_rule_id":x.related_rule_id()})
}
fn record(x: &ActionGuardCompatibility, c: &StaticContractProofContext<'_>) -> Value {
    json!({"source_component":{"catalog":x.source_component().catalog,"name":x.source_component().name},
        "recovery_action":identity(x.recovery_action()),"forward_entry_ordinal":x.forward_entry_ordinal(),"action_entry_ordinal":x.action_entry_ordinal(),
        "forward_guard":x.forward_guard().map(guard),"declared_action_guard":x.declared_action_guard().map(guard),"action_upper":x.action_upper().map(ceiling),
        "verdict":x.verdict().as_str(),"basis":match x.basis() {
            GuardCompatibilityBasis::FrozenGuardContainment=>"frozen_guard_containment",GuardCompatibilityBasis::ZeroPossibleActions=>"zero_possible_actions",
            GuardCompatibilityBasis::TypedCountermodel=>"typed_countermodel",GuardCompatibilityBasis::InsufficientEvidence=>"insufficient_evidence"},
        "countermodel":x.countermodel().map(|w|countermodel(w,c)),"reason":x.reason()})
}
fn run(x: &ObservedGuardRun, c: &StaticContractProofContext<'_>) -> Value {
    json!({"binding_ordinal":x.binding_ordinal,"identity":identity(&x.identity),"row_kind":match x.row_kind {
        ObservedEffectRowKind::Closed=>"closed",ObservedEffectRowKind::OpenKnown=>"open_known",ObservedEffectRowKind::Unknown=>"unknown"},
        "forward":resolution(&x.forward,|(n,e)|json!({"ordinal":n,"entry":entry(e)})),
        "action":resolution(&x.action,|(n,e)|json!({"ordinal":n,"entry":entry(e)})),
        "usage":resolution(&x.usage,|(n,u)|json!({"ordinal":n,"binding":usage(u)})),"bounds":x.bounds.as_ref().map(bound_trace),
        "forward_input":resolution(&x.forward_input,port_result),"action_input":resolution(&x.action_input,port_result),
        "primary_error":x.primary_error.map(|r|match r {ObservedGuardRole::Forward=>"forward",ObservedGuardRole::Action=>"action"}),
        "containment":resolution(&x.containment,|b|json!(b)),"branch":x.branch.map(|b|match b {
            ObservedGuardBranch::MissingEntries=>"missing_entries",ObservedGuardBranch::MissingUsage=>"missing_usage",
            ObservedGuardBranch::UnisolatedForward=>"unisolated_forward",ObservedGuardBranch::SameAtomBoundsFailure=>"same_atom_bounds_failure",
            ObservedGuardBranch::ActionUpperOverflow=>"action_upper_overflow",ObservedGuardBranch::MissingPort=>"missing_port",
            ObservedGuardBranch::SchemaUnresolved=>"schema_unresolved",ObservedGuardBranch::TypeMismatch=>"type_mismatch",
            ObservedGuardBranch::ZeroPotential=>"zero_potential",ObservedGuardBranch::FrozenContainment=>"frozen_containment",
            ObservedGuardBranch::TypedCountermodel=>"typed_countermodel",ObservedGuardBranch::NoCountermodel=>"no_countermodel"}),
        "record":x.record.as_ref().map(|r|record(r,c))})
}
pub fn observation(x: &CanonicalGuardObservation<'_, '_>) -> Value {
    json!({"canonical_admission":true,"canonical_declaration":serde_json::from_slice::<Value>(x.canonical_bytes()).unwrap(),
        "canonical_bytes_hex":hex::encode(x.canonical_bytes()),"runs":x.runs().iter().map(|r|run(r,x.context())).collect::<Vec<_>>()})
}
pub fn wire_numbers(x: &mut Value) {
    match x {
        Value::Number(n) => *x = Value::String(n.to_string()),
        Value::Array(a) => a.iter_mut().for_each(wire_numbers),
        Value::Object(o) => o.values_mut().for_each(wire_numbers),
        _ => {}
    }
}
