# 48 independent design vectors — all Not Executed

四组各12例。完整保留输入定义、真实来源包模板、Native十三字段序列、设计Physical
原始字节、每例条件及独立手算期望，见同前缀independent-vectors.json。
JSON中的fixture引用只是一种可无损展开的文档记法，不是新ModelRef、DAG rank、
opaque payload或可直接解码的正式wire数据。源X来自冻结C2完整输入；P是本次设计
预先给定的完整字节字面量，不是假称从C2 Native对象恢复的历史字节。

每例是在fixture基础上应用given中的明确条件。例如missing-package例确实删除
来源包，而不是仅因fixture名称存在就准入。第36例在实际新声明包中额外提交三个
不同真实Owner（zero-owner/positive-owner/duplicate-owner），分别声明X/Y/duplicate；
不是复用相同Owner/key暗改内容。Port判断只指实际到达的局部正数/唯一性分支。

所有Expected均为草案的手算条件结论，**Actual Status: Not Executed**。
当前只读C2诊断另行执行，不会把任何下列新向量改标Executed，更不计入Producer
Conformance。所有草案失败名须未来单独注册；无法构造完整typed输入的Portable
缺字段可先报IncompleteFacts，语义分类为MissingDocumentOccurrenceBinding。
第37例的content_subtrees=1仅指相等的完整RawDocument内容节点，不是总DAG节点数。

| CaseId | 独立条件 | 手算预期摘要 | Actual Status |
|---|---|---|---|
+| c2g1-design/01 | 独立同内容文档 | DistinctOccurrencesAndSources | Not Executed |
| c2g1-design/02 | 同源双观察 | SameFact | Not Executed |
| c2g1-design/03 | 不同真实Owner | DistinctOccurrencesAndSources | Not Executed |
| c2g1-design/04 | 复制/跨进程重建原来源 | SameOccurrenceAndFact | Not Executed |
| c2g1-design/05 | 真实Import条目 | DistinctOccurrencesAndSources | Not Executed |
| c2g1-design/06 | 同事实多个消费者 | SameFact | Not Executed |
| c2g1-design/07 | 重复相同观察 | SameFactAndObservationSet | Not Executed |
| c2g1-design/08 | 观察顺序交换 | SameFactAndCanonicalObservationSet | Not Executed |
| c2g1-design/09 | 同文档不同内部路径 | SameOccurrenceDistinctMembersAndRawSources | Not Executed |
| c2g1-design/10 | 真实Workflow来源集合 | DistinctOccurrencesAndSources | Not Executed |
| c2g1-design/11 | 不同实际声明域 | DistinctDeclaredOccurrences | Not Executed |
| c2g1-design/12 | 只有两个相等对象 | MissingDocumentOccurrenceBinding | Not Executed |
| c2g1-design/13 | 缺namespace | MissingDocumentOccurrenceBinding | Not Executed |
| c2g1-design/14 | 缺domain key | MissingDocumentOccurrenceBinding | Not Executed |
| c2g1-design/15 | 缺Owner key | MissingDocumentOccurrenceBinding | Not Executed |
| c2g1-design/16 | 缺Owner revision | MissingDocumentOccurrenceBinding | Not Executed |
| c2g1-design/17 | 缺occurrence key | MissingDocumentOccurrenceBinding | Not Executed |
| c2g1-design/18 | Owner域不匹配 | InvalidOccurrenceAssociation | Not Executed |
| c2g1-design/19 | 未知Owner/持久handle | UnsupportedOwnerVariant | Not Executed |
| c2g1-design/20 | Owner未声明 | InvalidOccurrenceAssociation | Not Executed |
| c2g1-design/21 | 容器条目越界 | InvalidOccurrenceAssociation | Not Executed |
| c2g1-design/22 | 相同Owner不同完整容器 | OwnerBindingConflict | Not Executed |
| c2g1-design/23 | 相同Owner/key不同内容 | BindingConflict | Not Executed |
| c2g1-design/24 | 重复Owner声明行 | MalformedSourcePackage | Not Executed |
| c2g1-design/25 | 相同原字节独立来源 | DistinctOccurrencesAndSources | Not Executed |
| c2g1-design/26 | 同physical重复观察 | SameFact | Not Executed |
| c2g1-design/27 | Native没有物理历史 | DeclaredOnly | Not Executed |
| c2g1-design/28 | Native双来源未注册 | UnsupportedPhysicalAssociation | Not Executed |
| c2g1-design/29 | Physical缺supplied关联 | UnsupportedPhysicalAssociation | Not Executed |
| c2g1-design/30 | Physical跨Occurrence混接 | InvalidOccurrenceAssociation | Not Executed |
| c2g1-design/31 | 同slot改变原字节 | BindingConflict | Not Executed |
| c2g1-design/32 | 共用真实Context的两文档 | DistinctSourcesWithSameContext | Not Executed |
| c2g1-design/33 | 同值不同Context对象 | InvalidContextBinding | Not Executed |
| c2g1-design/34 | Context不能替代Owner | MissingDocumentOccurrenceBinding | Not Executed |
| c2g1-design/35 | 同值但真实选择错Owner | InvalidOccurrenceAssociation | Not Executed |
| c2g1-design/36 | Owner不改变Port判断 | Violation,Proven,Proven,Violation | Not Executed |
| c2g1-design/37 | 相等内容仍共享 | 1 | Not Executed |
| c2g1-design/38 | 重复相等DAG节点 | NonCanonicalValueDag | Not Executed |
| c2g1-design/39 | 任意换rank | NonCanonicalValueDag | Not Executed |
| c2g1-design/40 | 无关根/不可达节点 | InvalidScopeOrNonCanonicalValueDag | Not Executed |
| c2g1-design/41 | 输入引用结果成环 | UnsupportedModel | Not Executed |
| c2g1-design/42 | 精确旧版content-only | ExactOldEditionContentIdentity | Not Executed |
| c2g1-design/43 | 旧标签贴新binding | UnsupportedModel | Not Executed |
| c2g1-design/44 | 未来新版缺O | MissingDocumentOccurrenceBinding | Not Executed |
| c2g1-design/45 | 未来新版完整绑定 | DeclaredOnly | Not Executed |
| c2g1-design/46 | 历史来源缺失不能补造 | UnmigratableMissingOccurrence | Not Executed |
| c2g1-design/47 | 来源不授Root权限 | NotAuthorized | Not Executed |
| c2g1-design/48 | 随机盐不能补来源 | MissingDocumentOccurrenceBinding | Not Executed |

覆盖必需的同内容独立文档、同源重复观察、两Manifest条目、两Owner、Owner/key
复用与冲突、缺来源、成员路径、完整内容变化、两种Source、Context对象身份、
DAG共享/冗余/rank/环、旧版解释/新binding/历史缺失及拒绝Root授权。
静态检查只核验文档CaseId/数量/引用/Not Executed等完整性，不执行这些身份、
Admission、DAG、Port或权限语义期望；不实现未来完整Python Oracle。
