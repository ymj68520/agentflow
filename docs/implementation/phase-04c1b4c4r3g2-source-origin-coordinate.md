# IR-free source, target, coordinate and origin

Source appends native_guard_subject{subject:Node<NativeRecoveryGuardSubject>} and native_guard_binding{subject,occurrence:NativeGuardOccurrence}. Target appends corresponding subject and binding{subject,action_binding_ordinal,action_identity}. Coordinate.native_guard retains subject,closed clause,optional occurrence,optional guard_role,and NativeGuardMemberPath. Subject-level absence applies only to canonical/context cases;per-binding cases require the genuine occurrence.

NativeGuardMemberPath is exactly subject,binding_identity{ordinal},effect_guard{closed/open_known row kind,entry ordinal},input_type{input ordinal,port_no},usage_profile{ordinal}. Origin uses the actual canonical declaration's recovery_action_bindings[i] and actual forward/action reference application;resolved E selection belongs to typed coordinate. It uses no fake ClosedField for enum variants,callerIR,node0,clock,UUID,probe ID,Rust line or root observation path. source_ordinal is actual binding ordinal;mapping_ordinal is absent;assertion_path is known empty. Repeated evaluation of one complete declaration/context/application does not mint a new origin.

