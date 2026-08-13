#!/usr/bin/env python3
"""Offline contract tests for release evidence generation."""

import json
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/generate_release_evidence.py"


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="csv_sculptor_evidence_") as directory:
        root = Path(directory)
        dist = root / "dist"
        dist.mkdir()
        (dist / "bundle.tar.gz").write_bytes(b"bundle")
        metadata = root / "metadata.json"
        metadata.write_text(
            json.dumps(
                {
                    "packages": [
                        {
                            "id": "path+file:///workspace#csv_sculptor_core@0.1.0-alpha.1",
                            "name": "csv_sculptor_core",
                            "version": "0.1.0-alpha.1",
                            "license": "MIT",
                        },
                        {
                            "id": "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.0",
                            "name": "serde",
                            "version": "1.0.0",
                            "license": "MIT OR Apache-2.0",
                            "source": "registry+https://github.com/rust-lang/crates.io-index",
                        },
                    ],
                    "workspace_members": ["path+file:///workspace#csv_sculptor_core@0.1.0-alpha.1"],
                    "resolve": {"nodes": []},
                }
            ),
            encoding="utf-8",
        )
        subprocess.run(
            [
                "python3",
                str(SCRIPT),
                "--dist",
                str(dist),
                "--metadata",
                str(metadata),
                "--repository",
                "Tinkora/csv_sculptor",
                "--version",
                "0.1.0-alpha.1",
                "--revision",
                "abc123",
                "--created",
                "2026-08-14T00:00:00Z",
            ],
            check=True,
        )
        sbom = json.loads((dist / "sbom.spdx.json").read_text(encoding="utf-8"))
        assert sbom["spdxVersion"] == "SPDX-2.3"
        assert sbom["creationInfo"]["created"] == "2026-08-14T00:00:00Z"
        assert len(sbom["packages"]) == 2
        assert (dist / "SHA256SUMS").read_text(encoding="ascii").startswith("1e6ed65d77")
        inventory = json.loads((dist / "license_inventory.json").read_text(encoding="utf-8"))
        assert inventory["packages"][1]["name"] == "serde"

    print("Release evidence contract passed.")


if __name__ == "__main__":
    main()
