# AI 中转额度消耗检测

本项目是一个本地优先的 Tauri v2 桌面应用，用来检测 AI 中转平台的接口 `usage`、理论消耗和平台实际扣费之间是否存在明显偏差。

## 当前功能

- OpenAI 兼容 `/v1/chat/completions` 探测。
- 串行测试 runner，支持目标金额、最大请求数、连续失败暂停和预算保护停止。
- 实时进度事件、累计 tokens、理论消耗、失败率和请求日志。
- 测试前余额 / 测试后余额手动录入，计算实际扣除、偏差金额和偏差比例。
- 本地报告保存，并支持 JSON、Markdown、CSV 导出。
- API Key 仅用于本次请求，不写入报告或普通本地数据。
- Base URL 安全校验，默认禁止请求内网、localhost 生产地址和 metadata 地址。

## 非桌面启动验证

```bash
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

## 开发启动

需要打开桌面端时再运行：

```bash
npm run tauri dev
```

## 主要目录

- `src/`：React + TypeScript 前端。
- `src-tauri/src/core/`：请求、计费、安全校验和 runner。
- `src-tauri/src/commands/`：Tauri command 边界。
- `src-tauri/src/storage/`：本地报告保存和导出。
