# 历史隔离与迁移

Analysis@8 / Model@7 / G2 registration / G3 production binder / R5 Boundary Actual /
Old Raw Expected 均原字节保留。旧规范、Registry、Compiler、production 源码没有变化，
因此八组历史构建本轮使用请求明确许可的状态：
`Not Rerun — Referenced R5 Frozen Evidence`。
这不是本轮新鲜执行；引用 R5 final runtime receipt，登记其 SHA256。

旧 Content Node 仍按旧 owner/Descriptor/ModelType rank/Root 理解，不改名、不升级。
未迁移 Source 的旧 G 保持合法旧输入，仍不能满足当前 C.source 的 exact Source 引用。
提供新 Context 重验不能赋予它新 Source nominal identity，也不恢复未保留的构造历史。

| 历史输入 | 后续处理 |
| --- | --- |
| 只有旧图/digest 或不完整 A 字段 | 不可自动迁移，保留旧版；不得默认空 Source |
| 六字段完整但缺真实 source/native Context 关联 | 字节数据不足以 issuer admission；不能自动升级 |
| 重新实际提供完整 A/package/Owner/O/context | 以后可按新正式 tuple 全新 admission+binding+C；新 G、新 ranks、新身份 |
| 需要原构造 Schema 历史但从未保留 | UNAVAILABLE，不能从当前 Context 恢复 |
| 旧 Source 引用混入 successor G/W | cross-edition 或 invalid reference，拒绝 |

重新绑定不是身份保持迁移，也不沿用历史 C、Raw Fact 或 RootExport 能力。
三 Slot Some 图必有 nominal/source target 变化；None 图也因完整新 Compiler/owner 可能
改变，绝不能保证旧154图中任何一个 successor 身份不变。未来新 Expected 必须另存。

R5 历史工作树 CRLF 与 Git archive LF 的逐字节对应回执保留。本轮原树 Raw SHA256
与历史 Git blob 身份是两种不同证据；不执行换行规范化，不宣称二者原始字节相同。
本轮 source-freeze 保存起始2343个实际非忽略路径/原始 SHA，新增设计材料仅独立增加。
