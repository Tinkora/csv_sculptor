# 贡献指南

[English](CONTRIBUTING.md)

## 开发环境

- Rust 1.95.0
- `wasm-pack` 0.15.0
- `wasm32-unknown-unknown` target
- Node.js 24 或更高版本，用于浏览器 smoke 测试

项目由平台无关的 `csv_sculptor_core` 和精简的 `csv_sculptor_web` WASM 边界组成。浏览器代码应调用 core，不得复制解析、转换或导出规则。

## Pull Request 流程

1. Fork 仓库。
2. 创建聚焦分支；Git 允许时，自有分支名使用下划线。
3. 改变行为前先编写面向结果且会失败的测试。
4. 运行本次变更涉及的 Rust、浏览器、文档和供应链检查。
5. 本仓库使用英文 Conventional Commit message。
6. 创建 PR，说明用户可见结果和实际运行的验证命令。

前端变更必须遵循 `AGENTS.md` 中的 `ui-ux-pro-max` 设计流程，并在 375、768、1024 和 1440 像素宽度检查。

不得提交生成的 `target/`、`pkg/`、`node_modules/`、Playwright 结果、私有输入、凭据或 secret。

## 行为准则

参与贡献即表示同意遵守[行为准则](CODE_OF_CONDUCT.zh-CN.md)。
