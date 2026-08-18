# CSV Sculptor 产品规格

[English](product_spec.md)

## 问题与用户

开发者和数据工作者经常需要快速查看未知来源的 CSV/TSV，筛选少量行，再转换成适合 API、文档或数据库审查的格式。桌面表格软件安装较重，在线转换器又要求上传可能敏感的数据。CSV Sculptor 提供一个无需账户、仅在浏览器本地运行的工作台。

## 已验证需求

- 查看构建日志、Agent trace、测试结果和批量导出的 CSV/TSV。
- 在复制到 issue、README、配置或脚本前完成筛选、排序和格式转换。
- 处理私有数据时不把输入发送给第三方服务。
- 明确暴露编码、大小、公式解释和 SQL 方言边界。
- 覆盖电子表格和 Agent 批处理常见的编码选择。OpenAI Codex 已合并将 CSV
  作为 Agent 工作清单的 [`spawn_agents_on_csv` 工作流](https://github.com/openai/codex/pull/10935)；
  [Directus #12970](https://github.com/directus/directus/issues/12970) 等报告显示，
  Excel 生成的 UTF-8 BOM 文件可能破坏下游导入。

## 当前用户流程

1. 用户拖放或选择文件、选择编码、粘贴 UTF-8 文本或加载示例。
2. 工具自动识别 UTF-16 BOM，或在解码浏览器文件前明确选择 UTF-8、UTF-16
   LE/BE 或 Windows-1252。
3. 工具检测逗号、Tab、竖线或分号，并按用户选择决定第一行是否为表头。
4. 工具显示行数、列数、分隔符、列筛选、可排序表头、列选择和最大行数控件。
5. 用户可以组合筛选、排序、选择列、限制行数、去重或重置到原始输入。
6. 用户检查后复制或下载 JSON、YAML、Markdown、SQL、CSV 或 TSV。

## Agent 工作流

1. 构建 `csv_sculptor_mcp`，并将二进制注册为本地 MCP stdio server。
2. 使用有界 UTF-8 文本调用 `csv_sculptor_parse`。
3. 将返回的结构化 `table` 直接传给 `csv_sculptor_filter`、
   `csv_sculptor_sort` 或 `csv_sculptor_export`。
4. 在使用生成文本前检查版本化输出 envelope 和所有导出警告。

该 server 不提供托管传输、身份验证、持久化或网络访问。诊断写入 stderr，
协议 JSON 只写入 stdout。

## 行为契约

- 粘贴文本和 MCP 输入上限为 10 MiB，且必须是有效 UTF-8；浏览器文件字节遵循
  下一条编码规则。
- 浏览器文件支持 UTF-8、UTF-16 LE/BE 或 Windows-1252。自动模式识别
  UTF-16 BOM，否则使用严格 UTF-8；非法字节会被拒绝而不会被替换。粘贴和
  MCP 输入仍使用 UTF-8。
- 有表头模式拒绝空白或重复的表头。
- 不接受列数不一致的行。
- 所有启用的筛选使用 AND 语义。
- `GreaterThan` 和 `LessThan` 在两端都能解析为数字时使用数值比较，否则使用文本比较。
- 当整列都能解析为有限数字时使用数值排序，否则使用不区分大小写的文本排序。
- 预览最多显示 500 行，但导出使用全部当前结果。
- 用户明确设置的列选择和行数限制同时作用于预览和导出。
- 所有导出都必须具有确定性，并保持输入字段顺序。
- Agent 结果使用 `{ "schema_version": "1", "tool": "...", "data": ... }` envelope。
- Agent 表格输入必须有唯一且非空的表头、统一的行宽、受支持的分隔符，
  且单元格数据总大小不得超过 10 MiB。
- MCP stdio JSON 行上限为 64 MiB；超长行会在工具分发前丢弃。

## 安全与隐私

- 应用不包含上传、分析、账户、持久化或网络 API。
- 文件解码使用明确支持的编码并在本地完成；应用不会猜测任意旧编码，也不会
  静默修复非法字节。
- 单元格使用 DOM `textContent` 渲染。
- 公式检测针对解析后的字段而不是原始文本行。直接风险前缀覆盖 `=`、`+`、`-`、`@`、Tab、CR、LF 及其全角变体，依据 [OWASP CSV Injection](https://owasp.org/www-community/attacks/CSV_Injection)。
- 检测还会跳过可选前导 ASCII 空格再次检查，因为 LibreOffice 的 [Trim spaces 导入选项](https://help.libreoffice.org/latest/en-US/text/shared/00/00000208.html) 可能移除这些空格。扫描器不会删除空格或修改字段值；RFC 4180 将空格视为字段内容。
- CSV/TSV 保留公式样式的前缀并显示警告，而不会静默修改数据。[CWE-1236](https://cwe.mitre.org/data/definitions/1236.html) 指出不同电子表格产品的缓解效果不同，因此该警告不代表已经完成通用清洗。
- SQL 使用引用后的标识符和值，但用户仍需针对目标数据库方言和权限模型审查生成文本。
- 本地 stdio MCP server 已可由 Agent 调用；`skills/mcp-tools.json` 记录五个已注册工具。
  当前不提供托管端点或身份验证。

## 非目标

- XLSX、图表、协作编辑、云存储或分享链接。
- 超过 10 MiB 的流式处理或托管 MCP 服务。
- 自动执行 SQL 或自动打开电子表格文件。
- 在没有具体用户证据前加入正则表达式、多层查询构建器或持久化项目。

## Alpha 验收门槛

- Native Rust 的格式、测试和 Clippy 通过。
- `wasm32-unknown-unknown` 编译和真实 `wasm-pack` 构建通过。
- 真实 Chromium 在 375、768、1024 和 1440 像素宽度完成导入、筛选、排序、导出、键盘和溢出检查。
- 中英文文档配对，公开能力声明与实现一致。
- 依赖审计和 GitHub Actions 静态检查通过。

Alpha 发布要求精确候选 commit 通过托管质量、供应链、文档和浏览器检查；
MCP 行为还必须通过本地工具和有界 stdio 测试。
