# Actual SourcePackage → Producer

入口为 `af_computation::raw_capture::capture_raw_ports`，现有 native_capture
模块也显式转出 collect_raw_ports。该入口接受 DocumentOccurrenceMetadataRequest，
实际 analysis/validation Context 和完整现行 CompilerSemanticsIdentity。
没有只接收 RawCandidate 然后补造来源的重载。

1. 原 C2G2 admit_document_occurrence_metadata 验证现行 Compiler、实际 Context
   对象相同、完整包、域/Owner 唯一性、所有真实条目及所选文档借用。
2. OwnerAnchor 完整保留 namespace/domain_key/owner_key/revision_key 的精确 UTF8
   字节；Slot 是 declarations/imports/sources 中的 U64 位置，不是 Port ordinal。
3. 本地完整文档借用必须就是包中选中的条目；外部同值克隆不够。地址仅用于
   本地关联，绝不输出为来源身份。绑定身份仍为完整 Anchor 与真实条目位置。
4. 原始内部路径选择候选值；Physical 严格 UTF8/JSON/重复键/封闭字段/宽度检查
   在 Port Predicate 前完成。Native 保留完整声明序列。转换所有 Native 文档
   为实际 ComponentContract，包括未选中的原始条目，不截断成单条文档。
5. 根据已冻结 Raw 描述符精确解释字节、序列、Option、Variant；只为实际两个
   Port Rust 字段执行明确的 type_ref/type 作者格式映射。任何未知形状失败关闭。
   String 逐字节 UTF8 解码，不做 NFC 修复、排序或 Canonical Contract 编码。
6. 同一借用 Context 传入 observe_raw_ports，Port 事件不能由元数据准入成功代替。

## 四种身份与冲突

Content 是完整 RawDocument。Occurrence 是完整 Anchor 与真实外部 Slot。
内部候选 Entry、Port original/sorted ordinal、报告观察位置分别保留。
same_subject 比较实际完整包、Owner/Slot、文档、候选、内部路径、Compiler 和
九成员 Context 全值；不以摘要或 Context 代替文档身份。不同内容的显式联合
Owner 记录拒绝 OwnerBindingConflict；绑定文档不同拒绝 BindingConflict。
只审查实际提交的有限包，没有全局账本。

重复 Capture 的完整输入相同，不发明新 Owner、修订、Slot 或调用序号。
Collector 合并相同局部键的合法观察，非观察 Payload 冲突拒绝，绝不择优覆盖。
同 O 但完整包/Compiler/Context 不同，不强行合并整个来源 Subject。

## 支持矩阵

| 实际文档 | Physical 参数 | 行为 |
|---|---|---|
| physical_json_utf8 | supplied 且 Owner/Slot/文档借用/路径完全匹配 | 严格解析并保留原字节 |
| declared_native_document | not_supplied | 完整声明转换，不补造物理历史 |
| Native | supplied | UnsupportedPhysicalAssociation |
| Physical | not_supplied | UnsupportedPhysicalAssociation |

没有来源的旧 Native API 仍返回 MissingDocumentOccurrenceBinding，普通 Contract
Diagnostic 保持原意。Namespace 是声明值，不是认证白名单；已加入合法不同域名
声明的正例，以及真正 Owner/包域不一致的负例。

## Context 与输入图

保留 modules/schema_universe/effects/resources/flow/usage_meters/recovery/
execution/relations 九个实际输入，角色分区 [[analysis,canonical_validation]]。
同值不同对象不能满足本次角色别名约束。输入图没有 Raw Fact、Witness、
Observation 或 Capture 回指：RawSubjectRef 只暴露已提供输入，输出引用输入，
不反向给输入添加生成结果。完整跨进程 Scope/最小 DAG 编码仍 Pending G02。

尚未提供这些九成员值与 CompleteCompiler 的独立全字段 Python 投影，因此
本地对象关联正确不能替代完整 Subject 的 Full-field Conformance。
