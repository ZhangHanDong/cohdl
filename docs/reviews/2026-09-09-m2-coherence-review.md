# M2 RFC 整体一致性复核 — 2026-09-09

> **后续依据校正（2026-09-10）：** 已拉取 main `0e3d770`，确认 `docs/design/rfc-032-subdesign.md` 为 Accepted 且收录于 note 10。此前“RFC-032 已关闭、不能作为 M2 依据”的判断混淆了同编号旧 harness 提案，现撤回。以下保留当时操作和验证事实，不再沿用其中的排除判断；最新决定见中英文 M2 草案的“main 基线与必须完成的组合修订”对应章节。

对象：[有限编译期可编程能力草案](../proposals/rfc-draft-bounded-compile-time-programming.md)。依据：[设计原则 P01–P10](../language-design-principles.md)、Accepted 规范，以及已取得的 main `ab44257`。本轮修订的是提案，不修改编译器或 Accepted 规范。

结论：数组、有限循环和共用表达式的方向成立；原草案仍有影响判定、输出与工具行为的未闭合契约。以下问题已在草案中给出具体修订；完整范围与几项设计成本仍需接受时决定。

| 级别 | 原问题与反例 | 修订处理 |
| --- | --- | --- |
| P1 | 参数绑定后、空循环进入前没有明确检查点：实际 `f::<0>()` 内部 `1 / N` 何时失败不确定 | 明定声明、实际调用绑定、实际迭代三个环境，共用检查机制；跳过的调用不特化被调用者，但实参自身的已知错误仍检查 |
| P1 | `actual elaboration uses` 未定义，无法判断无用 const/默认值是否启用整板预算；可能分配大量对象后才开启 | 在任何物化前按选定设计的静态引用图选择模式，明确无用 const、空循环、Int 默认值及未调用定义的边界，公开兼容成本 |
| P1 | 只数实例/调用，漏掉 helper 中 net_class 等布局约束；十万次调用一万个约束可放大到十亿项 | 每个展开约束、物理属性记录及其引用都计量，在加入延迟布局队列前计费 |
| P1 | “新特性区域”沿引用闭包强化旧 helper：给未调用函数加无用 const，可能使旧依赖的重复局部名突然报错 | 删除传递强化政策；新构造的静态规则与既有声明检查组合，旧 helper 不因调用者新增 const 获得一套额外规则 |
| P1 | 派生 Length 只有数值规则，没有用于诊断/输出的 text 规则 | 指定 i128 × 10^-15 mm、共享最短小数输出；旧字面量与纯参数转发保留原文，几何消费者范围单独检查 |
| P1 | 上下文只写 secondary/help，LSP 能力关闭及 Explorer 映射会丢弃它们 | 主消息携带必要的具体值、边界、展开路径与绑定；附注提供更多源位置 |
| P1 | 现有 API docs 仅有扁平实例/调用/网络摘要，不能准确表达未特化 N 和嵌套循环 | 提议包文档 schema v2、整数参数描述及 canonical body_source；保留旧 v1，明确服务器与查看器支持和失败降级边界 |
| P2 | loop 词法白名单和跨调用权限混在一起，会把允许的 helper inst 又禁止掉 | 区分词法范围选择与语义权限；直接 inst 暂缓是范围成本，不能包装成安全要求 |
| P2 | 只验证 fmt 幂等、概括验证“一个超限”，以及仅固定 source/lock 的重建前提 | 增加优先级改变的反例、三项独立边界与混合旧/新语法；固定完整输入，包括 DXF |
| P2 | Alternatives 把较小方案写成 `n + 1` 特例，遗漏同样共用表达式但暂缓 const/整数泛型的可行方案 | 加入真实较小方案；完整范围的推荐以数量参数化库是本次必需为条件 |

## 核验依据

- [声明检查的已知范围](https://github.com/conol-ai/cohdl/blob/ab44257/src/check/bodies.rs)：未调用函数的部分重复局部名、循环调用与布局检查留到实际展开。
- [展开与延迟布局队列](https://github.com/conol-ai/cohdl/blob/ab44257/src/check/expand.rs)：helper 约束会实际产生数据；派生 Length 已有统一文本生成先例。
- [UnitValue](https://github.com/conol-ai/cohdl/blob/ab44257/src/units.rs)和[几何输出](https://github.com/conol-ai/cohdl/blob/ab44257/src/emit/geom.rs)：固定点与原文 text 是不同字段。
- [LSP 诊断映射](https://github.com/conol-ai/cohdl/blob/ab44257/src/lsp.rs)与[Explorer 映射](https://github.com/conol-ai/cohdl/blob/ab44257/explorer/extractor/src/project_model.rs)：附注不是所有消费者的共有字段。
- [API docs 提取器](https://github.com/conol-ai/cohdl/blob/ab44257/src/emit/docsjson.rs)与[现有文档协议](https://github.com/conol-ai/cohdl/blob/ab44257/docs/apidocs.md)：整数参数、符号长度和循环不能靠“自动兼容”解决。

本轮实际运行两个当前语法探针：helper 含两个 net_class、被调用两次；未调用的旧 helper 含重复局部实例名。两者在同一 main 二进制上均 check 退出 0、无诊断，分别确认预算覆盖面的前提和旧声明检查边界。源码、命令、原始 JSON 与二进制哈希保存在 [review-2026-09-09.json](../proposals/fixtures/m2-programmability/review-2026-09-09.json)。没有执行巨量对象实验。

## 尚需在接受时作出的选择

1. M2 是否必须同时交付局部常量和整数泛型，还是先交付共用表达式、`.len` 与循环。
2. 是否接受所有循环命名及首版禁止直接 inst/subdesign 的书写成本。
3. 是否接受使用新语法后整板启用预算的兼容性边界，以及具体上限。
4. 是否接受完整方案带来的包 API 文档协议升级，并安排 compiler、registry 和查看器共同交付。

这些是已摆到台面上的设计取舍；本轮审查不替代正式接受。新循环语法、未来反例与构建验收均未声称通过。
