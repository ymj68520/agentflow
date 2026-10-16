# Forty-eight independent design vectors

**DRAFT — NOT REGISTERED.**48 independently specified scenarios/laws,all Actual Status=Not Executed. Distribution:16 native semantics,12 source/ordinal/origin/context,12 witness/owner/descriptor,8 root/edition/capability. These design IDs are NOT registered conformance CaseIds. They do not reuse the prior R1's48 vectors,R2's173 staging cases,or G1's separate isolated diagnostics as passes.

|Design ID|Group / law|Declared scenario|Independent Expected|Actual Status|
|---|---|---|---|---|
|G1-NG-001|Native semantics: Unconditional positive|Canonical C,Forward=true,Action=true,positive local potential|source/context/readiness P;frozen containment P;native evaluation P/FrozenGuardContainment;no assignment witness|Not Executed|
|G1-NG-002|Native semantics: Int64 counterexample|Int64 port1,Forward=true,Action=(p1==0),positive upper|frozen containment false/U;assignment p1=1 typed-valid;native evaluation V/TypedCountermodel;full check rejects|Not Executed|
|G1-NG-003|Native semantics: Equal conditional|Forward=(p1==0),Action=(p1==0),valid exact type|frozen containment P;native P;no fabricated countermodel|Not Executed|
|G1-NG-004|Native semantics: Forward narrower|Forward=(p1==0),Action=true|native P under frozen direction;never reverse F/A|Not Executed|
|G1-NG-005|Native semantics: Different guard ports|Forward=(p1==0),Action=(p2==0),both Int64|retain two exact typed assignments chosen by frozen builder;F true/A false gives V|Not Executed|
|G1-NG-006|Native semantics: Same port different constants|Forward=(p1==0),Action=(p1==1)|retain p1=0,do not overwrite it with action alternate;V|Not Executed|
|G1-NG-007|Native semantics: Zero potential|Valid guard ports,derived action upper0|native P/ZeroPossibleActions;containment not executed;no runtime eligibility or whole-contract success inference|Not Executed|
|G1-NG-008|Native semantics: Zero with missing port|upper0 but Action references absent port|input validation precedes zero;native V/InsufficientEvidence with missing-port structural witness,no input countermodel|Not Executed|
|G1-NG-009|Native semantics: Unit domain unknown|Forward=true,Action=(unit_port==Unit)|no alternate Unit value;frozen containment false and no countermodel=>U,not P or V|Not Executed|
|G1-NG-010|Native semantics: Missing E entry|Exact binding present but Forward E not found|actual absent resolution/optional ordinal;native U,not fabricated Unconditional/P|Not Executed|
|G1-NG-011|Native semantics: Missing usage|E entries present;matching usage absent|native U;bounds and guard validations not queried|Not Executed|
|G1-NG-012|Native semantics: Unknown schema expansion|Known exact machine-only schema unsupported by workflow primitive expansion|canonical admission must first be established;native input UnresolvedSchema U;never fabricated domain or assignment|Not Executed|
|G1-NG-013|Native semantics: Type mismatch|Bool input schema,Int64 guard constant|native V/InsufficientEvidence and typed constant-type mismatch cause;no well-typed assignment claimed|Not Executed|
|G1-NG-014|Native semantics: Action multiplication overflow|finite product exceeds u64 at actual checked step|native V with typed arithmetic failure;no saturation,partial success or guessed countermodel|Not Executed|
|G1-NG-015|Native semantics: Shared ceiling conflict|Same full Forward/action atom,C=2,lower2,multiplier2|divisor3 gives recovered upper0<lower2;native V premise failure before guard containment|Not Executed|
|G1-NG-016|Native semantics: First-error ordering|Forward schema expansion U,action port missing V;canonical admission verified|retain both eager helper outcomes;native primary selects forward U rather than max V|Not Executed|
|G1-NG-017|Source,ordinal,origin,context: IR-free source|Canonical-valid/full-check-negative native subject;no caller IR|native declaration/binding identity complete without fake IR/node/Checked pin|Not Executed|
|G1-NG-018|Source,ordinal,origin,context: No root object|Same native input with no Root ComputationId|no root requirement in native subject;root observation absent|Not Executed|
|G1-NG-019|Source,ordinal,origin,context: Forward ordinal change|Different actual selected Forward E occurrence|coordinate and source change;not an array count or generic0|Not Executed|
|G1-NG-020|Source,ordinal,origin,context: Action ordinal change|Different actual selected action E entry|exact action-entry ordinal changes;binding ordinal remains a different field|Not Executed|
|G1-NG-021|Source,ordinal,origin,context: Binding ordinal change|Same guard definition used by distinct action bindings|distinct declared binding application origins,full identities retained|Not Executed|
|G1-NG-022|Source,ordinal,origin,context: Action identity change|Change qualifier/resource selector in a full atom|full AtomKey lookup changes;operation-only equality cannot merge|Not Executed|
|G1-NG-023|Source,ordinal,origin,context: Guard role change|Same shared E guard evaluated as Forward versus Action|parameterized guard_inputs instances retain distinct real reference paths/roles|Not Executed|
|G1-NG-024|Source,ordinal,origin,context: Same content reevaluation|Repeat identical canonical declaration/context/application|one formal native origin,additional observations only;no call counter identity|Not Executed|
|G1-NG-025|Source,ordinal,origin,context: Different declaration same guard|Different complete canonical Contract body or binding occurrence,same guard text|different native source/origin;lineage alone insufficient|Not Executed|
|G1-NG-026|Source,ordinal,origin,context: Context alias mismatch|Canonical validation object bound to context A,analysis uses distinct B with equal bytes|capture admission fails or performs explicit fresh validation under B;no digest→alias proof|Not Executed|
|G1-NG-027|Source,ordinal,origin,context: Open known row|Real Forward/action references in Open.known|preserve row-kind tag and known-vector ordinals;do not call closed an object field|Not Executed|
|G1-NG-028|Source,ordinal,origin,context: Raw reorder/canonical source|Reorder valid raw action bindings;canonicalizer sorts full keys|origin uses explicitly admitted canonical vector;raw provenance not falsely claimed as the canonical position|Not Executed|
|G1-NG-029|Witness,owner,descriptor: All14 field differences|Independently change each row of the14-field projection contract|full typed comparison detects every required native field difference;not exists/count/hash comparison|Not Executed|
|G1-NG-030|Witness,owner,descriptor: Assignment schema pin|Same port/value but altered exact TypeSchemaRef digest|invalid typed witness/source pin;equal type name is not enough|Not Executed|
|G1-NG-031|Witness,owner,descriptor: Absent payload|V/TypedCountermodel with assignments or source_contract missing|IncompleteFacts/no complete fragment;do not repair from reason|Not Executed|
|G1-NG-032|Witness,owner,descriptor: Unknown witness tag|Undeclared NativeGuardWitness case or private error kind|UnsupportedModel;no Other(String) acceptance|Not Executed|
|G1-NG-033|Witness,owner,descriptor: Historical Owner Scope|Claim schema ModelType/ModelRef,owner model@1,name Scope|record current UnsupportedModel as safe staging availability limit,not nonexistent historical type|Not Executed|
|G1-NG-034|Witness,owner,descriptor: Current Owner Scope|Current owner/name Scope valid metadata claim|metadata-only acceptance;not complete Scope or capability|Not Executed|
|G1-NG-035|Witness,owner,descriptor: Declared but inactive|Native guard type exists in inherited snapshot but not active types|separate Declared/Registered/Included/Reachable/Materialized/Claimable states;no inferred import|Not Executed|
|G1-NG-036|Witness,owner,descriptor: Same name different owner|Native type spelling under current,historical,AFIDL and unknown owners|exact provenance/admission differs;unknown never broadened|Not Executed|
|G1-NG-037|Witness,owner,descriptor: Native enum ordering|Historical JSON variant keys alphabetically ordered|preserve actual Frozen/Zero/Typed/Insufficient native declaration ranks,not JSON key rank|Not Executed|
|G1-NG-038|Witness,owner,descriptor: Effective versus declared bound|Track original shared E ceiling and recovered local seed|retain both typed roles;do not invent a same-atom14-field countermodel that current containment short-circuits|Not Executed|
|G1-NG-039|Witness,owner,descriptor: Structural V without assignment|MissingPort/TypeMismatch/arithmetic V,legacy countermodel=None|preserve None and structural cause;do not fabricate GuardInt64 assignment|Not Executed|
|G1-NG-040|Witness,owner,descriptor: Derived assignment DAG|Generated value1 absent as a literal in source Guard0|witness payload owns its own typed DAG;do not insert a report/witness descendant into input Scope|Not Executed|
|G1-NG-041|Root,edition,capability: Native observed by root|Actual checked source/body/context join then RootRecovery consumes native P|original native stage/origin retained;separate root call-site fact and observation|Not Executed|
|G1-NG-042|Root,edition,capability: Two root observations|Two real call-sites consume same native declaration/binding|one native origin,two consumer/observation edges;not two native facts|Not Executed|
|G1-NG-043|Root,edition,capability: Two source occurrences|Different actual computation nodes,one native origin|root source occurrences remain distinct;root aggregate not substituted for native|Not Executed|
|G1-NG-044|Root,edition,capability: Negative rejected before root|Native typed countermodel V makes check_contract reject|retain native diagnostic and real earlier rejection;do not manufacture a checked root source or Root V|Not Executed|
|G1-NG-045|Root,edition,capability: Root multiplicity|Root Retry/Iterate factor added to an already local native action upper|apply root factor exactly once on root bounds;never multiply native guard origin/count|Not Executed|
|G1-NG-046|Root,edition,capability: Old label/new content|Current/legacy analysis label combined with proposed changed model/registry bytes|InvalidIdentity under exact future routing;not relabelled old success|Not Executed|
|G1-NG-047|Root,edition,capability: New label/old content|Future exact tuple label combined with old model/import bytes|InvalidIdentity or actual missing-content category at defined layer;no same-name repair|Not Executed|
|G1-NG-048|Root,edition,capability: Capability denial|Native local P or successful selected-type claim with no complete T3/T4/T5/T6/T7/T8|no RootExport Checked,CompleteView,runtime/publisher authority or automatic R3/T4 admission|Not Executed|

The14-field difference law requires14 independently authored field perturbations in a later comparator implementation;it is ONE design law here,not14 executed cases. Future real-input fixtures must pass actual earlier admission before claiming a native branch;otherwise record the genuine rejection and no capture coverage. G1's source diagnostics support the audit but do not change any row's Not Executed status. Machine-readable companion:independent-vectors.json.
