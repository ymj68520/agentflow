# 四方案独立比较

| 方案 | exact Source / g_external | Pre-G 来源 | 可达与权限 | 裁决 |
| --- | --- | --- | --- | --- |
| A：真实 Native A 的正式名义投影，三 Some 子目标 successor 改绑 | 保持 | 六字段真实现存 | 沿既有根路径，正式新版本边 | 推荐 |
| B：C.source 改指 Content | 不再是要求的 NativeSource 名义 | 内容虽完整 | 可达但改变消费者 Source 身份 | 不接受 |
| C：独立 Source/Validation 关联记录 | 单独记录不生成所需 G Source | 若在 C 后则过晚 | W/output 不能充当 G external | 不独立解决 |
| D：cast/hash/shape/UUID/counter/address/history/backfill/extra root | 违反至少一项 | 真实性无法成立 | 非法转换、回边或不可达 | 排除 |

## A

所需全部 Native 字段已经由 input_source.rs::native_relation() 实际读取，既无需 C 判断，
也不要求已遗失的构造 Schema。NativeSource 六字段与封闭 Catalog Variant 已正式定义。
三个 Some 目标的正式 successor 改绑构成合法新的生产边，不是对当前图做 alias cast。
Site registration、Compiler/edition 和递归引用闭包必须先正式注册；G1 不实施。
不用新节点类型、额外 Root 或新 NodeBody。Source 指向真实 A，C 指向先前 Source。

## B

RelationSnapshotContent 不是只有 digest 的贫乏值，其委托 body 现已包含全部 A 字段；
不能以“Content 没有这些字段”作为排除论据。但它仍是**不同的正式名义类型**。
将 C.source 改为它，会撤销用户要求保留的 exact Source 类型，改变 Source/Observation
消费者的名义契约；完整字段相等不证明两个注册身份相等。当前规范还允许其它 Context/
Manifest Content 用途，不能从形状相同推出这些用途都已取得 Native Source 关联权限。
因此 B 不满足本次“保持完整 Native Source 名义身份”的约束，不是代码量判断。
如要允许 B，须另行授权改变该权威约束，而非本轮自行选择。

## C

Derived association 若只引用现存 Alias，依然不能提供 exact Source；若放 W-local，违反
g_external；若 C 后追加进 G，违反0297/0308；若在 G 前注册一个 Source 加合法父边，
则它已经承担 A 的核心正式投影，关联记录只是另一种容器而非独立替代方案。
既有 Subject/Origin、Context 和 C.source/context/target 已保留关联，不存在需要另增
Source/Validation 回指记录才可表达的空缺。本最小方案无需扩大 W 或 output 权限。

## D

Nominal cast、same-shape reinterpretation 违反0303/0306；hash-only 不比较完整 typed
内容；UUID/counter/address 制造未注册身份；invented history 违反0298；C-result backfill
形成 G→C→G；unreachable Source/extra root/duplicate equal node 违反0217。
这些都是明确负例，不是未研究的备选实现。

唯一性是**满足本次固定权威约束的语义路线**，不是声称任何未来数据布局都数学上唯一。
完整 Source 必须在 G 中、先于 C、经正式输入边可达；所以必须由 A 的实际投影形成。
在这个必要语义类内，本推荐选取不增类型/根/Body/输出权限的三输入边改绑最小形式；
没有再并列推荐第二方案。正式登记前仍须独立验收该确切草案。
