# 完整现有绑定穷举

机器入口：`tools/oracle/native_relation_source_design.py`，`census()`。
两份同内容清单 existing-binding-census.json 和 r5g1-native-relation-source-binding-census.json
保留用户指定名称与统一阶段前缀。不是只搜索两个相似类型名。

先按冻结 v7 closed delta + 精确 SHA 固定的 v6 base 合成全部1131个 Descriptor；
遍历全部字段、Variant、引用、Alias、317个结构站点、66个 carrier-use 策略。
登记每个类型的确切 owner/name、全字段、ModelType rank、引用种类、body 委托、
G/W eligibility、来源/阶段/Slot、规则依据、所有直接入边和缺失绑定判断。
完整 Descriptor 引用计数2559；1297 Field/Variant 行仍是旧结构注册清点，不是本轮运行分母。

第二次遍历模拟**类型产生闭包**：从唯一 `RawValidationSubject` 根出发，记录每条实际
可生产节点的注册 child/site 目标；当声明是 alias 时只委托 body/site，保留原节点 key，
不将被委托的声明自动当第二个子节点。此遍历区别于普通 Descriptor 传递闭包；
后者确实能沿 alias 找到 NativeSource，但不能据此证明 G 中存在该名义节点。

`NativeRelationSnapshotSource` 六字段 record 已注册；`RelationSnapshotContent` 仅为
body ref alias。两者单独 value_binding，均有 g_input eligibility。eligibility 是许可的值类型，
不是无条件可加入唯一根图的额外来源站点。所有317个 inline site 的 value_type 都不是
NativeSource；其字段内部站点只能在已有 Source 或 Alias body 中生产字段子值。

机器清单为全部直接指向 NativeSource 的引用逐项记录排除原因：

- RelationSnapshotContent 的 ref 是 body 委托，不是产生 Source 子节点的边。
- C.source 是 G-external 消费引用，本身属于 derived C，不能作为 G 生产者。
- RelationCurrentValidationMetadata.source、RelationSourceDagMetadata.nodes 等独立元数据
  子图不在 Raw Subject 根闭包中，既不能补根，也不能当 Source admission 能力。
- 生成的 C inline site 仍处于输出/元数据环境，不产生 G 输入根边。

所有其它1131个声明均已在清单中保留；不在闭包的类型不可能通过一个未枚举的合法
child 被纳入，因为每个可达节点的所有 child 都已由封闭 Descriptor/site 表列举。
以根可达路径长度归纳：根不是 Source；每条枚举边都不以 Source 为名义目标，故无
合法可达 Source 节点。附加节点会不可达；附加根违反既定 root roster。

名称校正：没有独立 `RawValidationSource` Descriptor，真实来源是 `Source.raw_validation`；
`DagRank` 是 Node<T> 的 U64 语义，不是同名 ModelType；`VerifiedInputGraph` 与
`RawSubjectInputNode` 是私有构造的 Rust issuer/wrapper，不是可反序列化注册节点。
`CanonicalValueDag`、`DagNode`、`NodeBody`、`ModelType` 的确切字段见清单。

源码证据：input_graph.rs 的 descriptor() 保留 key、child_key() 返回确切 ref 目标，
build() 按原 key 实习完整 typed interning；finish() 独立校验可达和规范 ranks。
input_binder.rs 的私有图/根 wrapper、实际 source/context 借用不提供额外构造入口。
Python 独立 decoder 同样检验每条 child 的确切名义 key，不接受同 shape 转换。
结论：现有合法绑定穷举不存在；不是单纯“没有搜索到”，也不指控现有 Binder 违约。
