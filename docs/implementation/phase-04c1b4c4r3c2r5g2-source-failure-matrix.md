# Source binding / source failure / metadata boundary

实际Native来源读取合同保持kind、schema_family、revision、pin、canonical_content、catalog
声明顺序；来源为同一实际Context的确切typed Slot Snapshot getters，在G前存在。
实际构造Schema历史不保留且不恢复。当前SchemaDigest不等于A checksum/pin。

| 失败类别 | 既有domain/reason复用 | 真实阶段/约束 |
| --- | --- | --- |
| Some缺真实A或缺已知节点 | IncompleteFacts/missing_node | source_admission；不得改None |
| 错family/slot/native关联 | InvalidFacts/source_association | source_admission；Source六字段/Parent Slot同时核对 |
| 缺完整Source字段 | IncompleteFacts/missing_node 或invalid_shape的闭合结构诊断 | Source/registration；无默认空Source |
| 需引用的精确名义类型错误 | InvalidFacts/invalid_child | binding/consumer；Alias不是Source |
| 缺正式站点/边 | IncompleteContent/missing_site | registration/binding；不即时补规则 |
| 错g_external目标/环境 | InvalidFacts/invalid_child | C/consumer；W-local不能代替G |
| 跨版引用 | UnsupportedModel/cross_edition | registration；Portable外层保留InvalidIdentity/UnsupportedSemantics |
| 不可达/重复相等节点 | InvalidFacts/invalid_interning | 0217；不补Root |
| Rank/child顺序错误 | InvalidFacts/rank_dependency或invalid_child | pre_capture_binding；不复用Type rank |
| 输出进入输入 | InvalidFacts/input_output_backedge | G构建；C不回写G |
| 超界 | ResourceLimit/resource_bound | 实际触及阶段；无截断成功 |
| 未实现生产入口 | NotImplemented/production_not_implemented | 新版G/Raw；不是port Unproven |

新版当前API的Source检查只证明metadata与注册Source语法/来源字段自洽，不能证明
真实native borrow。metadata检查返回RegisteredModelFailure；不是原生Source admission
receipt。未来实际producer必须用完整StructuralBindingFailure(stage/site/typed paths等)
保留真实来源及冲突两端；此阶段不伪造该receipt或更改任何真实Native失败值。

None三Slot分别不产生Source、不执行Snapshot C；其absence不是Valid Snapshot证据。
Some/C-Missing保留真实A、G、port event/verdict，禁止完整Fact。C其它状态按0286..0298
真实阶段适用，Source refusal后NotReached，Found不等于Valid，RevisionMismatch只在
实际provided expected pin下适用；Native CompiledModuleSet的唯一性不伪造Ambiguous分支。

Owner/O/member/Origin保留在真实Subject父关联；完整相等Source可intern但父边不删。
新Typed Source通过原Context/Slot/Some路径到唯一RawSubject根，不加根或Body case。
C.source的output_record与w_value均精确g_external；每个metadata subgraph是局部声明
子树，不冒充完整Raw Subject输入图或Root-export Scope。
