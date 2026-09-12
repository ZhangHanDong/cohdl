# 2026-09-07：重启后的 MCP 与 IPC 核对

对应章节：[第 0 课](../course/00-workbench.md)、[第 1 课](../course/01-read-board.md)。状态：环境核对进行中；尚未读取当前 PCB 的板信息、器件清单和网络清单。

## 本次请求与执行

用户要求通过 Konnect 核对版本、发现工具集、检查 GUI/IPC 和练习副本，再读取板、器件、网络。此前会话没有暴露 Konnect 工具；用户重启 Codex 后，本次确实可以调用基础 MCP 工具。

实际依次调用了 `get_installation_info` 和 `list_toolboxes`，随后用 `load_toolset` 加载 `verification`、`pcb_board`、`pcb_components`、`pcb_routing`；另调用 `load_user_config`、`open_project`、`get_project_info`、`get_active_toolsets`。使用只读进程检查、KiCad 保存的公共设置和 GUI 窗口补充诊断。

## 实际证据

| 检查 | 本次结果 | 能证明什么 |
| --- | --- | --- |
| 服务版本 | Konnect 0.11.0，commit `1f96aad76cd4bcf5a0361ce22130f9a37efa0363`，release / aarch64 / macOS | 当前调用由这个版本的进程处理 |
| 服务路径 | `/Users/zhangalex/Work/CoHDL/Konnect/target/release/konnect` | 已使用更名后的路径 |
| KiCad CLI | `probe_status: ok`，10.0.6；路径为已配置的应用包内 CLI | 命令行程序可运行；不证明 IPC 已连通 |
| 工具目录 | 20 个工具集、221 个工具；加载后 6 个工具集、70 个活动工具 | 服务器已加载请求的工具集；统计不含始终可见的路由元工具 |
| 客户端工具入口 | 多轮发现仍只有基础 Konnect 工具；`check_kicad_ui`、`get_board_info` 等没有可调用入口 | 本轮新增工具尚未被客户端暴露 |
| Konnect IPC | `configured: false`；`open_project` 返回 `ipc_available: false`、地址为空、`requested_open: null` | 未能通过 IPC 判断目标 PCB 是否打开；空的 `open_boards` 不是成功查询的空板列表 |
| KiCad GUI | 进程存在；实际窗口为 `KiCad 10.0` 项目管理器，左侧工程文件列表为空 | 当前看到的管理器没有加载练习工程；不能把 IPC 不通解释成程序没运行 |
| 保存的 API 设置 | `~/Library/Preferences/kicad/10.0/kicad_common.json` 中 `api.enable_server: false` | 保存的设置尚未启用 API；本次没有修改它 |
| 练习副本 | `get_project_info` 确认 `.kicad_pro`、`.kicad_sch`、`.kicad_pcb` 均存在 | 文件位于目标目录；不证明文件已在编辑器中打开 |
| 工程版本字段 | project-file format 3；原理图和 PCB generator 均为 9.0 | 保存文件的格式/生成器元数据，与运行中的 CLI 版本不同 |

目标工程为：

```text
/Users/zhangalex/Work/CoHDL/watch-lab/lessons/01-sonde/sonde xilinx.kicad_pro
```

`load_user_config` 读的是用户偏好 `config.json`，与保存 `kicad_cli`、`kicad_binary`、`ipc_address` 等服务器启动参数的 `config.toml` 是两个文件。本次没有把默认制造偏好当作这块 demo 的实际板层或设计规则。

## 原因与尚未执行的修复

本地 Konnect `crates/konnect/src/config.rs` 明确提供 `eager_toolsets`：针对缓存初始工具目录、未采用 `notifications/tools/list_changed` 的客户端，在启动时加载全部工具。本轮现象与这一兼容情形相符；尚未修改配置并重启验证，所以不能记录为已修复。

另一个独立问题是 IPC：保存的 KiCad API 设置关闭，服务地址为空。需要打开练习工程的 PCB 编辑器，启用 API，读取本机实际监听端点，再按启动方式把端点交给 Konnect。不要猜测 socket 路径。把必要配置准备好后再重启客户端，可减少反复重启。

还核对了 `crates/konnect-core/src/tools/project.rs`：0.11.0 的 `open_project.kicad_ui_running` 直接使用 IPC ping 的布尔结果。因此本次返回 `false` 与实际 GUI 运行并不矛盾。若扩展工具可调用，应使用 `check_kicad_ui` 区分进程检测和 IPC 响应；本次未调用成功，不能编造该工具结果。

## 学习与 CoHDL 记录

第 0 课补入工具发现与 IPC 的分层排查；第 1 课补入板信息、器件清单、网络清单的含义。没有把先前对原始 demo 的网表分析冒充本次练习板的实时数据。

本次未编辑或保存 KiCad 工程，未执行 ERC/DRC，也没有硬件测量。用户尚未复述连接模型，学习进度仍为环境核对中。问题属于工具接通和课程前置，没有据此新增 CoHDL M1–M4 的语言需求。
