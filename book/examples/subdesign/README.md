# RFC-032 两路 RC 教学实验

使用 **main `ab44257`** 的编译器和 `lib/`，不是本地旧文档分支的二进制。

```sh
python3 book/examples/subdesign/verify.py \
  --compiler /private/tmp/cohdl-main-subdesign-ab44257/target/debug/cohdl
```

可加 `--record book/examples/subdesign/results.json` 保存本次运行结果。
脚本验证 check/build、全部网表端点、5 个真实器件、3 行合并 BOM 及其分组、整组布局与单件覆盖、重复产物字节，以及五组失败用例和已有跨包 fn 能力。
临时反例不会改动教学源文件；正常 build 会更新本例 out/ 和锁文件。

依赖锁定 connectors 0.1.6、passive 0.2.2、std 0.3.0；无自造采购料号。
这是语言组合实验：有 net/BOM/封装及 layout.json，没有板框、布线、制造验收或滤波实测。
完整讲解见 `book/src/cohdl/subdesign.md`。网表读取复用相邻 Sonde 实验的独立 Python S 表达式解析器。
