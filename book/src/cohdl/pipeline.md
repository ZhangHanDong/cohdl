# 从 check 到实际 PCB

> **定位**：理解每个命令产生什么，以及检查结论到哪里为止。前置：[连接模型](connections.md)。当前能力；基线 CoHDL 0.6.0 / `9ac0e3c`，KiCad 10。

```text
CoHDL 源码 + 固定依赖
        ↓ check：解析、名称、类型、连接、已建模 DRC
        ↓ build：具体 part、位号与输出
      .net / BOM / .kicad_pcb
        ↓ 复制板到 pcb/，配置规则、放置、布线
   保存后的实际 PCB → KiCad DRC + 逻辑/物理对应核对
        ↓ 制造导出、平台适配
      收到实板 → 固件与仪器测试
```

## check 与 build

`check` 检查逻辑设计及已建模条件；`build` 还要完成适用的 pad 一致性、part 绑定、位号分配与输出。某个模型能 check，不代表已有可采购 part 和可制造 footprint。

下面用仓库现有样例说明 CLI。命令在仓库根目录运行；第一次使用时需要已安装编译工具链，以及 manifest 锁定的组件包。缺少包时按诊断执行 `install`，不要为了通过检查任意更新锁文件。

```sh
cargo run -- check examples/sf32-miniboard
cargo run -- build examples/sf32-miniboard --emit kicad_pcb
```

这些是样例使用命令；本书本次自动检查的是 [最小连接例子](connections.md)，未把 SF32 的实物功能记录为通过。

| 文件 | 职责 |
| --- | --- |
| `cohdl.toml` / `cohdl.lock` | 包版本要求与固定内容身份 |
| `design.lock` | 实例到位号的持久分配和历史记录 |
| `<name>.net` | 元件与 pin-to-net 连接 |
| `<name>-bom.csv` | 实际选定的 PCB 元件资料 |
| `<name>.kicad_pcb` | 带封装、网络和可用布局信息的板起点 |
| 配套 `.kicad_pro` / `.kicad_dru` | KiCad 项目、制造和物理检查规则 |

PCB 起点没有实际走线、铜区和制造规则。默认输出的两铜层配置也不是手表叠层已经选定；真正设计时按器件和工艺重新核对。

## 保护布线结果

```text
watch/
  src/       电路描述
  out/       可重新生成的输出
  pcb/       配套 KiCad 工程与实际布线板
```

在 `pcb/` 中布线。`out/.cohdl-manifest` 管理编译器生成的文件；在它拥有的路径里手动布线，后续 build 仍可能覆盖或清理那个文件。

源代码修改后，生成候选板，比较元件和网络差异，再把需要的变化应用到工作板并重检。现阶段不假设自动往返同步可以无损保留所有布线。

## 原生 demo 与 CoHDL 工程的区别

demo 有原生 `.kicad_sch`，可以使用对应的 ERC 与原理图/PCB 同步。CoHDL 当前不生成原生原理图文件；Explorer 是只读投影。

因此，自制 CoHDL 工程要独立核对最终板的位号、pad 和 net。Konnect 制造工具在缺少 schematic 时不自动产生其原理图 BOM，应使用 CoHDL BOM，并与最终 PCB 的装配坐标核对。

## 一次检查说明一件事

- CoHDL 通过：本次检查没有阻断错误；仍需阅读警告，结论仅覆盖已建模条件。比如 D003 是警告，可以在命令成功时出现。
- KiCad DRC 通过：被检查的实际板满足当前启用规则，仍要看未连接项、排除项和未覆盖条件。
- 实测通过：该板、该固件、该条件下测得的结果满足判据。

课程中有一个实际例子：Konnect 的 DRC 返回 `schematic_parity: 0`，但 GUI 明确显示“原理图一致性（不运行）”；核对该版本的 CLI 调用也未请求这项检查。因此一个空结果字段不能单独证明对应检查执行过。[铜连接与检查基线](../learning/2026-09-07-copper-and-checks.md) 同时记录输入文件、启用/忽略规则和执行范围。理解 CoHDL 检查或未来 M4 契约时也要保持这个区别：已检查且无问题、被忽略、未运行或缺少模型，不是同一种结论。

source、PCB 或规则变化后，重跑受影响检查。[第 2 课](../course/02-edit-check.md)用移动器件说明这种区别，[第 7 课](../course/07-production-test.md)将它延伸到制造与测量。

来源：[pipeline](https://github.com/conol-ai/cohdl/blob/9ac0e3c/src/pipeline.rs)、[part 绑定](https://github.com/conol-ai/cohdl/blob/9ac0e3c/src/emit/mod.rs)、[原生 PCB 合同](https://github.com/conol-ai/cohdl/blob/9ac0e3c/docs/kicad_pcb.md)。
