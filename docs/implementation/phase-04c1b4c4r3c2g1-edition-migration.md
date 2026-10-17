# 版本影响及失败关闭迁移

当前 Analysis@5 / Model@4 / Report@4 / Obligation@4 全部冻结，不分配下一编号。
即使RawDocument内容Descriptor不改，新绑定沿整个身份闭包传播后仍改变整体语义。

| 层 | 未来必须审查/登记的变化 |
|---|---|
| Normative Source | 新来源声明/Owner/occurrence/信任/冲突/允许矩阵及原路径关联语义 |
| Descriptor Type Identity | 新封闭包/Anchor/容器/Key/Binding、RawInputOrigin/Physical及所有递归依赖，未知字段/Variant拒绝 |
| Canonical Encoding | 新typed输入内容、declared顺序/宽度/nominal key；ACE1 grammar及HashV1算法不随意改变 |
| CompilerSemanticsIdentity | 完整新规范/Registry/table/role/source内容；不能只是换显示版本号 |
| Report Shape | Source/Origin/Basis/Witness/reference等完整Descriptor闭包；parent编码变化必须更新所属身份 |
| Obligation Identity | 新Scope/Source/Origin影响完整key，旧key不可直接当新key |
| Portable Admission | 支持精确label/content，默认仍fail closed；不把Registered等于Supported/Activated |
| 历史兼容 | 精确旧label/旧内容继续旧解释；旧同值来源合并不是新版独立occurrence |

没有扩充Port predicate族；ABI775/AFIDL17/14个HashV1 domain当前均不动。
未来注册也应把本次模型语义变更与传输ABI/哈希域分配分开审查，不预认定需要或
不需要某个新编号。不得让现行完整语义内容身份下的descriptor暗变。

## 准入矩阵（未来设计，不是当前实现结果）

| 输入 | 结果 |
|---|---|
| Old content-only Raw Source + 精确旧Edition | 依旧内容身份解释，不满足新occurrence能力；旧支持状态仍决定可用性 |
| Old bytes/旧字段贴新版本标签 | UnsupportedModel/identity mismatch，不推断Owner |
| 新结构缺Owner/key/实际来源包 | MissingDocumentOccurrenceBinding，无fallback |
| 新结构有真实完整包及受检O | 可建立DeclaredOnly来源关联，尚非Fact或Checked |
| 同Owner/key冲突内容或矛盾关联 | BindingConflict / InvalidOccurrenceAssociation |
| 未知Owner/Occurrence Variant | UnsupportedOwnerVariant/UnsupportedModel，无开放Other |
| 当前值重新获得真实新来源声明 | 可显式重新提交并在未来受检，属于新来源建立，不恢复旧历史 |
| 只有旧摘要/旧Report/两个旧地址 | 无可迁移来源；明确UnmigratableMissingOccurrence |

跨两个输入包联合审查时检查Anchor/完整Owner/文档冲突；孤立进程没有外部完整历史
则不能证明全球不复用。历史来源证据缺失不是G02序列化实现可以事后生成的数据。

正式登记后仍应经过单独Producer/完整字段Oracle/Scope/Normalizer/能力拒绝的实施
授权与验收；本设计不自动恢复R3，也不使T4 eligible。
