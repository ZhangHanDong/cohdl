# Sonde：整板连接模型与真实编译对照

`sonde.cohdl` **完整描述原板的电气连接和已知元件值，`check` 通过；整板 `build` 失败，25 个 E801**。原图没有足够的制造商、采购型号、容差、额定值和可解析封装绑定，不能凭 `74LS125` / `BAT46` 等标注编造一份采购 BOM。这里没有绕过 build，也没有输出“整板可投产”的网表。

`rc-control/` 是**另外命名的两元件 0603 RC 物料绑定实验**，实际 `build` 成功。它不是原板上的两个直插元件，不是整个 Sonde，也没有缓冲器、接口和供电源；INPUT 与 GND 是单端被动网络。它仅证明已知真实料号与封装可以通过正常生产输出路径生成网表和 BOM。

## 重现

从仓库根目录运行（Rust/Cargo、Python 3；脚本无额外 Python 包；首次需要已缓存编译器的 Cargo 依赖）：

```sh
cargo run --quiet -- check book/examples/sonde/sonde.cohdl --no-std --json
cargo run --quiet -- build book/examples/sonde/sonde.cohdl --no-std --json
python3 book/examples/sonde/verify.py
cargo run --quiet -- build book/examples/sonde/rc-control --json
python3 book/examples/sonde/run_experiment.py
```

第二条**预期退出 1 / E801 × 25**，其余预期退出 0。完整实验脚本会运行 `cargo build --offline --locked`、两项 `fmt --check`、正常 check/build、下面的变异实验、独立网表比对和真实 RC 产物验证。它接受 `--record PATH` 将本次结果写入指定文件；默认只打印。`results.json` 是本次真实运行的精简记录，含源文件/参考网表哈希、命令、退出码、全部诊断代码计数和每种诊断的实例。临时变异文件生成于系统临时目录，记录里的 `<probe>` 对应脚本里命名的变异。

整板使用 `--no-std` 加本地最小 trait：只声明原图确实提供的 resistance/capacitance，没有为满足标准库完整采购规格而猜测容差或耐压。RC 工程正常锁定 `passive = 0.2.1`、`std = 0.3.0`，`cohdl.lock` 的实际内容哈希由编译器生成并验证，不使用开发覆盖路径。

## 独立比对如何工作

真值是本次 Konnect 导出的 [`sonde.net`](../../src/learning/evidence/2026-09-08-sonde-full/sonde.net)；第二份交叉证据是同目录 `konnect.json` 中的 25 次 `get_component_pads`。不解析或编辑 KiCad 原始文件。

`inspect/` 是单独的极小 Rust 工具，只通过公开 `check_files_in_with_deps` 检查**这一个无依赖源码文件**；若存在错误或设计选择失败即退出。它从 checked IR、已解析 device pin mapping 及显式 `#[designator]` 提取制表符记录，**不调用物料绑定、设计位号分配器或生产 emitter**。其记录不是 KiCad 产物，也不是通用依赖项目加载器。

`verify.py` 独立解析原始导出 S-expression，与 IR 的物理端点集合比较：

- 25 个实例，保持原位号（原图没有 R3）；逐个比较器件种类与数值，R2 的 `5,1K` 正规化为 `5.1kohm`。
- 26 个连接网络分区、92 个已连接端点、16 个 NC 端点，合计 108 个电气引脚；网络名改变不影响比较。
- 每个物理引脚角色与导出对应，唯一显式抽象是 `tri_state → output`；对照 108 个 Konnect PCB 焊盘的编号及原网名。
- 接线变异交换 J1.2/J1.3：编译器仍通过，但独立验证器拒绝，说明它能发现真实拓扑错误。

脚本检查值和连接，没有审核 footprint 尺寸、板层、走线、时序或电压兼容性。原 footprint 标识和坐标仍在导出证据中；这个电气模型没有重建几何，不能据它恢复已布线 PCB。

## 复用与明确的抽象边界

U1 和 U2 各为一颗完整的 14 脚四路器件，包含共用电源引脚。`buffered_path` 复用三条输出通道，把已有的整颗电阻/电容实例和芯片引脚作为参数；不会把每个门生成成一颗芯片。`ResistanceElement` / `CapacitanceElement` 为参数区分 R 与 C，继承 `TwoTerminal` 的 A/B 引脚；这属于现有 trait 的库接口设计。

引脚 E1…E4 的低有效含义、U1A 的低电平/高阻切换以及 R2 的上拉仅由说明解释；`[output]` 是对三态引脚的保守结构抽象。不能据此证明共享总线的互斥驱动。连接器按原导出标为 passive，不假装外部主机/目标电路已经建模。GND 显式标注 `[gnd]`，C1 和二极管实现 `Polarized`；D001 没有足够的额定值与网络电压可检查。

`TARGET_POWER`、D2 后的 `VCC`、D1/R1 电源感知链都不写猜测的固定电压。CoHDL 不会求解二极管压降、R1 压降、负载电流和断电反灌，因此原图 `3.3–5V` 标注不能变成兼容性证明。

## 本次执行的探针

| 变异 | 实际结果 | 能说明什么 |
| --- | --- | --- |
| R2 的 `5.1kohm` 改成 `5.1nF` | 退出 1，E112 | 泛型单位类型确实检查 |
| 移除 J1.1 的 NC | 退出 1，E701 | required 引脚穷尽性 |
| U2.O2 同时在 net 和 nc | 退出 1，E702 | 矛盾连接被拒绝 |
| 合并 U1.O2、U1.O3 | 退出 1，D004 | 同网两个 output 被拒绝；未模拟互斥使能 |
| 交换两条通道的 J1 端点 | check 退出 0，独立验证拒绝 | 语言合法性不等于符合参考设计 |
| 合成 `supply_min: 4V`，网络 `[1V]` | 退出 0 | 自定义下限字段不自动成为约束 |
| 合成 `voltage_rating: 5V`，网络 `[6V]` | 退出 1，D001 × 2 | 对照验证已有上限检查实际工作 |
| 将 `[output]` 写为 `[tri_state]` | 退出 1，E010 | 原生角色词表尚无三态 |

表中 4V/5V/1V/6V **全部是探针合成值，不是 74LS125 的数据手册参数或实测值**。目前结果支持研究“工作电压上下界”“三态使能/高阻契约”“带未知量与来源的供电约束传播”，这些仍是语言设计建议，未实现新语法，也未增加第五条结构 DRC。

## 成功的真实物料输出

`rc-control/out/`（git 忽略）实际生成 `sonde-rc-control.net`、`sonde-rc-control-bom.csv` 和 `footprints/passive-CHIP_0603.kicad_mod`。脚本独立核对真实输出的三组网络端点、元件值和两个 BOM 料号，并重复 build 比较全部输出字节。

| 对照位号 | 库 part | 主选制造商 / MPN |
| --- | --- | --- |
| R1 | `passive::R_100R_F_0603` | Yageo / `RC0603FR-07100RL` |
| C1 | `passive::C_100p_50V_C0G_0603` | Yageo / `CC0603JRNPO9BN101` |

来源为仓库已存在的 `lib/passive/src/resistors_0603.cohdl` 和 `capacitors_0603.cohdl`（各自带 Yageo 数据手册引用）。此实验没有做新的库存查询或元件采购认证。
