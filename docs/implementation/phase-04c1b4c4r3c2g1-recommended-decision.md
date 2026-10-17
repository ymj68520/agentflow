# 唯一推荐决议 / Option C

推荐采用 owner-occurrence-binding.md 中唯一闭合方案：
SourcePackage + OwnerAnchor + 实际容器条目ordinal + 完整RawDocument，
在新RawInputOrigin处引入DocumentOccurrenceBinding。RawDocument内容类型不变。

充分条件须同时满足：

1. 调用者在验证前真实提交完整来源域和封闭容器；键不是Collector补造。
2. Domain/Owner/revision齐备且作用域匹配；完整包无重复Owner/歧义/冲突。
3. Key选择真实Owner容器条目，完整文档、内部路径与候选严格一致。
4. Physical/Native采用明确允许矩阵；不得在一个输入中混入矛盾来源。
5. 精确Compiler、九成员同对象Context分区及全部旧Raw准入前提仍满足。
6. 只建立Declared Provenance；外部认证/全局唯一性不能从键的字符串推出。
7. O沿Subject/Source/Origin/证据闭包无损传播，DAG/观察/冲突规则保持正确。
8. 未来单独授权注册新的完整语义元组及封闭schema；旧元组不变，不反向补旧历史。

这些条件让“同内容不同实际声明来源”可表示，也让“只给两份相同值”的请求明确
失败关闭。因此本设计无需未知的外部认证服务才能完备；它并未保证任意输入成功。
需要认证durable handle的调用不能满足充分条件，继续拒绝，不偷换成声明已认证。

本阶段的 DESIGN COMPLETE / AMENDMENT DESIGN ACCEPTED 只表示设计交付完成。
C2-GAP-01 FORMAL REGISTRATION PENDING，实施仍阻断；C1-GAP-01保持已注册。
R3-C2/Typed Capture/G01-T3 incomplete；T4 not eligible；无额外权限。
