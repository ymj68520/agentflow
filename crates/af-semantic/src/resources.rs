//! Logical resource requirement slots.
//!
//! A [`ResourceSlot`] is **not** a database connection, a credential or any
//! live runtime object: it is the semantic IR's logical declaration that a
//! node will need a resource conforming to a pinned resource-contract schema.
//! Interpreting what that contract permits (authority, effect permissions,
//! residency, binding) belongs to the later Resource/Effect contract stage.
//!
//! Resource slots live in their **own namespace**: a value can never be passed
//! through a resource slot or vice versa, and `ValueSlotId(1)` and
//! `ResourceSlotId(1)` are unrelated objects. There is deliberately no
//! `ResourceHandle`, no binding snapshot and no runtime resolution here.

use crate::ids::ResourceSlotId;
use crate::types::TypeSchemaRef;
use serde::Deserialize;

/// One logical resource requirement slot.
///
/// `requirement_type` pins the resource-contract schema a future binding must
/// conform to. It is a [`TypeSchemaRef`] (a schema-level pin) rather than a
/// `SemanticTypeId`, because a resource contract is a contract schema, not an
/// ordinary value type in the document's type DAG.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceSlot {
    pub id: ResourceSlotId,
    pub requirement_type: TypeSchemaRef,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_slots_deserialize_with_their_own_namespace_id() {
        let slot: ResourceSlot = serde_json::from_str(
            r#"{
                "id": 1,
                "requirement_type": {
                    "specification_id": "agentflow.contracts@1",
                    "type_name": "Postgres",
                    "schema_digest": {
                        "algorithm": "sha256",
                        "domain": "type-schema",
                        "hex": "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
                    }
                }
            }"#,
        )
        .unwrap();
        assert_eq!(slot.id, ResourceSlotId::from_raw(1));
        assert_eq!(
            slot.requirement_type.identity(),
            "agentflow.contracts@1/Postgres"
        );
    }
}
