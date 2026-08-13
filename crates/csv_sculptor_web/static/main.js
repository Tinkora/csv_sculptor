const MAX_INPUT_BYTES = 10 * 1024 * 1024;
const MAX_PREVIEW_ROWS = 500;

const messages = {
  en: {
    skip: 'Skip to workbench', tagline: 'Local CSV and TSV inspection without uploads.', loading: 'Loading', ready: 'Ready', failed: 'Unavailable',
    open_file: 'Open file', paste: 'Paste', sample: 'Load sample', header_row: 'First row is a header', row_limit: 'Maximum rows', deduplicate: 'Deduplicate', reset: 'Reset', export: 'Export',
    drop_title: 'Drop a CSV or TSV file here', drop_help: 'or use Open file or Paste. UTF-8 text up to 10 MiB.', rows: 'rows', columns: 'columns', delimiter: 'delimiter',
    paste_title: 'Paste CSV or TSV', paste_help: 'Input stays in this browser tab.', paste_label: 'CSV or TSV input', cancel: 'Cancel', import: 'Import',
    export_title: 'Export data', export_help: 'Review generated text before copying or downloading it.', format: 'Format', table_name: 'Table name', output: 'Generated output', close: 'Close', copy: 'Copy', download: 'Download', formula_warning: '{count} cell(s) start with =, +, -, or @ and may be interpreted as spreadsheet formulas. Review before opening the export.',
    filters: 'Column filters', visible_columns: 'Visible columns', show_column: 'Show', column_word: 'column', filter_value: 'Filter value', all: 'No filter', equals: 'Equals', not_equals: 'Not equals', contains: 'Contains', starts: 'Starts with', ends: 'Ends with', greater: 'Greater than', less: 'Less than', empty: 'Is empty', not_empty: 'Is not empty',
    engine_failed: 'The WASM processing engine could not be loaded. Reload the page or rebuild the package.', empty_input: 'Paste CSV or TSV data before importing.', invalid_utf8: 'The selected file is not valid UTF-8.', too_large: 'Input exceeds the 10 MiB limit.', loaded: 'Loaded {rows} rows and {columns} columns.', preview: 'Previewing the first {rows} rows.', copied: 'Output copied.', downloaded: 'Download started.', reset_done: 'Original data restored.', deduplicated: 'Duplicate rows removed.', export_failed: 'Export failed: {error}', parse_failed: 'Parse failed: {error}', file_failed: 'The file could not be read.', filter_failed: 'Filter failed: {error}', sort_failed: 'Sort failed: {error}', column_hidden: 'At least one column must remain visible.', invalid_row_limit: 'Maximum rows must be a positive whole number.',
  },
  zh: {
    skip: '跳到工作台', tagline: '本地检查 CSV 和 TSV，不上传数据。', loading: '加载中', ready: '就绪', failed: '不可用',
    open_file: '打开文件', paste: '粘贴', sample: '加载示例', header_row: '第一行是表头', row_limit: '最大行数', deduplicate: '去重', reset: '重置', export: '导出',
    drop_title: '将 CSV 或 TSV 文件拖放到此处', drop_help: '也可使用“打开文件”或“粘贴”。支持最大 10 MiB 的 UTF-8 文本。', rows: '行', columns: '列', delimiter: '分隔符',
    paste_title: '粘贴 CSV 或 TSV', paste_help: '输入只保留在当前浏览器标签页。', paste_label: 'CSV 或 TSV 输入', cancel: '取消', import: '导入',
    export_title: '导出数据', export_help: '复制或下载前先检查生成内容。', format: '格式', table_name: '表名', output: '生成结果', close: '关闭', copy: '复制', download: '下载', formula_warning: '有 {count} 个单元格以 =、+、- 或 @ 开头，打开导出文件时可能被电子表格解释为公式。请先检查。',
    filters: '列筛选', visible_columns: '可见列', show_column: '显示', column_word: '列', filter_value: '筛选值', all: '不筛选', equals: '等于', not_equals: '不等于', contains: '包含', starts: '开头是', ends: '结尾是', greater: '大于', less: '小于', empty: '为空', not_empty: '非空',
    engine_failed: 'WASM 处理引擎加载失败。请刷新页面或重新构建 package。', empty_input: '请先粘贴 CSV 或 TSV 数据。', invalid_utf8: '所选文件不是有效的 UTF-8。', too_large: '输入超过 10 MiB 限制。', loaded: '已加载 {rows} 行、{columns} 列。', preview: '仅预览前 {rows} 行。', copied: '已复制输出。', downloaded: '已开始下载。', reset_done: '已恢复原始数据。', deduplicated: '已移除重复行。', export_failed: '导出失败：{error}', parse_failed: '解析失败：{error}', file_failed: '无法读取文件。', filter_failed: '筛选失败：{error}', sort_failed: '排序失败：{error}', column_hidden: '至少保留一列可见。', invalid_row_limit: '最大行数必须是正整数。',
  },
};

