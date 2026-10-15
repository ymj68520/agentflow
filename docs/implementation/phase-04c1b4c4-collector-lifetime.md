# Scoped collector lifetime — not implemented

No Collector storage, insertion API, lifetime relaxation, serialization or constructor is introduced. Existing CheckedContract/IR/snapshot/root/root-recovery context references and their pointer-identity checks remain unchanged. The diagnostic uses a real shared context object; it does not compare hashes as a substitute.

The later collector must borrow verified source objects or explicitly retain PendingLosslessScopeProjection with their checked associations. It cannot manufacture Complete Scope, persist pointers, or issue a FactKey/ObligationIdentity merely because its local collection validates. This is distinct from the admission blocker: G02 pending Scope is expected by T3; missing registered local predicate semantics is not repaired by a borrow.
