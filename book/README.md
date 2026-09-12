# CoHDL Book

这本书把 Konnect/KiCad 实操课程、CoHDL 当前能力、语言设计研究和真实学习记录分别组织。内容源文件在 `src/`，教材大纲见 [docs/book-outline.md](../docs/book-outline.md)。

从仓库根目录运行（本次验证使用 mdBook 0.5.3；链接检查需要 Python 3.9+）：

```sh
mdbook build book
python3 book/tools/check_book.py
mdbook serve book --hostname 127.0.0.1 --port 3001
```

打开 <http://127.0.0.1:3001>。生成目录 `book/html/` 已忽略，也可直接打开其中的 `index.html`。mdBook 的配置和命令以 [官方文档](https://rust-lang.github.io/mdBook/guide/creating.html) 为准；本书不需要额外预处理器或在线脚本。```mermaid 围栏由 `theme/mermaid-init.js` 在浏览器里渲染，它依赖同目录下本地打包的 `theme/mermaid.min.js`（mermaid 11.6.0，取自 mdbook-mermaid 0.17.0 自带的副本），通过 `book.toml` 的 `additional-js` 加载；CI 不需要安装 mdbook-mermaid。

书中的第一个完整 CoHDL 例子可单独检查：

```sh
cargo run -- check book/examples/connections.cohdl --no-std
```

它只验证连接模型，没有真实 part 和封装，不是制造示例。`--no-std` 仅用于这个自包含例子；真实项目按 manifest 锁定组件包。

Sonde 整板语言实验：

```sh
cargo run -- check book/examples/sonde/sonde.cohdl --no-std --json
python3 book/examples/sonde/verify.py
python3 book/examples/sonde/run_experiment.py
```

最后一条执行完整检查、期望失败的完整 build、临时反例和独立 RC 成功构建。全板有 25 个未绑定 part 的 E801；实验脚本验证这个真实边界，不把它改成生产构建成功。详细说明见 `book/examples/sonde/README.md`，正文为 [Sonde 实验](src/cohdl/sonde-case.md)。

2026-09-08 新增 [subdesign 两路 RC 实验](src/cohdl/subdesign.md)，源代码在 `examples/subdesign/`。该段保留当时的 main 实验基线与 worktree 复现说明。当前文档分支已经合入 main；重现旧报告仍应按提交与锁文件核对编译器，不能仅凭目录名判断版本。

本机当前预览使用 <http://127.0.0.1:3001/>（3000 由另一服务占用）：`mdbook serve book --hostname 127.0.0.1 --port 3001`。

[语法总览](src/cohdl/syntax/index.md)六节系统列出当前已实现的语法及其 PCB 对应；配套工程 `book/examples/syntax-tour/` 依赖锁定 std/passive/connectors，从仓库根目录运行：

```sh
cargo run -- check book/examples/syntax-tour
cargo run -- fmt book/examples/syntax-tour --check
cargo run -- build book/examples/syntax-tour --emit kicad_pcb
```

它覆盖九种顶层声明及常用体内语句，不是可投产的板。修改语法章节的例子后重跑这三条命令；`out/` 与 `design.lock` 是记录用的构建产物。

新增[十路 RC 实操](src/course/03-rc-workflow.md)与[诊断指南](src/cohdl/diagnostics.md)，当前语法验证基线为 main `0e3d770`。[课程编译器准备](src/appendix/course-compiler.md)使用由读者仓库路径推导的独立 worktree，实操课从复制电路开始；RC 验证器需要 Python 3.11+。记录证据时必须给验证器 `--compiler-source`，并先从该源码工作树构建。新版报告是 `docs/proposals/fixtures/m2-programmability/rc-workflow/book-results-2026-09-10.json`。

目录里的“课程提纲”描述教材体裁，不替代学习进度。Proposed RFC 与开源调研在“语言设计研究”；中文 RFC 全文与英文工作稿已于 2026-09-10 同步，沿用原阅读 URL。旧中文内容存入 `docs/proposals/archive/2026-09-10-m2-before-live-review.zh-CN.md`，不再作为当前全文展示；历史记录保留当时状态。

## 每次学习后如何更新

1. 在 `src/learning/` 新增一条实际记录，复制 [记录模板](src/learning/template.md)。按内容加入[工作台与电路实操](src/learning/workbench-circuits.md)、[语言设计与 RFC](src/learning/language-rfc.md)或[教材维护与验证](src/learning/book-maintenance.md)主题页，并链接到 `src/SUMMARY.md` 中同一主题的子目录；日期记录不再平铺到顶层。
2. 填写版本、问题、预测、操作、观察、用户复述和未检查项；没有执行的字段写“未执行”。
3. 更新 [进度表](src/learning/progress.md) 的学习状态。教材已写好不等于用户已掌握。
4. 把可复用的解释和错误案例补回对应课程/知识章节，并链接来源记录。
5. 遇到语言缺口，补到 [里程碑章节](src/cohdl/milestones.md)；提案只放在“未来能力”语境中。
6. 重跑构建、书内链接检查；修改可执行示例时重跑其编译检查。

章节链接使用书内相对 `.md` 路径；源码/规范引用使用有版本的 GitHub 链接，在资料页登记基线。个人目录只出现在本次学习环境说明中，公共教程优先使用读者自己的路径。

目录采用 mdBook 原生的[章节折叠配置](https://rust-lang.github.io/mdBook/format/configuration/renderers.html#outputhtmlfold)：`[output.html.fold] enable = true`、`level = 0`。新增主题只提供索引，保留已有记录的文件名与 URL；进度表和模板保持顶层入口。

课程仍在持续编写，当前学习者实际位置以进度表为准。后续章节的具体选型、测量值和截图随实操补充。发布到网站是独立步骤，当前提供本地阅读与 CI 构建。
