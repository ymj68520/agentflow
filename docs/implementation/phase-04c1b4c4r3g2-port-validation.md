# Eager port validation and first error

NativeGuardPortResult is closed:unconditional_not_required;valid{ComponentInputPort,EffectGuardInputEquals,full SchemaEntry,StructuralType};missing_port{guard,complete input declarations};unresolved_schema{port,guard,resolve/expand step};type_mismatch{port,guard,resolved SchemaEntry,expanded StructuralType}. SchemaEntry's actual specification_id/type_name/digest/body fields are imported with exact historical provenance. Unresolved results do not invent resolved entries or expanded types.

NativeGuardInputTrace retains separately resolved forward and action outcomes and optional primary_error role. The frozen array evaluates both helpers eagerly;selection then returns the first forward-before-action error. MissingPort/type mismatch are structural V,schema unavailability U. A later observed V does not override an earlier selected U. Input validation precedes zero potential. These are registered capture requirements,not implemented structured hooks.

