//! Nominal local identity types of the semantic core.
//!
//! The semantic core has **four independent local id namespaces**:
//!
//! ```text
//! SemanticTypeId    semantic type DAG nodes
//! ValueSlotId       SSA value slots
//! ExpressionId      pure expressions
//! ResourceSlotId    logical resource requirement slots
//! ```
//!
//! All four are u32 underneath, and all four are distinct nominal types: the
//! same numeric value in two namespaces never denotes the same object (a
//! value slot 1 and a resource slot 1 may coexist). Positivity (`> 0`) is a
//! *validation* rule rather than a construction rule, so an invalid corpus can
//! carry `0` into the validator and receive a structured error instead of a
//! parse failure.

use serde::Deserialize;
use std::fmt;

macro_rules! local_id {
    ($(#[$docs:meta])* $name:ident, $label:literal) => {
        $(#[$docs])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
        #[serde(transparent)]
        pub struct $name(u32);

        impl $name {
            /// Wraps a raw value. `0` is representable here and is rejected by
            /// the validator, not by the constructor.
            pub const fn from_raw(raw: u32) -> Self {
                Self(raw)
            }

            /// The raw numeric value.
            pub const fn raw(self) -> u32 {
                self.0
            }

            /// `true` when the id is in the valid range (`> 0`).
            pub const fn is_valid(self) -> bool {
                self.0 > 0
            }

            /// The namespace label used in diagnostics and error kinds.
            pub const fn namespace() -> &'static str {
                $label
            }
        }

        impl From<u32> for $name {
            fn from(raw: u32) -> Self {
                Self(raw)
            }
        }

        impl From<$name> for u32 {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}#{}", $label, self.0)
            }
        }
    };
}

local_id!(
    /// Identity of one semantic type node in the document-local type DAG.
    SemanticTypeId,
    "semantic-type"
);

local_id!(
    /// Identity of one immutable SSA value slot.
    ValueSlotId,
    "value-slot"
);

local_id!(
    /// Identity of one pure expression definition.
    ExpressionId,
    "expression"
);

local_id!(
    /// Identity of one logical resource requirement slot.
    ResourceSlotId,
    "resource-slot"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespaces_are_nominal_and_disjoint_at_the_type_level() {
        // The same numeric value in two namespaces is two distinct objects.
        let value = ValueSlotId::from_raw(1);
        let resource = ResourceSlotId::from_raw(1);
        assert_eq!(value.raw(), resource.raw());
        assert_eq!(ValueSlotId::namespace(), "value-slot");
        assert_eq!(ResourceSlotId::namespace(), "resource-slot");
        // (ValueSlotId == ResourceSlotId is a compile-time error.)
    }

    #[test]
    fn zero_is_representable_but_invalid() {
        assert!(!SemanticTypeId::from_raw(0).is_valid());
        assert!(SemanticTypeId::from_raw(1).is_valid());
        assert!(!ExpressionId::from_raw(0).is_valid());
    }

    #[test]
    fn conversions_and_display() {
        let id = ExpressionId::from(7u32);
        assert_eq!(id.raw(), 7);
        assert_eq!(u32::from(id), 7);
        assert_eq!(id.to_string(), "expression#7");
    }
}
