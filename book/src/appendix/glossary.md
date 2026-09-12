# 术语表

| 术语 | 在本书中的意思 | 从哪里实践 |
| --- | --- | --- |
| PCB | 印制电路板，承载铜连接和安装元件的物理板 | [第 1 课](../course/01-read-board.md) |
| Schematic | 原理图，表达元件和逻辑连接 | [第 1 课](../course/01-read-board.md) |
| Symbol | 原理图里代表元件的简化图形，带有用于表达电气连接的引脚 | [第 1 课](../course/01-read-board.md#先认识五个对象) |
| Pin / Pad | 逻辑引脚 / 封装物理焊盘，可能存在一对多映射 | [连接模型](../cohdl/connections.md) |
| Footprint | 器件在板上的焊盘及配套几何定义 | [第 1 课](../course/01-read-board.md) |
| Net | 应相连的端点集合 | [第 1 课](../course/01-read-board.md) |
| Trace / Via | 铜走线 / 连接铜层的导电过孔 | [第 2 课](../course/02-edit-check.md) |
| Ratsnest | 飞线，提示尚需实现的连接关系 | [第 2 课](../course/02-edit-check.md) |
| Reference designator | 位号，例如 R1、U2，用来标识实际元件 | [构建流程](../cohdl/pipeline.md) |
| ERC / DRC | 电气规则检查 / 设计规则检查；须说明使用哪个工具及检查对象 | [构建流程](../cohdl/pipeline.md) |
| BOM / CPL | 物料清单 / 元件贴装坐标等信息，不同平台格式可能不同 | [第 7 课](../course/07-production-test.md) |
| Gerber / Drill | 制造层图形 / 钻孔数据 | [第 7 课](../course/07-production-test.md) |
| PCBA | 已装配元件的 PCB | [第 7 课](../course/07-production-test.md) |
| MCU / BLE | 微控制器 / 低功耗蓝牙 | [第 5 课](../course/05-watch-prototype.md) |
| IMU | 惯性测量器件，本课从运动数据读取开始 | [第 5 课](../course/05-watch-prototype.md) |
| FPC | 柔性印制电路，常用于显示连接；需核对配对连接器和方向 | [第 5 课](../course/05-watch-prototype.md) |
| LDO / DC/DC | 低压差线性稳压器 / 直流电源转换电路，适用条件不同 | [第 6 课](../course/06-power-contracts.md) |
| Decoupling | 去耦，要同时考虑器件要求、连接与实际布局 | [复用章节](../cohdl/reuse.md) |
| MCP / IPC | 客户端调用工具的协议 / Konnect 与 KiCad 实时进程通信的接口 | [第 0 课](../course/00-workbench.md) |
| Module / fn / array | 源文件组织 / 复用展开 / 多个独立实例的集合 | [复用章节](../cohdl/reuse.md) |
| Contract | 带来源和条件的可检查约束；通用器件契约属于后续里程碑 | [M4](../cohdl/milestones.md) |

术语在这里集中维护。每次学习发现一个新的英文词，在这里加入简明定义和实际使用章节，而不是只列翻译。
