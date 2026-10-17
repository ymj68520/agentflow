# Physical / Native 映射

推荐保持 RawDocument 的两个内容 Variant 不动，把新 occurrence 放在外层 binding，
避免把“文档值”与“真实来源条目”揉成一个可绕过去重的对象。

| 项目 | Physical | Native |
|---|---|---|
| actual_document | physical_json_utf8 的完整原始字节 | declared_native_document 的完整十三字段声明序列 |
| 新来源层 | 实际 SourcePackage/Owner/容器slot | 同左；不是从 Native API 自动补造 |
| 旧内部路径 | 原字节中真实成员路径，严格解析选择 | 完整声明序列中的真实条目路径 |
| 物理关联 | supplied 必须绑定同一个 occurrence 并保留 value_path | not_supplied 合法，绝非空bytes或历史bytes已知 |
| 结构失败 | UTF8/重复键/未知字段/错误宽度等先拒绝 | 实际完整 typed values 与所选条目一致 |
| 历史主张 | 不用重序列化产物冒充原字节 | 不从当前值猜测当年 Raw bytes/Owner |

推荐新 RawPhysicalAssociation.supplied 也指向同一个 DraftOccurrenceBinding，
而不是再单独引用一个不带 occurrence 的 RawDocument。若 input.origin.binding
与 physical.binding 的 O 不同，即使字节完全相同也 InvalidOccurrenceAssociation。
初始方案不允许一个 Raw input 同时声称两个独立 Owner 分别是自己的原始来源。

初始新方案的允许矩阵明确为：physical origin + supplied(同binding)，以及
native origin + not_supplied。physical origin + not_supplied 或 native origin +
supplied 返回 UnsupportedPhysicalAssociation，不自动推断或补造关联。
Native文档值与Physical文档值属于不同RawDocument Variant，不能把它们称作同一个
binding.actual_document。若真实输入有两种独立来源，必须另行注册转换链/双视图
关联；本次不暗加转换receipt。实际Physical来源解析得到的Native候选仍然是
RawParsedInput.candidate，它的origin是Physical，并不需要额外造Native文档Owner。
这项新profile准入限制须在未来注册中明示；旧Edition准入不在本阶段改变。

Physical 同 bytes、不同合法 Owner/slot => 同内容、不同 occurrence。
同 occurrence 同原始 bytes重复验证 => 同来源；不同原始 bytes（包括空白、NFD/NFC）
复用同 Owner revision/slot => BindingConflict，不是自动canonical相等。
Native完整typed值相等但不同Owner/slot同样可区分；只是在本地复制同一来源对象则
不产生新 occurrence。所有新设计结论均非当前 Producer 执行结果。
