//! Nominal local identity types of the semantic IR.
//!
//! The semantic IR adds **three id namespaces and three numbering spaces** on
//! top of the four Phase 3A namespaces:
//!
//! ```text
//! ComputationId     nodes of the structured control tree
//! RunParameterId    static workflow input declarations
//! ContractRefId     closed contract table entries
//! PortNo            computation input/output port numbers
//! BranchOrdinal     select/parallel/race case and branch ordinals
//! HandlerOrdinal    handle clause ordinals
//! ```
//!
//! All six are u32 underneath and pairwise-distinct nominal types: the same
//! numeric value in two namespaces never denotes the same object. Positivity
//! (`> 0`) is a *validation* rule rather than a construction rule, so an
//! invalid corpus can carry `0` into the validator and receive a structured
//! error instead of a parse failure.

use serde::Deserialize;
use std::fmt;

macro_rules! ir_id {
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

ir_id!(
    /// Identity of one computation node of the structured control tree.
    ComputationId,
    "computation"
);

ir_id!(
    /// Identity of one static run parameter declaration.
    RunParameterId,
    "run-parameter"
);

ir_id!(
    /// Identity of one closed contract table entry.
    ContractRefId,
    "contract-ref"
);

ir_id!(
    /// A computation port number (unique within one computation's port list).
    PortNo,
    "port"
);

ir_id!(
    /// A select case, parallel branch or race branch ordinal (semantic ABI).
    BranchOrdinal,
    "branch"
);

ir_id!(
    /// A handle clause ordinal.
    HandlerOrdinal,
    "handler"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespaces_are_nominal_and_disjoint_at_the_type_level() {
        // The same numeric value in two namespaces is two distinct objects.
        let computation = ComputationId::from_raw(1);
        let value_slot = af_semantic::ValueSlotId::from_raw(1);
        assert_eq!(computation.raw(), value_slot.raw());
        assert_eq!(ComputationId::namespace(), "computation");
        assert_eq!(PortNo::namespace(), "port");
        assert_eq!(HandlerOrdinal::namespace(), "handler");
        // (ComputationId == ValueSlotId is a compile-time error.)
    }

    #[test]
    fn zero_is_representable_but_invalid() {
        assert!(!ComputationId::from_raw(0).is_valid());
        assert!(!PortNo::from_raw(0).is_valid());
        assert!(BranchOrdinal::from_raw(1).is_valid());
    }

    #[test]
    fn conversions_and_display() {
        let id = ContractRefId::from(7u32);
        assert_eq!(id.raw(), 7);
        assert_eq!(u32::from(id), 7);
        assert_eq!(id.to_string(), "contract-ref#7");
    }
}
