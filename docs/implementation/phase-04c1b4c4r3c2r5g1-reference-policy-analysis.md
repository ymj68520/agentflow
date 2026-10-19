# 全部66项引用策略

existing-binding-census.json.reference_policies 保留每项完整 parent/path/environment/
domain/target/profile 与评审结论；required roster 从冻结 Descriptor 与 carrier use 独立
重建，不从 Rust Actual 或待比较 view 构造。别名 inline site 不重复制造独立引用权限。

C.source 的两条确切规则：

| parent/path | environment | domain | exact target |
| --- | --- | --- | --- |
| CurrentContextRelationSchemaValidation/source | output_record | g_external | NativeRelationSnapshotSource |
| CurrentContextRelationSchemaValidation/source | w_value | g_external | NativeRelationSnapshotSource |

这些都是消费规则，不产生一个输入 Node。output_record 不表示 output-local。
所有 g_value 的现行 dag_ref 只能按注册 g_local 引用该 G 中已存在 exact target。
其它 output_record/w_value 的 G 引用也不赋予额外 Root、Source 重标或 alias cast 权限。

未来方案保留66项的既有方向/权限与确切目标含义；仅按新 Model owner/closure 重新登记，
不能把旧 owner 引用带入新载体。三条改绑 Some 是普通 named-value child 边，不是新增
C.source policy，也不扩充 W-local 权限。C.context 仍引用其实际 G 中完整 Context contents。

消费必须在 fixed G 中解析正确 Source type，并核对它确实来自同一 Subject 的该 Slot，
同一个实际 Context association；“载体里碰巧存在一个同类型值”不够。跨 Slot、跨 G、
跨 Edition、错误 Owner/O 的混合、同 body 的 W-local Source 全拒绝。
W 输出若使用 Source，external target 不参与 W local height；W 不能决定 G identity。
