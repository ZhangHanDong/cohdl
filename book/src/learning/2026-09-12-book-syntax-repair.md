# 2026-09-12：修复语法章节的几何说明与能力边界

## 本次任务

用户要求修复 Claude Live 新增语法章节的剩余审查问题。本次修订教材、示例注释和阅读入口；语言实现基线仍为 CoHDL 0.7.0 / main `0e3d770`。

## 修正内容

- [物料与封装](../cohdl/syntax/parts-footprints.md)：环形 `annulus` 的尺寸是外径、内径，要求外径 > 内径 > 0；`courtyard` 只接受 rect/circle/oval。按表格分别解释矩形、圆形和分段环形钢网开口，其中 `paste: circle(d)` 可以大于铜面。
- [语法总览](../cohdl/syntax/index.md)：当前实现包括 Accepted RFC、已实现的 provisional 语法和已记录的偏离；不把它们混成同一种规范状态。
- [单位与器件](../cohdl/syntax/units-devices.md)：电流规格仍有字段和泛型单位检查，尚不自动求解工作电流或检查电流上限。
- [设计与网络](../cohdl/syntax/design-nets.md)和[复用](../cohdl/syntax/composition.md)：位号稳定性依赖身份路径和设计锁；subdesign 直接声明的具名实例，与其内部 fn 按调用序号生成的对象，需要分别说明。
- 同步语法示例的 inst 注释，并把 Book README 的声明数更正为九种，明确示例覆盖常用体内语句。

前两轮审查与复现依据保留在仓库 `docs/reviews/2026-09-12-book-syntax-live-review.md`、`docs/reviews/2026-09-12-book-syntax-live-followup.md` 及对应 evidence JSON 中。它们记录修订前的错误和当时结果，不能当作当前章节仍未修复的清单。

## 实际验证

在临时副本中使用已核对 SHA-256 的 0.7.0 编译器，重新执行配套工程的 `check`、`fmt --check`、`build --emit kicad_pcb`，三项均通过。构建结果为 11 个实例、8 条网络、5 行 BOM、10 个放置，和课程记录一致。命令、诊断、版本和教学源码哈希存于仓库 `docs/reviews/evidence/2026-09-12-book-syntax-repair.json`。

教学 `.cohdl` 文件本次仅改注释，可执行内容保持不变；环形、courtyard 和圆形开口的四个规则探针沿用上一轮已保存的结果。本轮没有重复执行它们，也没有实现 M2 或声明验证器。

`mdbook build book` 和 `python3 book/tools/check_book.py` 均通过，覆盖 72 章、75 个 HTML 页面及 3708 个本地链接、资源和锚点。`git diff --check` 通过；本记录在 SUMMARY 中恰好出现一次。

## 学习与研发边界

上述结果是代理的教材和编译验证，没有新增学习者复述、KiCad GUI 导入、实板操作、制造或测量结果。学习位置保持在第 1 课，见[当前进度](progress.md)。M2 仍以 RFC 为当前重点；语法比较与声明验证器仍是附录中的后续工作。