const state = { language: 'en', wasm: null, original: null, working: null, current: null, controls: null, filters: new Map(), sort: null, visible: new Set(), rowLimit: null, exportText: '', exportFormat: 'csv' };
const byId = (id) => document.getElementById(id);

function t(key, replacements = {}) {
  let value = messages[state.language][key] || key;
  for (const [name, replacement] of Object.entries(replacements)) value = value.replace(`{${name}}`, String(replacement));
  return value;
}

function setMessage(key, replacements = {}) { byId('message').textContent = key ? t(key, replacements) : ''; }
function setError(key, replacements = {}) { byId('error').hidden = !key; byId('error').textContent = key ? t(key, replacements) : ''; }

function errorText(error) {
  if (typeof error === 'string') return error;
  if (error?.message) return error.message;
  if (error && typeof error === 'object' && 'code' in error) return `${error.code}: ${error.message || ''}`.trim();
  return String(error);
}

function applyLanguage() {
  document.documentElement.lang = state.language === 'en' ? 'en' : 'zh-CN';
  document.querySelectorAll('[data-i18n]').forEach((node) => { node.textContent = t(node.dataset.i18n); });
  byId('language-button').textContent = state.language === 'en' ? '中' : 'EN';
  byId('language-button').setAttribute('aria-label', state.language === 'en' ? 'Switch to Chinese' : '切换到英文');
  byId('engine-status').textContent = state.wasm ? t('ready') : t('loading');
  if (state.current) {
    render(JSON.parse(state.current), state.controls ? JSON.parse(state.controls) : JSON.parse(state.working));
    if (!byId('export-dialog').open) byId('formula-warning').hidden = true;
    else updateFormulaWarning(state.exportFormat);
  }
}

function parseResult(value) { return typeof value === 'string' ? value : String(value); }

function formulaLikeCellCount(tableJson) {
  const table = JSON.parse(tableJson);
  return table.rows.reduce(
    (count, row) => count + row.filter((value) => /^[=+\-@]/.test(value)).length,
    0,
  );
}

function updateFormulaWarning(format) {
  const warning = byId('formula-warning');
  const count = format === 'csv' || format === 'tsv'
    ? formulaLikeCellCount(state.current)
    : 0;
  warning.hidden = count === 0;
  warning.textContent = count > 0 ? t('formula_warning', { count }) : '';
}

async function initialize() {
  try {
    const wasm = await import('./pkg/csv_sculptor_web.js');
    await wasm.default();
    state.wasm = wasm;
    byId('engine-status').textContent = t('ready');
    byId('engine-status').className = 'engine-status ready';
  } catch (error) {
    byId('engine-status').textContent = t('failed');
    byId('engine-status').className = 'engine-status failed';
    setError('engine_failed');
    console.error('CSV Sculptor WASM initialization failed', error);
  }
}

function loadText(text) {
  setError('');
  setMessage('');
  if (!state.wasm) { setError('engine_failed'); return; }
  if (!text.trim()) { setError('empty_input'); return; }
  if (new TextEncoder().encode(text).byteLength > MAX_INPUT_BYTES) { setError('too_large'); return; }
  try {
    const tableJson = parseResult(state.wasm.parse_csv(text, byId('has-header').checked));
    state.original = tableJson;
    state.working = tableJson;
    state.current = tableJson;
    state.controls = tableJson;
    state.filters.clear();
    state.sort = null;
    state.rowLimit = null;
    const table = JSON.parse(tableJson);
    state.visible = new Set(table.headers);
    byId('drop-zone').hidden = true;
    byId('data-panel').hidden = false;
    byId('deduplicate-button').disabled = false;
    byId('reset-button').disabled = false;
    byId('export-button').disabled = false;
    byId('row-limit').disabled = false;
    byId('row-limit').value = '';
    render(table);
    setMessage('loaded', { rows: table.row_count, columns: table.headers.length });
  } catch (error) {
    setError('parse_failed', { error: errorText(error) });
  }
}

function createOption(value, label) { const option = document.createElement('option'); option.value = value; option.textContent = label; return option; }

