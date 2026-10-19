# 名义类型与 DagRank 独立分析

当前 exact owner 是 `agentflow.computation.proof-report.model@7`。
`RelationSnapshotContent`：rank107、alias body ref NativeRelationSnapshotSource。
`NativeRelationSnapshotSource`：rank754、六字段 record。
两个 eligibility 都包含 g_input；二者不是相同 ModelType，不自动共享名义节点。
Body/site 委托不是 `Content -> Source` cast（AF-SEMANTIC-0303）。

未来 Option A 不创建“旧别名的 Source 视图”。它从 actual native A 另按 successor 的
确切 Source Descriptor 投影，正式 Some 目标即 Source。原别名声明保留为旧含义和
其它路径使用；不倒置 alias 方向、不使所有 aliases 透明、不统一两个名义类型。
当前 NativeSource **已注册**，因此此方案不需要凭空新增第二个 Source 名称或 rank。
未来 source owner 属于 successor Model；具体 Edition 不分配。本轮不向当前表追加值。

Body 为 record，字段严格按 kind/schema_family/revision/pin/canonical_content/catalog
声明顺序，子引用按现有完整字段 Descriptor 的确切名义/site 类型处理。
Source type 不是普通 U64；C.source 的 Node<T> 才是 fixed G 中解析到确切 T 的 DagRank。
Source schema pin、ModelType rank754、source entry ordinal 都不可作该 DagRank。

Source typed equality = exact ModelType + complete typed record/children。Alias 的 body
相等不能省去 ModelType。完整字节参与原值比较，checksum 不替代 catalog/bytes equality。
Native Snapshot Eq 不受此新增 carrier 含义影响。

高度 h 为叶0，否则1+max(children h)。在完整最小 G 中按 height，然后 owner 的 unsigned
UTF8、active declared ModelType rank、exact name、nonrecursive body key、ordered child
ranks 规范排序，分配连续零起点 U64。所有 child rank < Source rank < Some rank < parents。
即使未来某数值碰巧等于旧 DagRank，也必须由全图重新推导，绝非复用或跨版引用。

本轮 Oracle 输出 symbolic rank dependency 而非真实数字；完整 Compiler、输入图全体
节点和未来登记 tuple 尚未确定，不能伪造“生产 DagRank”。具体注册时维护旧 rank prefix，
新增类型仅在确需新结构时追加；本推荐无新增类型需求，Opaque Site 的 successor 定义
须有正式版本身份，不得原位改写旧表。
