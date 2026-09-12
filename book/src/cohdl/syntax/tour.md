# 走一遍：syntax-tour 的源码与产物

> **定位**：把前几节的片段合起来，从 `check` 走到 `build`，逐个看产物。工程在仓库 `book/examples/syntax-tour/`，依赖锁定 `std 0.3.0`、`passive 0.2.2`、`connectors 0.1.6`。基线：CoHDL 0.7.0 / `0e3d770`。它不是一块可投产的板：连接器上挂三颗 LED、一路 RC 和一个定位孔，只为覆盖语法。

## 文件

| 文件 | 内容 | 对应章节 |
| --- | --- | --- |
| `cohdl.toml` | 包名、顶层设计名、精确版本依赖 | [复用](composition.md#模块名字从哪里来) |
| `cohdl.lock` | 每个依赖的版本和内容哈希 | 同上 |
| `src/led.cohdl` | `trait`、`device`、`impl`、`pad`、`footprint`、`part` | [单位与器件](units-devices.md)、[物料与封装](parts-footprints.md) |
| `src/mech.cohdl` | 无引脚器件与定位孔封装 | [物料与封装](parts-footprints.md#没有引脚的器件) |
| `src/filter.cohdl` | `subdesign RcLowPass`、`fn drive_led` | [复用](composition.md) |
| `src/main.cohdl` | `design SyntaxTour` | [设计与网络](design-nets.md)、[布局事实](layout-facts.md) |

## 运行

```sh
cargo run -- check book/examples/syntax-tour
cargo run -- fmt book/examples/syntax-tour --check
cargo run -- build book/examples/syntax-tour --emit kicad_pcb
```

`check` 无诊断；`fmt --check` 报告全部文件已是规范形式；`build` 输出：

```text
Built design `SyntaxTour`: 11 instances, 8 nets
```

先自己数一遍再对照。实例：1 个插座、3 颗 LED、2 颗电源指示灯电阻、`fn` 展开的 1 颗电阻、1 颗去耦电容、RC 里的 1 电阻 1 电容、1 个定位孔，共 11。网络：顶层写了 4 条，`fn` 里 3 条匿名网，RC 里 3 条匿名网，共 10 条声明；其中 `fn` 的 `gnd` 网和 RC 的 `GND` 端口都并入顶层 `GND`，减 2；`IN`、`OUT` 接到的 `j.P1`、`j.P2` 没有别的连接，各自仍是独立节点。10 减 2 得 8。

## 产物

`out/` 目录：

| 文件 | 是什么 | 看什么 |
| --- | --- | --- |
| `syntax-tour.net` | KiCad 网表 | 每个网络下的焊盘端点；`lpf` 不出现，只有 `lpf::r`、`lpf::c` |
| `syntax-tour-bom.csv` | BOM | 5 行，按 MPN 合并：`R1,R2,R3,R4` 一行，`D1,D2,D3` 一行，`M1` 独占一行 |
| `syntax-tour-layout.json` | 放置与约束 | 10 条放置；`lpf::c` 在 (9, 11)，`lpf::r` 在 (5, 10) |
| `syntax-tour.kicad_pcb` | KiCad 10 板文件 | 可直接用 pcbnew 打开；`leds_1`、`r_pwr_1` 在 B.Cu |
| `footprints/*.kicad_mod` | 用到的封装 | 本地 LED 封装带阴极带丝印 |
| 八个 `*.csv` | Quilter 物理约束 | 因为设计里有 `#[ground]`、`#[high_current]`、`#[bypass]` |

`design.lock` 在项目根目录，不在 `out/`：

```toml
[designators]
"SyntaxTour::__fn0_drive_led::r" = "R1"
"SyntaxTour::c_byp" = "C1"
"SyntaxTour::hole" = "M1"
"SyntaxTour::j" = "J1"
"SyntaxTour::led_status" = "D1"
"SyntaxTour::leds_0" = "D2"
"SyntaxTour::leds_1" = "D3"
"SyntaxTour::lpf::c" = "C2"
"SyntaxTour::lpf::r" = "R2"
"SyntaxTour::r_pwr_0" = "R3"
"SyntaxTour::r_pwr_1" = "R4"
```

四种路径形态并排出现：普通实例 `c_byp`，数组元素 `leds_0`，subdesign 内部 `lpf::c`，`fn` 展开 `__fn0_drive_led::r`。`J1` 来自 `#[designator("J1")]`，其余由前缀加序号自动分配。

## 故意改错

每次只改一处，改完 `check`，然后恢复。这些是本工程在 0.7.0 上的实际输出。

| 改动 | 结果 |
| --- | --- |
| 删掉 `nc: j.P6` | E701，`SyntaxTour::j.P6` 未处理 |
| `drive_led(led_status, …)` 改成 `drive_led(leds[0], …)` | E202，`unknown instance leds`，数组元素目前不能作为实例参数；另有三条 E701，因为 `j.P3` 和 `led_status` 的引脚随之无人接线 |
| `net V3V3 [3.3V]` 改成 `net V3V3 [3.3]` | E010，解析错误：`expected a voltage literal (e.g. 3.3V) or gnd`；解析器恢复后还会跟着几条 E010 和 E701，先修第一条 |
| `place leds[1]` 改成 `place leds[2]` | E202，越界 |
| 删掉 `impl Polarized for ChipLed {}` | E302，`impl Indicator for ChipLed requires impl Polarized for ChipLed`，随后 `led_status` 两个引脚各一条 E701 |

## 它没有证明什么

三颗 LED 的限流电阻是否合适、RC 的截止频率、定位孔的位置，编译器一概不知道。`build` 通过只说明：每个必需引脚都被交代过，每个实例都有物料，每个封装用到的焊盘编号集合和器件引脚号集合一致，四条残余 DRC 没有触发。
