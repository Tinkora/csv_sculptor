#!/usr/bin/env python3
"""Generate deterministic checksums and dependency evidence for a web release."""

import argparse
import hashlib
import json
import re
from pathlib import Path
from urllib.parse import quote


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dist", type=Path, required=True)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--revision", required=True)
    parser.add_argument("--created", required=True)
    return parser.parse_args()


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def spdx_id(name: str, version: str, package_id: str) -> str:
    suffix = hashlib.sha256(package_id.encode("utf-8")).hexdigest()[:12]
    value = re.sub(r"[^A-Za-z0-9.-]+", "-", f"{name}-{version}").strip("-")
    return f"SPDXRef-Package-{value}-{suffix}"


def build_spdx(metadata: dict, repository: str, version: str, revision: str, created: str) -> dict:
    packages = sorted(
        metadata["packages"],
        key=lambda item: (item["name"], item["version"], item["id"]),
    )
    workspace_members = set(metadata.get("workspace_members", []))
    ids = {item["id"]: spdx_id(item["name"], item["version"], item["id"]) for item in packages}
    spdx_packages = []
    for package in packages:
        spdx_packages.append(
            {
                "SPDXID": ids[package["id"]],
                "copyrightText": "NOASSERTION",
                "downloadLocation": "NOASSERTION",
                "externalRefs": [
                    {
                        "referenceCategory": "PACKAGE-MANAGER",
                        "referenceLocator": f"pkg:cargo/{quote(package['name'])}@{quote(package['version'])}",
                        "referenceType": "purl",
                    }
                ],
                "filesAnalyzed": False,
                "licenseConcluded": "NOASSERTION",
                "licenseDeclared": package.get("license") or "NOASSERTION",
                "name": package["name"],
                "sourceInfo": (
                    "workspace"
                    if package["id"] in workspace_members
                    else package.get("source") or "unknown"
                ),
                "versionInfo": package["version"],
            }
        )

    relationships = []
    for node in sorted(metadata.get("resolve", {}).get("nodes", []), key=lambda item: item["id"]):
        source_id = ids.get(node["id"])
        if not source_id:
            continue
        for dependency in sorted(node.get("deps", []), key=lambda item: item["pkg"]):
            target_id = ids.get(dependency["pkg"])
            if target_id:
                relationships.append(
                    {
                        "spdxElementId": source_id,
                        "relatedSpdxElement": target_id,
                        "relationshipType": "DEPENDS_ON",
                    }
                )

    for member in sorted(metadata.get("workspace_members", [])):
        if member in ids:
            relationships.append(
                {
                    "spdxElementId": "SPDXRef-DOCUMENT",
                    "relatedSpdxElement": ids[member],
                    "relationshipType": "DESCRIBES",
                }
            )

    return {
        "SPDXID": "SPDXRef-DOCUMENT",
        "creationInfo": {
            "created": created,
            "creators": ["Tool: Tinkora release evidence generator"],
        },
        "dataLicense": "CC0-1.0",
        "documentNamespace": f"https://github.com/{repository}/sbom/{quote(version)}/{revision}",
        "name": f"{repository.replace('/', '-')}-{version}",
        "packages": spdx_packages,
        "relationships": relationships,
        "spdxVersion": "SPDX-2.3",
    }


def main() -> None:
    args = parse_args()
    if not args.dist.is_dir():
        raise ValueError("dist must be an existing directory")
    metadata = json.loads(args.metadata.read_text(encoding="utf-8"))
    workspace_members = set(metadata.get("workspace_members", []))
    artifacts = sorted(
        path for path in args.dist.iterdir() if path.is_file() and not path.name.endswith(".sha256")
    )
    if not artifacts:
        raise ValueError("dist contains no release artifacts")

    checksums = "".join(f"{sha256(path)}  {path.name}\n" for path in artifacts)
    (args.dist / "SHA256SUMS").write_text(checksums, encoding="ascii")

    packages = []
    for package in sorted(metadata["packages"], key=lambda item: (item["name"], item["version"])):
        packages.append(
            {
                "name": package["name"],
                "version": package["version"],
                "license": package.get("license") or "NOASSERTION",
                "source": (
                    "workspace"
                    if package["id"] in workspace_members
                    else package.get("source") or "unknown"
                ),
                "purl": f"pkg:cargo/{quote(package['name'])}@{quote(package['version'])}",
            }
        )

    evidence = {
        "schema_version": 1,
        "repository": args.repository,
        "version": args.version,
        "revision": args.revision,
        "packages": packages,
    }
    (args.dist / "license_inventory.json").write_text(
        json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    (args.dist / "sbom.spdx.json").write_text(
        json.dumps(
            build_spdx(metadata, args.repository, args.version, args.revision, args.created),
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    (args.dist / "THIRD_PARTY_NOTICES.md").write_text(
        "# Third-Party Notices\n\n"
        "This release includes the Rust dependencies listed in `license_inventory.json`.\n\n"
        "| Package | Version | License |\n| --- | --- | --- |\n"
        + "".join(
            f"| {item['name']} | {item['version']} | {item['license']} |\n"
            for item in packages
            if item["source"] != "workspace"
        ),
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
