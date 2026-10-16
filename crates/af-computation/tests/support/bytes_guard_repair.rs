//! Actual-only typed F1 lowering projection; no Expected or G01 encoding.
use af_core::Value as Core;
use serde_json::{Value, json};

fn field<'a>(v: &'a Core, name: &str) -> &'a Core {
    let Core::Record(fields) = v else {
        panic!("record required")
    };
    &fields.iter().find(|(n, _)| n == name).unwrap().1
}
fn variant<'a>(v: &'a Core, tag: &str) -> &'a Core {
    let Core::Variant { case, value } = v else {
        panic!("variant required")
    };
    assert_eq!(case, tag);
    value
}
fn constant(v: &Core) -> &Core {
    let Core::List(entries) = variant(field(v, "row"), "closed") else {
        panic!("list required")
    };
    field(
        variant(field(&entries[0], "guard"), "input_equals"),
        "expected",
    )
}
fn typed_payload(v: &Core) -> Value {
    let Core::Variant { case, value } = v else {
        panic!("constant required")
    };
    match value.as_ref() {
        Core::Bytes(raw) => {
            assert_eq!(case, "bytes");
            json!({"kind":"bytes","value":raw})
        }
        Core::String(text) => {
            assert_eq!(case, "string");
            json!({"kind":"string","value":text})
        }
        _ => panic!("F1 covers bytes/string only"),
    }
}
pub fn lowering(row: &Value, modules: &af_idl::CompiledModuleSet) -> Value {
    let raw: af_effect::EffectSignature = serde_json::from_value(json!({"row":{"closed":[{
        "atom":{"operation":{"catalog":"agentflow.effects@1","name":"read"},
            "resource_selector":{"requirement_port_no":1,"scope_bindings":[]},"qualifiers":[]},
        "bounds":{"lower":0,"upper":{"finite":1}},
        "guard":{"input_equals":{"port_no":1,"expected":row["constant"]}}
    }]}}))
    .unwrap();
    match raw.to_value() {
        Ok(tree) => {
            let wire = raw.canonical_bytes(modules).unwrap();
            assert_eq!(wire, raw.canonical_bytes(modules).unwrap());
            let shape = modules
                .shape_of("agentflow.effect.signature@1/EffectSignature")
                .unwrap();
            let decoded = af_core::decode(&shape, &wire).unwrap();
            assert_eq!(wire, af_core::encode(&shape, &decoded).unwrap());
            let Core::Variant { value, .. } = constant(&tree) else {
                panic!("constant")
            };
            let scalar_shape = match value.as_ref() {
                Core::Bytes(_) => af_core::Shape::Bytes,
                Core::String(_) => af_core::Shape::String,
                _ => panic!("payload"),
            };
            json!({"case_id":row["case_id"],"input_constant":row["constant"],"canonical_admission":true,
                "typed_payload":typed_payload(constant(&tree)),"decoded_payload":typed_payload(constant(&decoded)),
                "scalar_ace1_utf8":String::from_utf8(af_core::encode(&scalar_shape,value).unwrap()).unwrap(),
                "signature_ace1_utf8":String::from_utf8(wire).unwrap(),"rejection_kind":null})
        }
        Err(e) => {
            assert!(raw.canonical_bytes(modules).is_err());
            json!({"case_id":row["case_id"],"input_constant":row["constant"],"canonical_admission":false,
                "typed_payload":null,"decoded_payload":null,"scalar_ace1_utf8":null,"signature_ace1_utf8":null,
                "rejection_kind":e.kind()})
        }
    }
}
