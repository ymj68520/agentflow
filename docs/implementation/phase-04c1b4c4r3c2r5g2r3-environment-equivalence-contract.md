# R5G2R3 — 环境等价契约草案

状态：设计草案，非新增正式Specification；环境资格、真实签名验证及跨机执行均 **Not Executed**。源码身份、构建配置、环境身份、资源政策四维分别匹配；一个维度不能弥补另一个维度缺失。

每Gate从冻结HEAD生成完整路径/长度/raw SHA256/GitBlob清单。静态采集已逐文件验证现有九个冻结树；完整清单放在ignored target，紧凑摘要及路径记入机器契约。Registry覆盖spec/registry；Compiler覆盖crates与Cargo.toml/lock；Expected覆盖完整spec/conformance；Oracle覆盖全部tools。类别为保守超集，跨类别依赖仍由完整源码清单绑定，不能只校验几个入口文件。

批准环境必须提供：Windows版本/架构、x86_64-pc-windows-msvc ABI、精确rustc/cargo/rustdoc/LLVM/linker/MSVC/SDK二进制与运行依赖身份；Python解释器/标准库/所需包身份；锁文件和实际解析的依赖源码哈希；所有Cargo配置层、环境/RUSTFLAGS、features、target、路径和build-script/proc-macro输入。缓存.rustc_info只是旧自报，不能取代这些身份。

构建输出按Gate隔离，不共享未经认证的target、增量缓存或生成文件；禁用网络补取和隐式环境继承，所需依赖须事先审核。冻结源只读，临时输出单独目录；所有允许写入范围与生成sidecar必须列入批准清单，不得回填Expected或修复历史输入。构建前后全量源码复核。原有按路径EOL对应关系只能引用既有批准及精确两侧哈希，不得新增通用换行归一化规则。

路径重定位须证明：逻辑源路径映射唯一，盘符/大小写/Unicode/长路径/符号链接策略明确，CARGO_MANIFEST_DIR、include文件、相对fixture和运行cwd仍有效；不得把路径差异全部忽略。对编译嵌入绝对路径、PDB和非确定性元数据，原始产物哈希保留用于来源，不强求不同机器必然得到同二进制。任何允许的输出差异都须有事先批准的逐字段规则；没有规则就拒绝，绝不编辑实际或Expected以匹配。

可信根不来自提交者：完整CaseId、target、产物、输出字段分母和批准schema必须在证据外独立冻结。分别验两侧完整性后才能比较；必须检查重复、缺失、未知、未比较项以及每个原始值，防止both-sided omission和缩分母。CaseId分组计数与Cargo test记录不同，不能互相替代或重复记账。精确CaseId/target清单的后继资格采集尚未运行，不能凭现有计数准入。

证据包需同时绑定EnvironmentId、执行者、资格回执、批准资源政策、Gate/输入/配置/命令/产物/输出/Oracle/权限拒绝和完整时序。执行者凭据由用户/独立认证者线下固定，不能信任清单自带的trusted=true或自签公钥。传输前后校验完整文件清单、长度和SHA256；在读取任何内容前限制文件数、总字节数、单文件大小、嵌套深度和路径，拒绝绝对/越界/大小写冲突/链接。证据仅数据，不执行其中脚本。

签名设计：域分隔R5G2R4-EVIDENCE-v1、独立schema/requirements摘要、canonical UTF-8 JSON（键排序、无重复键、无NaN/浮点；大小和整数范围受限，时间使用精确UTC整数纳秒+单调时钟序列）、detached签名及完整包摘要；签名算法/库/密钥撤销与重放防护必须另行选定验证。签名覆盖nonce、Gate执行序号及源/环境/资源/输出全字段，拒绝旧包重放。签名实现和真实资格证明均未完成，本轮toy sentinels不是密码学认证。

已允许的仅是原有精确冻结身份及既定历史对应关系。新主机认证、路径重定位、跨机传输、构建/执行分离、编译保护修订均**没有从本轮取得实施权限**。缺任何PASS必填字段、非零退出、取消/超限、未认证执行者、无真实运行时间、产物缺失或未知Oracle结果都失败关闭。真实模板为空值，不可用于PASS。

执行回执不是生产证明：Native/Portable/RootExport保持原拒绝边界，CompleteView/ObligationIdentity不签发；签名运输包本身不得变成任何能力令牌。
