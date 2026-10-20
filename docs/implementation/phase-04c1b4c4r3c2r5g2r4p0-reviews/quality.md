# P0 Quality / Independent Oracle Review

Static PASS；同一代理的独立回读，非外部认证。主验证器调用已有JSON Schema实现；独立Oracle不import/call它，不读取它的Actual，而以另一个Schema子集解释器和正交合同比较评价相同合成输入。Expected在static-test-vectors.json独立手写，遗漏矩阵按固定必填规则推导。

105项Manifest向量与独立Oracle一致，涵盖全部root/candidate必填遗漏、双侧字段/CaseId遗漏或共同伪造、源/Compiler/Oracle归属、错误HEAD/profile/命令/DocTest、越权资源、环境冲突、来源/批准/执行者/信任缺失、重放、状态/时序、Lane B越界和权限升级。未知候选可以Static PASS，但拒绝资格/执行；测试分别断言blockers与Not Qualified/Not Authorized，绝不把“结构可表达未知”当环境通过。

原R3 110项及原Expected字节保全并重放静态检查；5项公开Ed25519验证接口向量、6项严格解析拒绝、7项本地文件身份/路径检查通过。公开RFC key/signature不是生产根，无私钥或临时密钥生成。真实Trust Policy和执行者缺失，密码学有效不能升级为权威。

Lane B原2403路径、四清单、41脚本和82原日志重新按字节/hash核对，既有范围信用保全；没有生成器新执行或扩展树信用。已封存失败/中断/通过回执不改写。机器回执target/phase-04c1b4c4r3c2r5g2r4p0-review-quality.json绑定受审文件。
