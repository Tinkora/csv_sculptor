# CSV Sculptor

CSV Sculptor 是一个浏览器原生的 CSV/TSV 工作台，用于查看、筛选、排序和转换表格文本，输入不会上传。Rust 负责数据行为，精简的 WebAssembly 边界将其提供给浏览器。

[English](README.md)

## 成熟度

**Alpha。** 托管质量和发布工作流已通过，浏览器工作台已部署到 GitHub Pages，发布包包含 checksum、SPDX SBOM、许可证清单和构建证明。

- **在线体验：** [GitHub Pages](https://tinkora.github.io/csv_sculptor/)
- **最新候选版本：** [v0.1.0-alpha.2 Release](https://github.com/Tinkora/csv_sculptor/releases/tag/v0.1.0-alpha.2)

- **本地人类界面：** 已实现，并由托管 Chromium smoke 测试覆盖。
- **Agent schema 草案：** `skills/mcp-tools.json` 记录了可能的工具结构。
- **尚不可由 Agent 调用：** 未提供 MCP server、托管端点、身份验证或工具注册。

## 当前范围

- 解析最大 10 MiB 的 UTF-8 CSV、TSV、竖线和分号分隔文本，包括带引号字段。
- 拒绝空白或重复表头以及列数不一致的行。
- 使用九种运算符筛选，并以 AND 语义组合所有启用的筛选条件。
- 数字列按数值排序，其他列按不区分大小写的文本排序。
- 从当前结果中选择列、限制行数并移除重复行。
- 导出 JSON、YAML、Markdown 表格、SQL `INSERT`、CSV 或 TSV。
- 所有导入数据仅保留在当前浏览器标签页中。
- 工作台支持英文和简体中文切换。

列选择和行数限制同时作用于浏览器预览及所有导出格式；重置会恢复完整的导入表格。

## 安全边界

- 输入按 UTF-8 解码，应用不会把输入发送到服务器。
- 预览使用 `textContent` 而不是 HTML 渲染单元格内容。
- CSV/TSV 导出保留原始单元格值。导出对话框会提示以 `=`、`+`、`-` 或 `@` 开头的单元格可能被电子表格软件解释为公式，但不会修改原始值；使用此类软件打开不可信导出前必须检查内容。
- SQL 输出会引用标识符和字符串，但它是生成的文本，不是数据库迁移；应按目标数据库方言复核。
- 浏览器最多预览 500 行；转换和导出仍针对内存中的完整表格。

## 开发

环境要求：

- Rust 1.95.0，并安装 `wasm32-unknown-unknown` target
- `wasm-pack` 0.15.0
- Node.js 24 或更高版本，用于浏览器 smoke 测试

在仓库根目录运行 Rust 检查：

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check -p csv_sculptor_web --target wasm32-unknown-unknown --locked
```

构建真实 WebAssembly package 并运行工作台：

```bash
cd crates/csv_sculptor_web
wasm-pack build --target web --out-dir static/pkg .
npm ci --ignore-scripts
npm run serve
```

打开 `http://127.0.0.1:4173/static/`。

在 375、768、1024 和 1440 像素宽度运行浏览器 smoke 测试：

```bash
cd crates/csv_sculptor_web
npm run test:wasm-smoke
```

运行文档和供应链检查：

```bash
ruby scripts/test_check_docs.rb
ruby scripts/check_docs.rb
cargo deny check advisories bans licenses sources
cargo audit --no-yanked
```

`cargo deny` 负责已撤回 package 的门禁；`cargo audit --no-yanked` 独立执行
RustSec advisory 扫描，避免重复请求 registry API。

生成的 `target/`、`pkg/`、`node_modules/`、Playwright 结果和浏览器产物均已忽略，不得提交。

## 目录结构

| 路径 | 职责 |
| --- | --- |
| `crates/csv_sculptor_core` | 解析、转换、导出和稳定错误 |
| `crates/csv_sculptor_web` | 精简 WASM 边界和浏览器工作台 |
| `skills/` | 面向 Agent 的工作流和工具 schema 草案 |
| `docs/` | 双语产品契约 |
| `scripts/` | 离线仓库契约检查 |

## 贡献与支持

- [贡献指南](CONTRIBUTING.zh-CN.md)
- [安全策略](SECURITY.zh-CN.md)
- [支持](SUPPORT.zh-CN.md)
- [行为准则](CODE_OF_CONDUCT.zh-CN.md)
- [变更日志](CHANGELOG.md)

[在 Ko-fi 上支持 Tinkora](https://ko-fi.com/tinkora)

## 许可证

MIT，详见 [LICENSE](LICENSE)。
