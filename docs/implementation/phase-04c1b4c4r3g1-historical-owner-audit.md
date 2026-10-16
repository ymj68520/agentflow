# Historical owner and current admission adjudication

## Observed ownership chain

historical native declaration → exact model@1 registered source/content → unchanged inherited_registered_model payload inside model@2 → no selected active materialized alias for the five guard roots → current metadata lookup rejects → no presently confirmed complete native typed projection. These are separate states,not supported:boolean.

All five are Declared=yes,RegisteredSource=yes,IncludedHistoricalPayload=yes,NativeSemanticallyApplicable=yes. ActiveMaterialized=no,active named-ref reachability=no,current tested claimability=no,usable complete current projection=not established. This does NOT say they were never registered or cannot be represented by a future explicit import.

## Exact implementation predicate

For ModelType and ModelRef,check_named_constraints accepts (owner==model@2 AND types contains exact name) OR types contains exact owner+'/'+name. It does not consult historical snapshots for that lookup. Names are source-qualified where actually catalogued;af-contract::NativeName is not an AFIDL module/name merely because the crate implements ComponentContract.

The [complete56-row matrix](phase-04c1b4c4r3g1-owner-matrix.json) records schema,owner,name,full claim and actual Result.8 rows accept metadata-only unit;48 reject. The five native names under current,historical,ComponentContract AFIDL and unknown owners all reject for both schemas. Controls:current Scope accepts,historical Scope rejects,current af-semantic::TypeSchemaRef accepts,agentflow.semantic.core@1/TypeSchemaRef accepts under its actual owner,current model's explicitly qualified imported key also accepts,ComponentContract AFIDL/ComponentContract and unknown names reject in this particular materialized catalogue. Four DeclaredStep nested controls show no hidden nested admission for the five missing roots. Ordinary Rust nesting in a retained native snapshot is not an active ModelType import.

## Is historical Scope rejection a bug?

It is a **safe fail-closed staging availability/coverage limitation**,not proof of an absent historical type and not an unsafe capability bypass. The logical model permits exact inherited ownership while the non-authoritative metadata helper implements only the current materialized view plus actual source-qualified keys. It is not a complete historical materializer or CompleteAndValid decoder. Record the gap separately;do not infer that every historical claim should pass.

In particular historical Scope/Fact/Source differ from current shapes. Allowing owner=model@1 and then validating against current Scope merely by the same name would be WRONG. An eventual implementation must either validate the exact frozen historical descriptor closure with its exact content/owner,or use a separately registered,explicit per-type lossless import/alias. It must preserve unknown-owner/name/field/variant rejection and current-only ANALYSIS routing. Type-level selected imports do not authorize old checker editions,old reports,or capability issuance.

No metadata policy is changed here. Current-owner native names correctly have no active binding;historical native references additionally lack a current usable selected projection. Repairing a legitimate owner/materialization omission is a separate implementation decision after the exact import policy is authorized,and still does not add native Predicate/Source/Origin semantics. A broad historical-owner whitelist is rejected.
