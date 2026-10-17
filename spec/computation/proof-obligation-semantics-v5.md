# agentflow.computation.proof-obligation.semantics@5

Registered staging only. Current exact model@5/report@5/obligation@5/analysis@6
adds document occurrence inputs; it does not implement Raw capture or G02/T4.
RawInputOrigin has document_binding:Node<DocumentOccurrenceBinding> and internal path.
RawPhysicalAssociation.supplied has the same binding reference and value_path.
RawDocument content variants are unchanged. RawParsedInput,RawValidationSubject,
Source.raw_validation,RawValidationOrigin,Origin,FactKey,Scope and all basis,witness,
evidence/prerequisite/consumer references follow that exact new typed closure.
Mixed Source/Origin/Physical/Witness occurrences reject even when content is equal.

Same full Scope/compiler/context/predicate/source/member/non-observation payload
permits union only of legitimate observations. Conflicting same-key semantic values
are InvalidFacts,not last-write or verdict-max repair. Same occurrence in different
Scopes,compilers or contexts does not force full fact identity equality.

AF-SEMANTIC-0217 is unchanged: finite reachable typed DAG,equal content interned,
canonical bottom-up ranks,no redundant equal nodes,alternate numbering,salts or
unrelated roots. Complete source packages contain inputs only; domain anchors are
scalar input records,not package backreferences. Package->owners->containers->documents
never points to a binding,Scope,Fact,Witness,Obligation or Report. Output DAGs remain
separate,so there is no generated-output provenance or report/Scope fixed point.
No fragment becomes Root CompleteView. Full Scope and canonical identities remain
NotImplemented; the registered constraints and metadata checks do not implement T4.

<a id="identity-propagation"></a>

## AF-SEMANTIC-0280 — DO-09

MUST: Occurrence binding starts at RawInputOrigin and propagates through subject, source, raw origin, FactKey, basis, witness and references without mixed occurrences.

Direct normative dependencies: AF-SEMANTIC-0200, AF-SEMANTIC-0267.
Registration conformance: g01-document-occurrence-staging/rule-0280; NOT Producer Capture.

<a id="input-dag"></a>

## AF-SEMANTIC-0281 — DO-10

MUST: Equal typed content still interns under the finite minimal reachable DAG; different real occurrence bindings are different typed values and source packages cannot refer to generated outputs.

Direct normative dependencies: AF-SEMANTIC-0217, AF-SEMANTIC-0269.
Registration conformance: g01-document-occurrence-staging/rule-0281; NOT Producer Capture.

<a id="observation-copy"></a>

## AF-SEMANTIC-0282 — DO-11

MUST: Only equal complete scoped originating facts with equal non-observation payload union legitimate observations; conflicting payloads fail InvalidFacts and observations never generate occurrences.

Direct normative dependencies: AF-SEMANTIC-0210, AF-SEMANTIC-0211.
Registration conformance: g01-document-occurrence-staging/rule-0282; NOT Producer Capture.
