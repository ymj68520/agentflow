# Registration manifest / complete provenance chain

规范源分别是component/document-occurrence.md、computation/proof-obligation-
semantics-v5.md、tables/proof-report-model-v5.md、portable/portable-proof-inputs-v6.md，
均位于spec/。14条DO与真实规则一一映射于本次实际分配0272–0285；设计条款数
并未被当作必须等于规则数的先验，分配前先做实际冲突检查，收据另附。

spec/conformance/g01-document-occurrence-staging/registration-map.json完整记录：
DO → Source/SpecificationId/Anchor → RuleId → exact Model owner/Descriptor names
→ executed registration CaseIds → CompilerSemanticsIdentity。rule-registration.json
保存完整声明，Rust逐条检查真实Registry及规范Anchor，并输出真实执行回执。

## 完整新类型（均属Model@5，旧内容不改）

DocumentSourceKey、DocumentSourceDomain、DocumentOwnerAnchor、
DeclarationDocumentEntry、ImportDocumentEntry、WorkflowDocumentEntry、
DocumentOwnerContainer、DocumentOwnerRecord、SourcePackage、DocumentOccurrenceKey、
DocumentOccurrenceBinding、DocumentSourceAssurance、DocumentOccurrenceAdmissionFailure、
DocumentSourceMigrationState，共14个正式关系/状态类型。

另7个明确的有限inline metadata companions：MetadataDeclarationDocumentEntry、
MetadataImportDocumentEntry、MetadataWorkflowDocumentEntry、MetadataDocumentOwnerContainer、
MetadataDocumentOwnerRecord、SourcePackageMetadata、DocumentOccurrenceMetadataBinding。
它们把正式Node<RawDocument>/Node<SourcePackage>替换为完整inline目标值，字段/
Variant语义不变，不把数组下标冒充DAG rank，不是reflection或OpaquePayload。
这使本阶段能够验证完整元数据关联，而不伪称已实现G02 lossless Scope编码。

RawInputOrigin在新模型用document_binding:Node<DocumentOccurrenceBinding>替代
旧document字段；RawPhysicalAssociation.supplied使用同一binding。旧RawValidation-
AdmissionFailure在新模型追加MissingDocumentOccurrenceBinding；旧模型仍精确冻结。
其余旧680个Descriptor原值相等，Source/Subject/Origin/FactKey等引用自然进入新闭包。

descriptor-manifest.json保存全部2064引用边、704 ranks、24个新建/替换完整
Descriptor及精确Owner；权威完整表为新model@5。原model@4以完整对象嵌入，
原model@1/@2/@3/@4源文件及97 selected imports均逐内容冻结，禁止latest同名回退。

Current-only实现绑定全部62注册源、36选定完整源、188实际applied规则以及四个
完整closed tables；不是count/hash-only比较。Source/Rule内容及Registry原字节
都进入CompilerSemanticsIdentity。Reference coverage不等于完整依赖闭包，F03仍OPEN/P2。
