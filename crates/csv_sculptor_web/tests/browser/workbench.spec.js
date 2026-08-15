import { expect, test } from "@playwright/test";

async function loadWorkbench(page) {
  await page.goto("/static/");
  await expect(page.getByText("Ready", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Load sample" }).click();
  await expect(page.getByText("Loaded 3 rows and 3 columns.")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Drop a CSV or TSV file here" })).toBeHidden();
}

test("loads, filters, sorts, and exports through the real WASM boundary", async ({ page }) => {
  await loadWorkbench(page);

  const statusFilter = page.getByLabel("status: Column filters");
  await statusFilter.selectOption("Contains");
  await page.getByLabel("status: Filter value").fill("retry");
  await page.getByLabel("status: Filter value").press("Tab");
  await expect(page.locator("tbody tr")).toHaveCount(1);
  await expect(page.locator("tbody")).toContainText("retry");

  await page.getByRole("button", { name: "Reset" }).click();
  await page.getByRole("button", { name: "Sort by duration_ms" }).click();
  await page.getByRole("button", { name: "Sort by duration_ms" }).click();
  await expect(page.locator("tbody tr").first()).toContainText("310");

  await page.getByRole("button", { name: "Export" }).click();
  await page.getByLabel("Format").selectOption("json_pretty");
  await expect(page.getByLabel("Generated output")).toHaveValue(/"agent": "executor"/);
});

test("warns before exporting spreadsheet formula-like cells", async ({ page }) => {
  await page.goto("/static/");
  await expect(page.getByText("Ready", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Paste" }).click();
  await page.getByLabel("CSV or TSV input").fill(
    "name,＝value\nfirst,=1+1\nsecond,ok\nthird,\"  @cmd\"\nfourth,\"\tcommand\"\nfifth,＝1+1\n",
  );
  await page.getByRole("button", { name: "Import" }).click();

  await page.getByRole("button", { name: "Export" }).click();
  await expect(page.getByRole("alert")).toHaveText(
    "5 cell(s) may be interpreted as spreadsheet formulas after optional leading spaces. Review before opening the export.",
  );
  await page.getByLabel("Format").selectOption("json_pretty");
  await expect(page.getByRole("alert")).toBeHidden();
});

test("language switching preserves the imported table", async ({ page }) => {
  const pageErrors = [];
  page.on("pageerror", (error) => pageErrors.push(error.message));
  await loadWorkbench(page);
  await page.getByRole("button", { name: "Switch to Chinese" }).click();
  await expect(page.getByRole("button", { name: "加载示例" })).toBeVisible();
  await expect(page.locator("tbody tr")).toHaveCount(3);
  expect(pageErrors).toEqual([]);
});

test("reset restores rows removed by deduplication", async ({ page }) => {
  await page.goto("/static/");
  await expect(page.getByText("Ready", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Paste" }).click();
  await page.getByLabel("CSV or TSV input").fill("name,value\nfirst,1\nfirst,1\n");
  await page.getByRole("button", { name: "Import" }).click();
  await expect(page.locator("tbody tr")).toHaveCount(2);
  await page.getByRole("button", { name: "Deduplicate" }).click();
  await expect(page.locator("tbody tr")).toHaveCount(1);
  await page.getByRole("button", { name: "Reset" }).click();
  await expect(page.locator("tbody tr")).toHaveCount(2);
});

test("column selection changes both the preview and exported data", async ({ page }) => {
  await loadWorkbench(page);

  await page.getByRole("checkbox", { name: "Show status column" }).uncheck();
  await expect(page.locator("thead")).not.toContainText("status");

  await page.getByRole("button", { name: "Export" }).click();
  await page.getByLabel("Format").selectOption("json_pretty");
  await expect(page.getByLabel("Generated output")).not.toHaveValue(/"status"/);
  await expect(page.getByLabel("Generated output")).toHaveValue(/"agent": "executor"/);
});

test("row limit changes both the preview and exported data", async ({ page }) => {
  await loadWorkbench(page);

  await page.getByLabel("Maximum rows").fill("2");
  await page.getByLabel("Maximum rows").press("Tab");
  await expect(page.locator("tbody tr")).toHaveCount(2);

  await page.getByRole("button", { name: "Export" }).click();
  await page.getByLabel("Format").selectOption("json_pretty");
  const exported = JSON.parse(await page.getByLabel("Generated output").inputValue());
  expect(exported).toHaveLength(2);
  expect(exported.map((row) => row.duration_ms)).toEqual(["42", "310"]);
});

test("invalid row limits keep the last valid result", async ({ page }) => {
  await loadWorkbench(page);

  await page.getByLabel("Maximum rows").fill("1");
  await page.getByLabel("Maximum rows").press("Tab");
  await expect(page.locator("tbody tr")).toHaveCount(1);

  await page.getByLabel("Maximum rows").fill("0");
  await page.getByLabel("Maximum rows").press("Tab");
  await expect(page.getByRole("alert")).toContainText("Maximum rows must be a positive whole number.");
  await expect(page.getByLabel("Maximum rows")).toHaveValue("1");
  await page.getByRole("checkbox", { name: "Show status column" }).uncheck();
  await expect(page.locator("tbody tr")).toHaveCount(1);
});

test("reset restores all selected columns and rows", async ({ page }) => {
  await loadWorkbench(page);

  await page.getByRole("checkbox", { name: "Show status column" }).uncheck();
  await page.getByLabel("Maximum rows").fill("1");
  await page.getByLabel("Maximum rows").press("Tab");
  await expect(page.locator("tbody tr")).toHaveCount(1);

  await page.getByRole("button", { name: "Reset" }).click();
  await expect(page.getByRole("checkbox", { name: "Show status column" })).toBeChecked();
  await expect(page.getByLabel("Maximum rows")).toHaveValue("");
  await expect(page.locator("tbody tr")).toHaveCount(3);
  await expect(page.locator("thead")).toContainText("status");
});

test("column and row controls are keyboard operable", async ({ page }) => {
  await loadWorkbench(page);

  const statusColumn = page.getByRole("checkbox", { name: "Show status column" });
  await statusColumn.focus();
  await expect(statusColumn).toBeFocused();
  await page.keyboard.press("Space");
  await expect(statusColumn).not.toBeChecked();

  const rowLimit = page.getByLabel("Maximum rows");
  await rowLimit.focus();
  await expect(rowLimit).toBeFocused();
  await page.keyboard.type("2");
  await page.keyboard.press("Tab");
  await expect(page.locator("tbody tr")).toHaveCount(2);
});

test("deduplication, filtering, sorting, column selection, and row limits compose", async ({ page }) => {
  await page.goto("/static/");
  await expect(page.getByText("Ready", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Paste" }).click();
  await page.getByLabel("CSV or TSV input").fill([
    "agent,status,duration_ms",
    "planner,ok,42",
    "executor,retry,310",
    "executor,retry,310",
    "executor,ok,196"
  ].join("\n"));
  await page.getByRole("button", { name: "Import" }).click();

  await page.getByRole("button", { name: "Deduplicate" }).click();
  await page.getByLabel("agent: Column filters").selectOption("Contains");
  await page.getByLabel("agent: Filter value").fill("executor");
  await page.getByLabel("agent: Filter value").press("Tab");
  await page.getByRole("button", { name: "Sort by duration_ms" }).click();
  await page.getByRole("button", { name: "Sort by duration_ms" }).click();
  await page.getByRole("checkbox", { name: "Show status column" }).uncheck();
  await page.getByLabel("Maximum rows").fill("1");
  await page.getByLabel("Maximum rows").press("Tab");

  await page.getByRole("button", { name: "Export" }).click();
  await page.getByLabel("Format").selectOption("json_pretty");
  expect(JSON.parse(await page.getByLabel("Generated output").inputValue())).toEqual([
    { agent: "executor", duration_ms: "310" }
  ]);
});

test("has accessible controls and no page-level horizontal overflow", async ({ page }) => {
  const problems = [];
  page.on("console", (message) => {
    if (["error", "warning"].includes(message.type())) problems.push(`${message.type()}: ${message.text()}`);
  });
  page.on("pageerror", (error) => problems.push(`pageerror: ${error.message}`));

  await loadWorkbench(page);
  await expect(page.getByRole("heading", { name: "CSV Sculptor" })).toBeVisible();
  await expect(page.getByLabel("CSV data table")).toBeVisible();
  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > document.documentElement.clientWidth
  );
  expect(horizontalOverflow).toBe(false);
  expect(problems).toEqual([]);
});
