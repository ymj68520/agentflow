# 唯一根与最小可达 DAG

本方案的最小改动是**替换未来正式 child target**，不是并列追加一个不可达 Source。
三条确切声明边为：

```text
ContractProofContextContents
  .relation_snapshots.effect_resource.some : ref NativeRelationSnapshotSource
  .relation_snapshots.recovery_usage.some  : ref NativeRelationSnapshotSource
  .relation_snapshots.flow_execution.some  : ref NativeRelationSnapshotSource
```

这里只是 successor 草案。当前三条仍为 ref RelationSnapshotContent，不可执行新语义。
注册时须同步更新 parent inline relation_snapshots record、三个 option site Descriptor/
child binding 和相关 source binding/full closure，不能只改一个表面字段。
本清单 proposed_node_production_closure 在**副本的闭包演算**中恰好改变三条 Source 边；
没有提交改动到 Catalogue。它是类型可达性证明，不是将未来 Graph 作为模型@7 Actual。

Source 到根的路径为 RawValidationSubject.context:RawContextBinding，接 contents:
ContractProofContextContents，再经上述 relation_snapshots record/slot option 到 Source。
Option None 是叶，不产生 Source。Option Some 的六字段真实数据已存在，record/variant/
option/sequence/set/map 等 Body 全由原八个 NodeBody case 表达，无需新 Variant。
根仍恰好一个 RawValidationSubject；Source 自身及字段都是其后裔。没有额外 root。

最小性归纳：真实叶按完整类型化相等 intern；每个合法父保留所有按 Descriptor 确定的
边；每个新节点都是根展开的后裔。同型同值重复只保存一个节点，不同名义不合并。
这同时允许两个 originating Subject/Owner 合法共享同一内容子值而不删除父关联。
最后独立验证所有节点可达、全部字段齐全、child rank 更早、无重复相等节点、根确切，
符合 AF-SEMANTIC-0217/0305。Source 等于 Content 的 shape 不意味着两个类型都要建；
新 Some 只生成正式要求的 Source，旧 Content 不以“保留它”为由成为不可达冗余节点。

禁止：补根、无父 link、任意 rank、伪造 parent、append 后回填、从 C 记录反推 Source。
设计向量分别保存这些反例，拒绝结果不记生产图验证信用。

G3现有154正图按旧 Compiler/Edition/Expected 继续原样有效，不要求每个旧图升级。
未来真正重新绑定时，有 Some 的图改变名义目标及相应 ranks；None 图也不能保证整体
身份相同，因为 full Compiler/新规范源和 owner tuple 都可能改变。全部154未来新图必须
重新独立对照；不得覆盖旧 Expected 或把旧成功称为 successor 生产实现。