function renderFilters(table) {
  const strip = byId('filter-strip');
  strip.replaceChildren();
  strip.setAttribute('aria-label', t('filters'));
  const operators = [['', 'all'], ['Equals', 'equals'], ['NotEquals', 'not_equals'], ['Contains', 'contains'], ['StartsWith', 'starts'], ['EndsWith', 'ends'], ['GreaterThan', 'greater'], ['LessThan', 'less'], ['IsEmpty', 'empty'], ['IsNotEmpty', 'not_empty']];
  for (const header of table.headers) {
    const wrapper = document.createElement('div'); wrapper.className = 'filter-control';
    const label = document.createElement('label'); label.textContent = header;
    const select = document.createElement('select'); select.setAttribute('aria-label', `${header}: ${t('filters')}`);
    for (const [value, key] of operators) select.append(createOption(value, t(key)));
    const saved = state.filters.get(header); select.value = saved?.operator || '';
    const input = document.createElement('input'); input.setAttribute('aria-label', `${header}: ${t('filter_value')}`); input.placeholder = t('filter_value'); input.value = saved?.value || '';
    select.addEventListener('change', () => updateFilter(header, select.value, input.value));
    input.addEventListener('change', () => updateFilter(header, select.value, input.value));
    wrapper.append(label, select, input); strip.append(wrapper);
  }
}

function renderTable(table) {
  const scroll = byId('table-scroll'); scroll.replaceChildren();
  const tableElement = document.createElement('table');
  const head = document.createElement('thead'); const headRow = document.createElement('tr');
  const indexHead = document.createElement('th'); indexHead.scope = 'col'; indexHead.textContent = '#'; headRow.append(indexHead);
  const visibleHeaders = table.headers.filter((header) => state.visible.has(header));
  for (const header of visibleHeaders) {
    const cell = document.createElement('th'); cell.scope = 'col';
    const button = document.createElement('button'); button.type = 'button'; button.className = 'sort-button';
    const direction = state.sort?.column === header ? (state.sort.ascending ? ' ↑' : ' ↓') : '';
    button.textContent = `${header}${direction}`; button.setAttribute('aria-label', `Sort by ${header}`); button.addEventListener('click', () => sortBy(header));
    cell.append(button); headRow.append(cell);
  }
  head.append(headRow); tableElement.append(head);
  const body = document.createElement('tbody');
  table.rows.slice(0, MAX_PREVIEW_ROWS).forEach((row, rowIndex) => {
    const tr = document.createElement('tr'); const number = document.createElement('td'); number.className = 'row-number'; number.textContent = String(rowIndex + 1); tr.append(number);
    table.headers.forEach((header, columnIndex) => { if (state.visible.has(header)) { const cell = document.createElement('td'); cell.textContent = row[columnIndex] ?? ''; cell.title = row[columnIndex] ?? ''; tr.append(cell); } });
    body.append(tr);
  });
  tableElement.append(body); scroll.append(tableElement);
}

function renderColumns(table) {
  const strip = byId('column-strip'); strip.replaceChildren(); strip.setAttribute('aria-label', t('visible_columns'));
  for (const header of table.headers) {
    const label = document.createElement('label'); label.className = 'column-toggle';
    const input = document.createElement('input'); input.type = 'checkbox'; input.checked = state.visible.has(header); input.setAttribute('aria-label', `${t('show_column')} ${header} ${t('column_word')}`);
    input.addEventListener('change', () => { if (!input.checked && state.visible.size === 1) { input.checked = true; setError('column_hidden'); return; } setError(''); input.checked ? state.visible.add(header) : state.visible.delete(header); try { rebuildCurrent(); } catch (error) { setError('parse_failed', { error: errorText(error) }); } });
    const text = document.createElement('span'); text.textContent = header; label.append(input, text); strip.append(label);
  }
}

function render(table, controlsTable = table) {
  byId('row-count').textContent = String(table.row_count); byId('column-count').textContent = String(table.headers.length);
  byId('delimiter-value').textContent = table.delimiter === '\t' ? 'TAB' : table.delimiter;
  byId('preview-note').textContent = table.row_count > MAX_PREVIEW_ROWS ? t('preview', { rows: MAX_PREVIEW_ROWS }) : '';
  renderFilters(controlsTable); renderTable(table); renderColumns(controlsTable);
}

function rebuildCurrent() {
  let value = state.working;
  if (state.sort) value = parseResult(state.wasm.sort_table(value, state.sort.column, state.sort.ascending));
  const filters = [...state.filters.entries()].filter(([, condition]) => condition.operator).map(([column, condition]) => ({ column, ...condition }));
  if (filters.length) value = parseResult(state.wasm.filter_table(value, JSON.stringify(filters)));
  const controlsTable = JSON.parse(value);
  const selectedColumns = controlsTable.headers.filter((header) => state.visible.has(header));
  if (selectedColumns.length !== controlsTable.headers.length) value = parseResult(state.wasm.select_columns(value, JSON.stringify(selectedColumns)));
  if (state.rowLimit !== null) value = parseResult(state.wasm.limit_table(value, state.rowLimit));
  state.controls = JSON.stringify(controlsTable);
  state.current = value; render(JSON.parse(value), controlsTable);
}

