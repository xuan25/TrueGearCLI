# TrueGear-CLI

[English](README.md) | [简体中文](README_zh.md)

TrueGear-CLI 是一个第三方命令行界面（CLI）工具，用于通过蓝牙低功耗（BLE）与 TrueGear 设备进行通信。

它能够通过 WebSocket 协议接收并转发传入的命令。

## 快速开始

1. 从 [rustup.rs](https://rustup.rs/) 安装 Rust 和 Cargo。
2. 克隆此仓库：
   ```sh
   git clone https://github.com/xuan25/TrueGearCLI.git
   ```
3. 进入项目目录：
   ```sh
   cd TrueGearCLI
   ```
4. 使用 Cargo 构建项目：
   ```sh
   cargo build --release
   ```
5. 打开你的 TrueGear 设备，并确保电脑已启用蓝牙。
6. 运行 TrueGear-CLI：
   ```sh
   cargo run --release
   ```
   随后它会自动通过 BLE 连接到 TrueGear 设备。

   你可能会看到如下输出：
   ```sh
   Successfully connected to peripheral: "Name: Truegear_C*"
   Listening on: 127.0.0.1:18233
   ``` 
7. 连接到 `ws://127.0.0.1:18233/v1/tact/` 的 WebSocket 服务器，并发送 JSON 格式的效果指令。

   有关 WebSocket API 的更多细节，请参阅 [WebSocket Protocol](doc/websocket_protocol.md)。

## 命令行选项

你可以运行 `truegearcli --help` 来查看所有可用的命令行选项：

```
用法：truegear-cli [选项]

选项：
  -l, --listen-addr <LISTEN_ADDR>
          用于监听 WebSocket 连接的地址 [默认：127.0.0.1:18233]
  -e, --electical-effect-factor <ELECTICAL_EFFECT_FACTOR>
          电击效果强度系数（通常在 0.0 到 1.5 之间）[默认：1]
  -v, --verbose
          启用详细日志输出
  -h, --help
          打印帮助信息
  -V, --version
          打印版本信息
```
