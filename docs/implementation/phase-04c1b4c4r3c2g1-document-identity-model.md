# 四种身份与不可能性证明

本文所有 Draft 名称均为设计名，不是已登记 ModelType、RuleId、Edition 或 wire ABI。

## 冻结模型的结论

令完整文档为 D、内部路径 P、Compiler C、九成员 Context K；其它 Predicate、
Rule、坐标、实际角色分区等均固定为 I。当前 RawDocument 仅保存物理 bytes 或
完整 declarations，RawInputOrigin 仅保存 document/path。因确定性与完整 typed equality：

```text
(DA, PA, CA, KA, IA) = (DB, PB, CB, KB, IB)
  => F(DA, PA, CA, KA, IA) = F(DB, PB, CB, KB, IB)
```

相等输入不能靠编码、换 rank 或多观察一次产生不同身份。不同地址不是该函数输入。
更改 Equality 或 AF-SEMANTIC-0217 会破坏现行身份合同，不能解决缺失输入。
新输入必须在 RawInputOrigin 的文档来源绑定处引入，并沿 Subject/Source/Origin
传播；不能只在最终 FactKey 后缀拼接 ID。

## 唯一推荐的语义分层

```text
ContentIdentity(D)           = 完整 nominal RawDocument typed value
OwnerIdentity(A)             = (namespace, domain_key, owner_key, revision_key)
DocumentOccurrence(O)        = (A, actual document-entry ordinal)
DocumentMember(M)            = (O, exact internal RawDocumentMember path)
RawPortOrigin                = (M, direction, original ordinal, exact DeclaredStep path)
ScopedFactIdentity           = 既有完整身份合同(新 binding 已进入 Scope/Source/Origin)
ObservationIdentity          = 原有 ObservationSite/consumer edge；不参与生成 O
```

身份计算不使用摘要代替完整内容。A 的各成员均为实际来源声明的 exact UTF8 key，
而不是 Collector 的新标签。Owner revision 是调用者声明的不可变版本键，不是本地
自动递增计数。相同 A 被当作同一声明式 Owner；对象重新分配不改变它。

Content 与 Occurrence 是不同等价关系：D 相等而 A 或真实条目不同，O 可以不同。
同一 A/ordinal 的 D 不同不是两个合法 O，而是 BindingConflict，先拒绝再谈身份。
完整 Owner、容器、文档、Compiler、Context 仍作为输入保留，不能只留下 A 的字符串。

同一 O 不保证在任何 Compiler/Context/Scope 下都是同一 Fact。重复观察合并的前提
是完整 originating FactKey 及所有非观察 payload 相等；跨版本、跨 Context 或输入
包发生实质变化时不得抹去 Scope 差异。Observation 不回写输入包，因此普通的两次
观察不改变该 Scope。

## 手算，不是运行结果

实际声明域 (team, submission-7)，Owner (bundle-main, revision-3)，两条真实文档
slot0=X、slot1=X。O0=(team,submission-7,bundle-main,revision-3,0)，
O1=(team,submission-7,bundle-main,revision-3,1)，故 O0≠O1，而 Content(X)=Content(X)。
slot0 再观察一次仍是 O0。第二个真实 Owner bundle-import 在自己的 slot0 声明 X，
它与 bundle-main/slot0 也不同。仅有两份 X、没有上述实际输入，结果为
MissingDocumentOccurrenceBinding，不得替它们创造 submission-7 或 Owner。
