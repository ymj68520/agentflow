# Scope / Canonical DAG 不变量

AF-SEMANTIC-0217 的规范源为 spec/tables/proof-report-model-v1.md，完整最小 DAG
约束由当前 Model@4 继承。不得改变 nominal typed equality、bottom-up interning、
规范 rank、集合语义、有限可达性或拒绝冗余相等节点的规则。

草案节点方向：

```text
RawValidationSubject -> RawParsedInput -> RawInputOrigin -> OccurrenceBinding
OccurrenceBinding -> SourcePackage -> OwnerRecords -> closed containers -> RawDocuments
OccurrenceBinding -> OwnerAnchor / actual slot key / actual_document
```

包/Owner/容器不引用 Binding、Subject、Scope、Fact、Observation 或 Report。
Anchor 中的 Domain 是输入标量记录，不是回指包含自己的 SourcePackage。
因此不会产生包→binding→包循环；共享 actual_document 与容器条目形成 DAG，不是环。
Scope.input 只保存上述真实输入闭包及原 Compiler/Context；生成的 Witness/Fact
仍在独立 output DAG，不加入 scope roots，不发生 Report/Scope 身份固定点。

同一个包内 Owner A 的slot0=X、slot1=X：X 的相等内容子树可以只占一个节点；
两个 Binding 的 ordinal 不同，所以是不同 typed values，各有独立规范节点。
两个真实 Owner 的相同容器也可共享内容；Anchor 的实际声明键不同，所以 OwnerRecord
及 Binding 不同。不存在“同值节点必须为了来源分配两个rank”的需求。

同一完整输入 Scope、同一来源第二次观察时，不把观察计数或消费者放进包；所有
输入值不变，最小 DAG 和来源身份不变。若真实输入包/Compiler/Context 改变则 Scope
可以合法改变，不能以 occurrence相同为由跨 Scope 强行合并 Fact。

Scope 保留完整实际 SourcePackage，而不是删掉其他 Owner 后的人工子集；这让
跨进程准入检查包内冲突有完整输入。未使用但属于完整声明容器的数据仍是该输入
的内容后代，不是人为填入的无关根。完全独立于输入的额外根仍拒绝。

跨进程验证接收完整包、引用和所选输入，重复检查完整结构与关联；不接收“已经
验证过”的 caller bool。完整 lossless Scope 实现仍属 G02，本文只给结构与算法设计。
