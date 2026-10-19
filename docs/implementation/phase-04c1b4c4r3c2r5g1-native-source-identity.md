# Native A、类型内容与观察身份

正式依据 AF-SEMANTIC-0286/0287/0293/0296/0298、0280/0281/0282。
Native `RelationCatalogSnapshot<T>` 的私有字段为 schema、revision、canonical_bytes、
catalog、pin；derive PartialEq/Eq 保持原意，没有原始构造 Schema 的保留字段。
new() 曾使用提供模块编码，但只保留上述完整结果，不能从当下 Context 恢复历史 Schema。

| Source 字段 | C 之前真实读取位置 | 禁止替代 |
| --- | --- | --- |
| kind | 实际 relations 中的命名 Slot / T | 从 C Valid 猜测 |
| schema_family | snapshot.schema() / T::SCHEMA_ID | 当前模块最新名字 |
| revision | snapshot.revision() | Schema revision 或临时值 |
| pin | snapshot.pin() 的完整 family/revision/checksum | 仅 checksum |
| canonical_content | snapshot.canonical_bytes() 完整字节 | 重新编码后覆盖原字节 |
| catalog | snapshot.catalog().to_value() 的闭合类型化投影 | Debug 或 schema digest |

input_source.rs::native_relation() 已在 G 之前直接读这些真实借用；C 没有调用它的
生成权限。原始 package、OwnerAnchor、entry ordinal、member path、physical 路径由
输入 admission 关联；analysis/canonical_validation 必须是同一实际 Context，不是
内容相等的一份复制品。Context 和 Snapshot 的借用关系保留在私有 issuer，地址不编码。
未来改绑只改变投影的**正式节点类型与子边**，不改变这些 native 读取或来源真实性要求。

Source value 是六字段 canonical input；它不是独立的“每次使用事件”。
同一个完整相等 A 的重复合法使用不应生成新 Source。不同 Owner/Occurrence 的使用关联
由 `(complete source package, OwnerAnchor, actual entry ordinal, member path, Subject,
named relation slot, actual Context association)` 保留，不能只记 Source Node。
这些坐标沿既有 input.origin.document_binding 和 Subject/context 父边可达，不新增 UUID。
两份完全相等 Source 可以 intern，共享子节点不删除两条真实父边或两个不同 Origin。
这正是0281的内容共享与 occurrence 区分，不是把 Owner 并入 Native Snapshot Equality。

三 Slot 的 nominal Source type 相同，但 kind、family、pin family、catalog variant 的
正式闭合区别保留；即使空 catalog 编码字节偶合，也不能跨 Slot 合并或交叉使用引用。
不同真实 Owner 本身不使相等 Native A 变成不相等；不同 originating association 不得被合并。

同 A 在 Context K1 Valid、K2 Missing：A 的六字段保持相等；完整 G 可不同，因为 current
Context 模块输入已不同。Source 子值相等不保证跨载体 DagRank 相同，更不许可跨 G 引用。
C 观察应区分确切 G/Subject、Context、Slot、Compiler/algorithm、lookup/validation 全字段；
不能因 Source 相等把两个 C 合并。改变 C 输出而不改变输入 G 的实验不允许回写 G。

设计向量含重复来源、另一真实声明 Owner、三 Slot、同 A/不同 C、原始 A 不变和双侧
遗漏反例。另一 Owner 的配对是设计重放输入，不冒充本轮 Rust 已执行的原生借用。
不存在必须恢复而不可得的构造历史字段；若未来另要求该历史，本方案不能提供，须另立缺口。
