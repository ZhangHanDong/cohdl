# 2026-09-10：校正 RFC 依据，研究 tscircuit

> **后续依据校正（2026-09-10）：** 已拉取 main `0e3d770`，确认 `docs/design/rfc-032-subdesign.md` 为 Accepted 且收录于 note 10。此前“RFC-032 已关闭、不能作为 M2 依据”的判断混淆了同编号旧 harness 提案，现撤回。以下保留当时操作和验证事实，不再沿用其中的排除判断；最新决定见中英文 M2 草案的“main 基线与必须完成的组合修订”对应章节。

用户指出可编程 RFC 尚未定义清楚 CoHDL 应有的可编程能力，随后明确 RFC-032 已关闭、不能作为本次依据，并要求研究行业开源项目，指定本地 `/Users/zhangalex/Work/CoHDL/tscircuit`。

本次从中英文 M2 草案移除 RFC-032 特有的端口、数组、布局内部访问、整体变换、错误码和验收依赖；配套原则指南和解读同步校正。既有 subdesign 实验只保留为历史实现观察。可编程模型仍需独立论证，此项校正不表示否定复用需求，也不说明历史实现已经被删除。原始实验数据和样例未修改。

本地 tscircuit 入口包为 `7afedf434c3fdb7528191db84d9571bc9bbcce67` / `0.0.2478`。它没有已安装的 node_modules，主要委托 core/eval/CLI。为读取实现，在 `/private/tmp/cohdl-ref-tscircuit-core` 获取 core 提交 `e1f5a0edc482897d67969e694c8fbe3e18d4f339`，声明版本为锁文件对应的 `0.0.1875`；未验证与发布 tar 的逐字一致性。

已跟读 React 元素到对象的转换、分阶段 render、Circuit JSON 返回路径、重复电容构造、函数组件复用、作用域连接和错误记录。结论及固定源码链接进入[tscircuit 参考章节](../cohdl/tscircuit-reference.md)。另查询了 Polymorphic Blocks、atopile 和 SKiDL 的官方资料，作为后续比较入口；不把 README 宣称记为本地运行结果。

本次未执行 tscircuit 上游测试、安装完整依赖、操作 KiCad、生成制造文件或记录物理测量；没有新增学习者独立掌握证据。

文档验证：`mdbook build book` 与 `python3 book/tools/check_book.py` 通过；本机新章节返回 HTTP 200。中英文 RFC 的源码哈希、代码块、标题层级、表格行数和错误码集合一致，正文未残留 RFC-032 专属契约。源码链接对应文件存在，六颗电容的 TSX 节选与固定版本源码一致。独立审查后补齐字符串求值测试引用，将跨层访问结论改为引用选择器与端口解析实现，并同步移除两种语言预算说明中的“节点”残留；这些检查不代表执行了候选 CoHDL 语法。
