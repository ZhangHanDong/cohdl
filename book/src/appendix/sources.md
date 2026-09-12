# 资料与版本

本书基础内容基线核对于 2026-09-07，subdesign 专章于 2026-09-08 单独跟进 main。实现状态来自本地源码，历史环境检查与本次教材验证分开记录。版本更新时，先重跑相关例子和操作，再修改“当前能力”的描述。

## 本地基线

| 对象 | 基线 | 用途 |
| --- | --- | --- |
| CoHDL 原基线 | 0.6.0，`9ac0e3c2ec7731eb636cb33b9c9a781e1f144242` | Sonde 等原有章节、CLI、库与 PCB 输出 |
| CoHDL subdesign 基线 | 0.6.0，main `ab44257` | RFC-032、两路 RC 实验、M1 状态更新；没有据此重标旧 Sonde 测试 |
| CoHDL main 依据复核 | 0.7.0，main `0e3d770`，2026-09-10 pull | RFC-032 仍为 Accepted；规范与 subdesign 实现未变，24 项专项测试再次通过，详见[校正记录](../learning/2026-09-10-main-rfc032.md) |
| CoHDL 语法总览基线 | 0.7.0，main `0e3d770`，2026-09-12 | [语法总览](../cohdl/syntax/index.md)六节；`book/examples/syntax-tour/` 用该版本通过 `check`、`fmt --check`、`build --emit kicad_pcb`，五个故意改错探针的结果记录在[走一遍](../cohdl/syntax/tour.md) |
| Konnect | 0.11.0，`1f96aad` | 2026-09-08 临时标准 MCP 的 IPC 已验证；Codex 内置连接重载另记 |
| KiCad CLI | 10.0.6 | 2026-09-08 重新执行网表、原理图导出与 ERC/DRC；GUI 操作另行记录 |
| mdBook | 本机 0.5.3 | 本书构建与预览 |

## 原始资料

- [RFC-032 subdesign](https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/rfc-032-subdesign.md)、[专项测试](https://github.com/conol-ai/cohdl/blob/ab44257/tests/subdesign.rs)及[实际实验记录](../learning/2026-09-08-subdesign.md)：新功能的规范、代码与验证边界。
- [CoHDL 语言规范](https://github.com/conol-ai/cohdl/blob/9ac0e3c/docs/design/10-language-specification.md)与[合规/偏离记录](https://github.com/conol-ai/cohdl/blob/9ac0e3c/docs/compliance-report.md)：语言事实的依据。
- [CoHDL 库说明](https://github.com/conol-ai/cohdl/blob/9ac0e3c/lib/README.md)：组件包和 namespace。
- [原生 PCB 输出合同](https://github.com/conol-ai/cohdl/blob/9ac0e3c/docs/kicad_pcb.md)：输出边界、坐标和所有权。
- [SF32 样例](https://github.com/conol-ai/cohdl/blob/9ac0e3c/examples/sf32-miniboard/README.md)：显示、电平转换、电源、时钟及未验证项。
- [Konnect README](https://github.com/mixelpixx/Konnect/blob/1f96aad/README.md)与[工具目录](https://github.com/mixelpixx/Konnect/blob/1f96aad/tool-directory.md)：安装、工具发现与能力。运行时再以实际 schema 核对参数。
- [KiCad 10 官方入门](https://docs.kicad.org/10.0/en/getting_started_in_kicad/getting_started_in_kicad.html)：原生 KiCad 操作与工作流。
- [mdBook 官方文档](https://rust-lang.github.io/mdBook/guide/creating.html)：构建、目录和本地预览；命令参数可用本机 `mdbook --help` 复查。

## Demo 来源与复制

本次本地目录是 `/Users/zhangalex/Work/CoHDL/kicad-demos`，使用 sonde xilinx、multichannel、royalblue54L_feather 和 LM317 仿真例子。完整项目副本保留原始文件和随附许可证；本书未打包完整第三方 demo，学习证据包含 Sonde 副本的导出图、网表与查询结果，来源和哈希见相应记录。

## Sonde 电气资料

- [TI SN74LS125A 数据手册](https://www.ti.com/lit/ds/symlink/sn74ls125a.pdf)：第 1–2 页为引脚和使能功能，第 6 页为商用 LS 器件的推荐工作条件。用于核对本地 `74LS125` 标注的含义，不据此确认原采购厂商。
- [TI SN74HC125 数据手册](https://www.ti.com/lit/ds/symlink/sn74hc125.pdf)：对照不同逻辑家族的供电和输入条件，不能作为未经验证的替换结论。
- [Vishay BAT46 数据手册](https://www.vishay.com/docs/85662/bat46.pdf)：DO-35 小信号肖特基器件家族；引脚与封装在具体采购前仍需逐项匹配。
- [本次整板证据](../learning/2026-09-08-sonde-language.md)：当前练习副本路径、源文件哈希、工具版本、连接核对和实际语言实验。

首次实际练习时在记录中加入 demo 的 commit（若有）或文件 hash，避免修改后仍引用旧网表。后续硬件选型另记厂家文档版本、页码和具体条件；材料尚未取得时保留未验证。

目前已有的观察见 [课程起点](../learning/2026-09-07-start.md)。源码引用使用固定 commit；课程中的未来能力明确标注，不由链接到某个提案就视为已经接受。
