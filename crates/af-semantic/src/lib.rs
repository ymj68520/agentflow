//! `af-semantic` — the AgentFlow workflow semantic core (Phase 3A).
//!
//! This crate freezes the **semantic data ABI** that every later layer
//! (Semantic IR, Plan, Task Contract, Run Parameters, Value Store, Reducer)
//! will reference:
//!
//! 1. the 9-member workflow primitive universe ([`SemanticPrimitiveType`]),
//!    deliberately narrower than the 13 AFIDL machine primitives;
//! 2. [`SemanticTypeNode`] — a document-local semantic **type DAG** expressing
//!    arbitrary finite nesting without recursive AFIDL schemas;
//! 3. [`TypeSchemaRef`] — an exact, digest-pinned reference to a named AFIDL
//!    type, resolved only against an explicit [`TypeSchemaUniverse`];
//! 4. [`ValueSlot`] — immutable SSA value slots with their own id namespace;
//! 5. [`ExpressionDef`] / [`ExpressionBody`] — a closed, pure, total-once-
//!    well-typed expression language;
//! 6. [`ResourceSlot`] — logical resource requirement slots, separate from
//!    value slots.
//!
//! [`SemanticCoreDocument`] bundles the four tables and [`SemanticCoreDocument::check`]
//! validates them; [`CheckedDocument`] is the only way to obtain resolved
//! types, and it can lower expressions to ACE1 `(Shape, Value)` pairs.
//!
//! Deliberately absent: workflow computation nodes (INVOKE/AWAIT/SEQUENCE/
//! SELECT/PARALLEL/RACE/ITERATE/RETRY/TIMEOUT/HANDLE/SCOPE/COMPENSATION_SCOPE/
//! CALL_SUBFLOW), `SemanticIRDocument`, executable plans, journal, reducer,
//! runtime commands, resource handles and any runtime identity. This crate
//! depends only on `af-core` and `af-idl` — never on registry tooling.
//!
//! No new HashV1 domain is introduced: type identity comes from
//! `TypeSchemaDigest`, and lowering produces ordinary ACE1 bytes.

pub mod document;
pub mod error;
pub mod expressions;
pub mod ids;
pub mod resources;
pub mod types;
pub mod universe;
pub mod values;

pub use document::{CheckedDocument, SemanticCoreDocument};
pub use error::SemanticError;
pub use expressions::{ExprTable, ExpressionBody, ExpressionDef, MapEntryExpr, RecordFieldExpr};
pub use ids::{ExpressionId, ResourceSlotId, SemanticTypeId, ValueSlotId};
pub use resources::ResourceSlot;
pub use types::{
    DigestRefValue, SemanticPrimitiveType, SemanticTypeNode, SemanticTypeNodeBody, StructuralType,
    TypeSchemaRef,
};
pub use universe::{SchemaEntry, TypeSchemaUniverse};
pub use values::{PrimitiveConstant, ValueSlot};
