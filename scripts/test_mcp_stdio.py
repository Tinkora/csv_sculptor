#!/usr/bin/env python3
"""Exercise the local MCP server through its real stdio boundary."""

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def request(request_id: int, method: str, params: dict | None = None) -> str:
    payload = {"jsonrpc": "2.0", "id": request_id, "method": method}
    if params is not None:
        payload["params"] = params
    return json.dumps(payload, separators=(",", ":"))


def main() -> None:
    table_input = {
        "headers": ["name", "age"],
        "rows": [["Ada", "37"], ["Bob", "29"]],
        "delimiter": ",",
        "row_count": 2,
    }
    lines = [
        request(
            1,
            "initialize",
            {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "contract-test", "version": "1"},
            },
        ),
        json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        request(2, "tools/list", {}),
        request(
            3,
            "tools/call",
            {
                "name": "csv_sculptor_filter",
                "arguments": {
                    "table": table_input,
                    "conditions": [
                        {"column": "age", "operator": "GreaterThan", "value": "30"}
                    ],
                },
            },
        ),
        request(
            4,
            "tools/call",
            {
                "name": "csv_sculptor_filter",
                "arguments": {
                    "table": {**table_input, "rows": [["Ada"]]},
                    "conditions": [],
                },
            },
        ),
    ]
    result = subprocess.run(
        ["cargo", "run", "--quiet", "--locked", "-p", "csv_sculptor_mcp"],
        cwd=ROOT,
        input="\n".join(lines) + "\n",
        capture_output=True,
        text=True,
        timeout=120,
        check=False,
    )
    if result.returncode != 0:
        raise AssertionError(f"MCP server exited with {result.returncode}: {result.stderr}")

    responses = [json.loads(line) for line in result.stdout.splitlines() if line.strip()]
    by_id = {response["id"]: response for response in responses}
    assert by_id[1]["result"]["serverInfo"]["name"] == "csv_sculptor_mcp"
    tools = by_id[2]["result"]["tools"]
    assert {tool["name"] for tool in tools} == {
        "csv_sculptor_parse",
        "csv_sculptor_filter",
        "csv_sculptor_sort",
        "csv_sculptor_export",
        "csv_sculptor_detect_delimiter",
    }
    successful = by_id[3]["result"]
    assert successful["isError"] is False
    assert successful["structuredContent"]["data"]["table"]["row_count"] == 1
    failed = by_id[4]["result"]
    assert failed["isError"] is True
    assert "INVALID_INPUT:" in failed["content"][0]["text"]


if __name__ == "__main__":
    main()
