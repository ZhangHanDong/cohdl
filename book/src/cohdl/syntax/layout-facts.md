# 布局事实：layout、place、约束、物理属性

> **定位**：CoHDL 不布局也不布线，但它记录“作者知道的物理事实”并交给板文件和布局工具。这一层的每条语句都遵守同一条原则：只要写得合法，它们不改变电气判定、位号和网表字节，也不新增四条以外的 DRC。它们自己仍要过检查：放置目标不存在是 E1007，属性单位写错是 E110，这些会让 `check` 失败。基线：CoHDL 0.7.0 / `0e3d770`。

## `layout { … }` 出现在哪里

`design` 和 `subdesign` 体内各可以有一个 `layout` 块。`design` 里的坐标是板坐标；`subdesign` 里的是相对本单元原点的默认位置，外层整体摆放时一起平移、旋转、翻面。`fn` 体内的 `layout` 只允许约束语句，写 `place` 是 E1007。

本例的布局块：

```cohdl
    layout {
        net_class Supply { V3V3, GND }
        place j at (0mm, 0mm) rotate 90
        place leds[0] at (10mm, 2mm)
        place leds[1] at (10mm, 6mm) side bottom
        place led_status at (10mm, 10mm)
        place r_pwr[0] at (14mm, 2mm)
        place r_pwr[1] at (14mm, 6mm) side bottom
        place c_byp at (4mm, 4mm)
        place hole at (20mm, 0mm)
        place lpf at (5mm, 10mm)
        place lpf.c at (9mm, 11mm)
    }
```

`fmt` 把约束语句放在放置语句前面，这是规范形式。

## `place`

`place 目标 at (x, y) [rotate 角度] [side top|bottom]`

- 目标是本设计的顶层实例、数组元素 `leds[1]`、subdesign 使用点 `lpf`，或穿过 subdesign 的点路径 `lpf.c`、`bank.channels[1].c`。
- `rotate` 接受 0 到 359 的任意整数度，`360` 及以上是错误。这是 RFC-020 之后由板级作者定下的偏离，规范文本仍写着四个基点角度。
- `side` 缺省为 `top`。翻面时封装只写一份，镜像由输出端完成。
- 同一目标放两次是 E1007。目标不存在也是 E1007，路径中某段找不到是 E1305。

**覆盖规则**：外层对 `lpf.c` 的显式放置胜过 subdesign 自己 `layout` 里给 `c` 的默认位置，只对这一个使用点生效。本例 `lpf` 放在 (5mm, 10mm)，`c` 的默认位置是 (3mm, 0mm)，合成后应为 (8mm, 10mm)；`layout.json` 里它实际是 (9mm, 11mm)，因为被覆盖了，而 `lpf::r` 保持 (5mm, 10mm)。

**PCB 对应**：`place` 就是 KiCad 板文件里封装的位置、角度和层。`build --emit kicad_pcb` 把这些原样写进 `.kicad_pcb`，没有放置的元件排在板外的暂存区。编译器不检查元件是否重叠、是否超出板框。

## `board_outline`

```cohdl
        board_outline: "mechanical/joint-mc-outline.dxf"
```

来自仓库 [`examples/joint-motor-controller`](https://github.com/conol-ai/cohdl/blob/0e3d770/examples/joint-motor-controller/src/main.cohdl)。路径相对项目根，每个设计至多一条，只能在 `design` 的 `layout` 里。`build` 时打开这个 DXF，取约定图层上的一条闭合折线作为板框，写进 IPC-2581 的 Profile 和 `.kicad_pcb` 的 Edge.Cuts。缺文件、折线不闭合、无法解析都是 E1006。CoHDL 不是 DXF 解析器，其他图层和实体一概不读。

**PCB 对应**：板框来自机械工程师的 CAD 文件，不在 CoHDL 里画。

## 三种布局约束

```cohdl
        net_class HighSpeed { USB_DPX, USB_DMX }
        diff_pair(USB_DPX, USB_DMX) [differential_impedance: 90ohm, frequency: 480MHz]
        length_match(USB_DPX, USB_DMX) [tolerance: 0.15mm]
```

形式来自 [`examples/rpi-pico2`](https://github.com/conol-ai/cohdl/blob/0e3d770/examples/rpi-pico2/src/main.cohdl)（数值为示意）。`net_class` 给一组网络起名；`diff_pair` 恰好两条网络，可选方括号里是阻抗和频率；`length_match` 两条以上网络，可选公差。引用的网络必须已经存在，重名、数量不对、先用后声明都是 E10xx 错误。

**PCB 对应**：差分对、等长、网络类是布线工具的输入。CoHDL 不验证公差是否达到、差分对是否真的被成对布线；它没有几何可比。

## 物理属性

七个 `#[…]` 属性挂在 `net` 或 `inst` 上，见[设计与网络](design-nets.md#属性给工具的事实)的表。它们对应 Quilter 文档里的物理约束字段，`build` 在设计含有任何一条时输出八个 CSV：`ground_nets.csv`、`high_current_nets.csv`、`single_ended_impedance_signals.csv`、`differential_pairs.csv`、`bypass_capacitors.csv`、`crystal_oscillators.csv`、`switching_converters.csv`、`bga_components.csv`。没有物理事实的设计不输出这组文件。

`#[bypass(j.P4, 100nF)]` 挂在电容自己的 `inst` 上，第一个参数是被去耦的引脚。写在 `fn` 体内时，第一个参数可以是 `fn` 的 `Pin` 参数名，每个调用点各得一行 CSV。

## 这一层编译器不检查什么

不检查间距、重叠、走线、回流路径、热和 EMC。`layout.json`、`.kicad_pcb`、IPC-2581 和 CSV 都是给下游工具的输入；下游工具的 DRC 才是几何检查发生的地方，见[从 check 到实际 PCB](../pipeline.md)。