function updateFilter(column, operator, value) { state.filters.set(column, { operator, value }); try { rebuildCurrent(); setError(''); } catch (error) { setError('filter_failed', { error: errorText(error) }); } }
function sortBy(column) { state.sort = { column, ascending: state.sort?.column === column ? !state.sort.ascending : true }; try { rebuildCurrent(); setError(''); } catch (error) { setError('sort_failed', { error: errorText(error) }); } }

function reset() { if (!state.original) return; state.working = state.original; state.current = state.original; state.controls = state.original; state.filters.clear(); state.sort = null; state.rowLimit = null; const table = JSON.parse(state.original); state.visible = new Set(table.headers); byId('row-limit').value = ''; render(table); setMessage('reset_done'); setError(''); }
function deduplicate() { if (!state.working) return; try { state.working = parseResult(state.wasm.deduplicate_table(state.working)); rebuildCurrent(); setMessage('deduplicated'); setError(''); } catch (error) { setError('parse_failed', { error: errorText(error) }); } }

function updateRowLimit() {
  const input = byId('row-limit');
  if (!input.value) { state.rowLimit = null; setError(''); if (state.working) rebuildCurrent(); return; }
  const limit = Number(input.value);
  if (!Number.isSafeInteger(limit) || limit < 1) { input.value = state.rowLimit ?? ''; setError('invalid_row_limit'); return; }
  state.rowLimit = limit;
  try { rebuildCurrent(); setError(''); } catch (error) { setError('parse_failed', { error: errorText(error) }); }
}

function updateExport() {
  if (!state.current) return;
  try {
    const format = byId('export-format').value; state.exportFormat = format; const isSql = format === 'sql';
    byId('table-name').hidden = !isSql; byId('table-name-label').hidden = !isSql;
    state.exportText = state.wasm.export_table(state.current, format, byId('table-name').value); byId('export-output').value = state.exportText; setError('');
    updateFormulaWarning(format);
  } catch (error) { setError('export_failed', { error: errorText(error) }); }
}

async function copyExport() { try { await navigator.clipboard.writeText(state.exportText); setMessage('copied'); } catch (error) { setError('export_failed', { error: errorText(error) }); } }
function downloadExport() { const extensions = { csv: 'csv', tsv: 'tsv', json_pretty: 'json', yaml: 'yaml', markdown: 'md', sql: 'sql' }; const blob = new Blob([state.exportText], { type: 'text/plain;charset=utf-8' }); const link = document.createElement('a'); link.href = URL.createObjectURL(blob); link.download = `csv_sculptor_export.${extensions[state.exportFormat]}`; link.click(); URL.revokeObjectURL(link.href); setMessage('downloaded'); }

async function readFile(file) {
  if (file.size > MAX_INPUT_BYTES) { setError('too_large'); return; }
  try { const bytes = await file.arrayBuffer(); const text = new TextDecoder('utf-8', { fatal: true }).decode(bytes); loadText(text); } catch (error) { setError(error instanceof TypeError ? 'invalid_utf8' : 'file_failed'); }
}

byId('file-input').addEventListener('change', (event) => { const file = event.target.files[0]; if (file) readFile(file); event.target.value = ''; });
byId('paste-button').addEventListener('click', () => byId('paste-dialog').showModal());
byId('paste-import-button').addEventListener('click', () => { const input = byId('paste-input').value; if (!input.trim()) { setError('empty_input'); return; } byId('paste-dialog').close(); loadText(input); });
byId('sample-button').addEventListener('click', () => loadText('agent,status,duration_ms\nplanner,ok,42\nexecutor,retry,310\nexecutor,ok,196\n'));
byId('reset-button').addEventListener('click', reset); byId('deduplicate-button').addEventListener('click', deduplicate);
byId('row-limit').addEventListener('change', updateRowLimit);
byId('export-button').addEventListener('click', () => { updateExport(); byId('export-dialog').showModal(); });
byId('export-format').addEventListener('change', updateExport); byId('table-name').addEventListener('input', updateExport);
byId('copy-button').addEventListener('click', copyExport); byId('download-button').addEventListener('click', downloadExport);
byId('language-button').addEventListener('click', () => { state.language = state.language === 'en' ? 'zh' : 'en'; applyLanguage(); });
for (const eventName of ['dragenter', 'dragover']) byId('drop-zone').addEventListener(eventName, (event) => { event.preventDefault(); byId('drop-zone').classList.add('dragging'); });
for (const eventName of ['dragleave', 'drop']) byId('drop-zone').addEventListener(eventName, (event) => { event.preventDefault(); byId('drop-zone').classList.remove('dragging'); });
byId('drop-zone').addEventListener('drop', (event) => { const file = event.dataTransfer.files[0]; if (file) readFile(file); });

applyLanguage();
await initialize();
