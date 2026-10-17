# 身份生成路径及最小变更位置

最早增加维度的位置是新 RawInputOrigin 的文档关联，不是最终 FactKey：

```text
DraftDocumentOccurrenceBinding
 -> DraftRawInputOrigin {document_binding, path}
 -> RawParsedInput.origin
 -> RawValidationSubject.input
 -> Source.raw_validation.subject

DraftRawInputOrigin
 -> RawValidationOrigin.input_origin + direction + original_ordinal
 -> Origin.raw_validation_origin
 -> FactKey.source / FactKey.origin
```

RawDocument 的内容 Variant 保持不变。RawParsedInput 的 candidate/physical、
RawValidationSubject 的 Compiler/Context 都保留；只是引用的新 Origin 发生传递
类型变化。Source.raw_validation 的现有字段形式可保留，但 subject 的完整类型
身份及编码内容已变。RawValidationOrigin/Origin/FactKey/Scope 的完整闭包身份也
随之变化，不能因字段名没变便沿用旧版本的完整语义内容身份。

RawPhysicalAssociation.supplied 的 document 字段在草案中改为同一 binding 引用；
not_supplied 不变。所有 raw member EvidenceRef、RawPortCheckBasis.subject、
contradiction/rejection witness 中的 subject/input-origin、RawValidationFragment、
supporting/prerequisite/consumer 的原始 source 引用都应追踪同一新闭包。
不能漏改一条路径，造成 Source 来自 O1 而 Origin/Witness/Physical 指向 O2。

内部两路径继续保留：RawDocumentMember 从已绑定文档的真实 entry 到端口，
DeclaredStep 从 subject.input.candidate 到相同原始端口。新 Owner/容器slot 路径
是更外层，不塞进现有 candidate 的内部成员路径，不把 ordinal强制cast成DeclaredStep。

不同 O => Source 与 Origin 的真实输入不同；不同文档内部成员 => member身份不同。
同 O/member/Predicate 等完整key、同所有非观察payload，只 union合法观察集合；
若相同key出现状态/条件/证据/支持/witness冲突则 InvalidFacts，不择优合并。
保持 AF-SEMANTIC-0200/0210/0211 语义；ObservationOwner 是报告族标签，不是 SourceOwner。

Raw Port 原判定冻结：一个 positive_unique_io_ports Case，Input在Output之前、
稳定U32排序、保留实际原ordinal、正数且不重复，初始0哨兵非真实前驱。
Fresh Positive=P、Zero=V、Duplicate=V；早期停止和未访问事件不伪造。
新来源维度不修改这些 Verdict，也不把局部 Proven 升级为 schema/whole Checked。

Context 的九成员、analysis/canonical_validation 同一对象分区完全独立；
禁止借 ContextRole、Context Digest、resolver、报告位置编码文档 Owner。
