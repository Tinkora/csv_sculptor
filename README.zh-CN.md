# CSV Sculptor

CSV Sculptor 是一个浏览器原生的 CSV/TSV 工作台，用于查看、筛选、排序和转换表格文本，输入不会上传。Rust 负责数据行为，精简的 WebAssembly 边界将其提供给浏览器。

[English](README.md)

## 成熟度

**Alpha。** 托管质量和发布工作流已通过，浏览器工作台已部署到 GitHub Pages，发布包包含 checksum、SPDX SBOM、许可证清单和构建证明。

- **在线体验：** [GitHub Pages](https://tinkora.github.io/csv_sculptor/)
- **最新候选版本：** [v0.1.0-alpha.3 Release](https://github.com/Tinkora/csv_sculptor/releases/tag/v0.1.0-alpha.3)

- **本地人类界面：** 已实现，并由托管 Chromium smoke 测试覆盖。
- **本地 Agent 界面：** `csv_sculptor_mcp` 通过 stdio 提供五个 MCP 工具。
- **没有托管服务：** MCP server 不提供网络传输、账户或身份验证，输入只留在本地进程中。

## 当前范围

- 解析最大 10 MiB 的 UTF-8 CSV、TSV、竖线和分号分隔文本，包括带引号字段。
- 拒绝空白或重复表头以及列数不一致的行。
- 使用九种运算符筛选，并以 AND 语义组合所有启用的筛选条件。
- 数字列按数值排序，其他列按不区分大小写的文本排序。
- 从当前结果中选择列、限制行数并移除重复行。
- 导出 JSON、YAML、Markdown 表格、SQL `INSERT`、CSV 或 TSV。
- 所有导入数据仅保留在当前浏览器标签页中。
- 工作台支持英文和简体中文切换。

## Agent 集成

在仓库根目录构建本地 MCP server：

```bash
cargo build --release -p csv_sculptor_mcp --locked
```

将生成的 `target/release/csv_sculptor_mcp` 注册到 MCP 客户端的本地 stdio
配置中。server 提供以下工具：

| 工具 | 用途 |
| --- | --- |
| `csv_sculptor_parse` | 解析有大小限制的 CSV/TSV 文本并返回结构化表格 |
| `csv_sculptor_filter` | 使用 AND 语义组合筛选条件 |
| `csv_sculptor_sort` | 按数值或文本排序一列 |
| `csv_sculptor_export` | 生成确定性的 JSON、YAML、Markdown、SQL、CSV 或 TSV 文本 |
| `csv_sculptor_detect_delimiter` | 检查分隔符而不解析表格 |

转换和导出工具接收结构化的 `table` 对象，因此 Agent 可以直接串联调用，
不需要在 JSON 中再次嵌入 JSON 字符串。每个成功结果都使用
`{ "schema_version": "1", "tool": "...", "data": ... }` envelope；无效输入
会作为 MCP tool error 返回稳定的 core 错误码。

原始 CSV/TSV 数据上限为 10 MiB。stdio JSON 行上限为 64 MiB，以容纳转义后的
表示；超长行会在工具分发前丢弃。server 将诊断写入 stderr，只把协议消息写入
stdout。机器可读目录见 [`skills/mcp-tools.json`](skills/mcp-tools.json)。

列选择和行数限制同时作用于浏览器预览及所有导出格式；重置会恢复完整的导入表格。

## 安全边界

- 输入按 UTF-8 解码，应用不会把输入发送到服务器。
- 预览使用 `textContent` 而不是 HTML 渲染单元格内容。
- CSV/TSV 导出保留原始单元格值。导出对话框会按安全策略检查解析后的字段：跳过可选 ASCII 空格后，如果字段以 ASCII 或全角公式前缀开头就显示警告。工具不会改写数据；使用电子表格软件打开不可信导出前必须检查内容。
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
| `crates/csv_sculptor_mcp` | 本地 stdio MCP server 和有界 Agent 工具 |
| `skills/` | 面向 Agent 的工作流和机器可读工具 schema |
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
