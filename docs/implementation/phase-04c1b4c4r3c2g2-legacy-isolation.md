# Exact legacy isolation / migration

五组分别使用自己的精确源码、Registry、CompilerSemanticsIdentity和Expected运行：

| 组 | 冻结完整HEAD | 历史/注册CaseIds |
|---|---|---|
| Edition1 | abc91d533b4d62e91226722e1d1fcaa30bd00883 | 512 |
| Edition2 | 51239a0045d7a6fb274bb24cdc8130f2346bf085 | 51 |
| Edition3 | 4d3bb59031795d31ecfccf29e3d9f2d89a7d9ce8 | 173 |
| Analysis@4 | 875e6e41f16845c78d4f6d31573826c547d48c07 | 889 |
| 起始Analysis@5 | e623aab705c5f8269807201e112a32e67e6869bf | 原262注册语料及C2诊断/现有准入全套 |

不是拿新全局Registry解释旧测试。五棵原始Git blob树共9542文件，前后核对；
各Cargo pass records另计，不等于CaseIds。实际退出、组数和源码收据见最终运行帐本。

原model@1/@2/@3/@4、旧58/479完整JSON records、97历史导入、所有旧Expected、
775 ABI、17 AFIDL、14 HashV1 domains、ACE1 grammar及历史digest意义冻结。
新增epoch5 Registry原字节快照供旧raw生成器使用，避免它用新62/493重新生成旧
compiler或Portable Expected。固定旧生成器输入不改变它的旧输出。

旧content-only Source在精确旧Edition保留旧合并/身份语义；在新Edition缺O则拒绝。
没有实际额外来源不能自动迁移、拆分旧合并Fact或生成Owner/slot。
UnmigratableMissingOccurrence状态保留；明确新包下的重新提交是新的来源建立，
不是恢复旧历史。贴旧标签的新内容、新标签旧内容、未知Owner/name等都失败关闭。

新当前Corpus在Analysis@6单独运行。旧原生端口算法和Byte lowering不是本次修复
对象，Native53/F1 62/40须新鲜独立重跑，原51完整Actual值必须相同。
R3-NG-B01持续关闭不代表G01/Raw/Native Full Seed完成。
