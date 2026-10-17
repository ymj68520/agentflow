# Existing Exact Binding Audit — COMPLETE / NOT FOUND

本次审查不是依字段名猜含义。机器附件 descriptor-census.json 保存：全部683类型的
现行rank、2018条完整位置/目标引用边、全部引用闭包解析、66个候选完整Descriptor、
每个候选的精确 ModelType owner/name、正式table源/字段/适用族/admission/equality/
实际证据/缺项。其选择含全部13个题目类型、52个Owner/Source/Occurrence/Origin/
Document/Scope/Observation/Operand等名称候选以及额外闭包类型。题目所列
Source.raw_validation 是Source的已冻结分支，而非额外未计数Type。

精确注册Owner：未显式历史限定的候选均为
agentflow.computation.proof-report.model@4/<table-key>。嵌入的
agentflow.computation.proof-report.model@1/<name> 保持该历史Owner，不能改挂最新Owner。
表源：spec/tables/proof-report-model-v4.md，注册digest
52150b387c48b8c6e0800e80cf0c82607cd295d91fd53de5f96c499a7c655a8b。
current schema admission在 registered_model.rs 检查精确owner/name，拒绝旧owner/newtype
及unknown。历史导入完整源/递归声明由冻结的原审查工具独立核验，不只比较名字。

## 题目核心类型的完整身份维度

| 候选 | 精确字段或分支 | 当前意义 / 缺少什么 |
|---|---|---|
| RawDocument | physical_json_utf8{bytes}; declared_native_document{declarations} | 完整内容，无外部Owner或occurrence |
| RawInputOrigin | document:Node<RawDocument>, path:Seq<RawDocumentMember> | 仅文档值及内部位置，第一处缺新来源维度 |
| RawParsedInput | candidate, origin, physical | 引用上述同值origin，没有另外的Owner |
| RawPhysicalAssociation | not_supplied; supplied{document,value_path} | 物理关联仍是文档值，不含独立外部来源 |
| RawValidationSubject | owner_compiler, input, context | owner_compiler是Compiler不是文档Owner |
| RawValidationOrigin | input_origin,direction,original_ordinal | 声明内部坐标，无法增加文档外部来源 |
| RawContextBinding | contents,roles | 九成员和Context角色分区，不是文档角色 |
| Source.raw_validation | subject:Node<RawValidationSubject> | 唯一Raw族合法Source，完整subject仍无O |
| ScopeSubject | root_export/contract_validation/computation/standalone_substitution/native_recovery_guard/raw_contract_validation | Raw分支仅RawValidationSubject，不允许假造其它族输入 |
| Scope | edition,analysis_semantics,content,subject | 完整输入闭包；没有任意nonce扩展点 |
| Origin | predicate,declared_path,source_ordinal,mapping_ordinal,raw_validation_origin | 实际声明角色/序列路径，非Owner分配接口 |
| FactKey | stage,assertion_path,kind,dimension,source,target,coordinate,rule,requirement,origin | Observation不在此；不能附加临时ID |
| CanonicalValueDag | nodes,roots | 相等typed值去重，rank不是来源身份 |

Raw准入规范：spec/component/raw-contract-validation.md 的 AF-SEMANTIC-0261–0268，
G01对象/来源/复制规则0197/0198/0199/0200/0210/0211，闭合Descriptor规则0270。
实际实现 raw_admission.rs 只核验Compiler、同一Context、文档/条目完整值或物理
结构与路径，返回unit，不产生Fact。不存在验证独立来源Owner的参数或路径。
66项的Equality均是完整nominal typed equality；AF-SEMANTIC-0217的去重规则不改。

## 其它看似可用的候选及精确拒绝理由

| 候选/域 | 已注册来源与适用条件 | 不适用于本反例的原因 |
|---|---|---|
| ApplicationDocument{descriptor,owner,applications}; ApplicationOrigin{document,declaration_path} | standalone-substitution-proof.md，AF-SEMANTIC-0230/0238/0239，27-case族真实Checked operands及实际applications[index] | owner为SpecificationId；没有独立同内容外部文档occurrence；更不能替换Raw族Source |
| OperandSubject{role,contents,identity,context_role} | 同上；ComponentIdentity为catalog/name | 是Checked operand角色/内容，不是文档提交所有者 |
| NativeRecoveryGuardSubject / NativeGuardOccurrence | native-recovery-guard.md，0245/0247/0248；7-case canonical声明及真实action/effect/usage内部ordinal | 完整canonical声明的内部出现位置，不追溯Raw来源，Port0无法取得canonical输入 |
| Source.root/component/subflow/scope/control | 0198/0200及原Root/IR实际关联，完整Caller/Checked IR和节点/导出/entry ordinal | C2直接Raw输入没有这些真实对象；不能生成虚构Root/IR/Resolver |
| Source.contract_operand/substitution_application/native_guard_* | 各封闭族的完整subject/真实application/member | Raw case descriptor不允许其它Source Variant替代 |
| Source.context/compiler | 实际目录角色/member路径 | 目录或Compiler来源，不是文档来源；C2共用同一个C/K |
| RootProvenance/ResolverAssociations | 完整IR/source/target/context及实际resolver绑定 | 不是通用外部来源manifest；反例中无这些输入 |
| OwnerVersion/ModelType/ModelRef | 版本/NFC文本或精确规范owner/name；0242/0256/0270及注册lookup | “规范拥有者”不等于“具体文档Owner”；任意解释会改变已注册语义 |
| DeclaredStep/$Ordinal64/BranchOrdinal/HandlerOrdinal | enclosing实际声明field/entry/member，数学U64序列顺序 | 只能选择已有结构，不可补造来源容器或将push顺序重命名为真实路径 |
| ObservationOwner/ObservationSite | 0200/0210；报告族与field/exposing路径、consumer | 是结果暴露位置，不能反向决定输入文档Owner |
| Effect occurrence bounds/ScopeBinding/FlowSourceRef/RootEffectOccurrenceIdentity/RecoveryActionSourceIdentity及Raw/历史对应项 | 原Effect/Flow/IR/Recovery闭合Descriptor和该域真实声明 | bounds是数量，scope binding是资源属性，source coordinate是该图内部来源；没有Raw文档来源域，也没有Raw引用入口 |

所有同名Raw companion/历史owner副本只保留原字段/宽度/来源域，不获得新文档语义。
即使找到一个可存不同字符串的标量，Raw身份生成路径也没有合法的新增承载位置。
Source 其它Variant不能因为“看起来像Owner”就被Raw case接受。

## 当前精确反例

旧C2报告、formal-binding-gap.json、final-runtime-results.json已读取；旧运行帐本
是2026-10-16阶段证据，不充当本次新鲜门禁。本次又运行原只读诊断：两份独立
存储的两条目文档，完整typed值相等，选entry0/inputPort0；共用Compiler及真实
同Context。均source-admit Ok(unit)，canonicalize均AF-CONTRACT-0004/
invalid_or_duplicate_port，Checked=false。Different Rust Objects=true；
Equal Complete Typed Values=true；Equal Registered Identity Inputs=true；
Separate Formal Provenance=NOT CURRENTLY REPRESENTABLE。

结论：Option A 不成立，不裁决 Existing Binding Verified；现行缺口仍须正式修订。
