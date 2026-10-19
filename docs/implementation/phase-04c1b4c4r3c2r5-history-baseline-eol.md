# 冻结 Git 快照与原 Windows 工作目录账本的换行边界

第 3 组完整 Cargo 回归首次退出 0，运行前后精确 Git Blob 与原始字节也一致。
随后额外执行旧 `audit-substitution-staging.py` 时退出 1。该失败保留，不算审计
通过，也没有更改冻结源码或旧 Expected 来消除失败。

旧工具的输入 `phase-04c1b4c4r2-baseline.json` 是原 Windows 工作目录的原始字节
SHA256 账本，不是 Git Blob 账本。举例，同一份
`docs/implementation/phase-04c1b4a-type-field-inventory.md` 在旧 Git BASE 与第 3 组
HEAD 中完全相同：225,855 字节、LF、SHA256
`4fbb711cdd84638d20ca1094466d378a9c177a09a2ffae31009e15f0dac26e3f`。
原工作目录则为 227,916 字节，含 2,061 个 CRLF，SHA256
`d91561c7fb5ea9f135fbd56df8e0c14985ed8f2059e41fecca988f918dc6cb93`。
前后差异严格等于 LF→CRLF，不是规范或 Expected 内容变化。

独立受限辅助检查逐文件读取各自确切旧 BASE 的 Git Blob，只允许两种对应：
原账本 SHA 恰等于 Blob SHA，或恰等于该 Blob 的纯 LF→CRLF 展开 SHA。
任何其它差异或缺失路径立即拒绝。结果为：

| 组 | 原账本路径数 | 原始字节已相同 | 仅 LF/CRLF 不同 | 未解释差异 |
|---|---:|---:|---:|---:|
| 3 / BASE 51239a0 | 1,811 | 1,621 | 190 | 0 |
| 7 / BASE 2ad4359 | 2,170 | 1,958 | 212 | 0 |
| 8 / BASE 8dc4f50 | 2,235 | 2,014 | 221 | 0 |

原工作目录账本在主树与历史树忽略目录的 `*-original-worktree.json` 中保留。
另从每个精确 Git BASE 生成 Git 快照版审计输入账本，供未修改的旧审计工具
在精确 Git 归档上执行；这不是原 CRLF 工作目录冻结声明，也不是对旧 Expected
作归一化修改。所有跟踪源码、Registry、Compiler Identity、Expected 和历史
工具自身仍是其确切 Git Blob。源树在回归前后的原始字节一致性继续独立验证。

完整路径级对应记录位于
`target/phase-04c1b4c4r3c2r5-history-{3,7,8}-baseline-eol-receipt.json`。
第 3 组审计输入预检查已实际退出 0；随后重新运行第 3 组完整 Cargo 与审计，
最终结果另由执行回执封存。第 1、2 组已完成的独立源树/日志记录未被替换。
