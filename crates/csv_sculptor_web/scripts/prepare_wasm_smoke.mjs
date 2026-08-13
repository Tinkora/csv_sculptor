import { cp, lstat, mkdir, readdir, realpath, rm } from "node:fs/promises";
import { basename, join, relative, resolve } from "node:path";

const artifact = process.env.WASM_SMOKE_PACKAGE;
const destination = resolve("static/pkg");
const required = ["package.json", "csv_sculptor_web.js", "csv_sculptor_web_bg.wasm"];

async function validateTree(root, current = root) {
  for (const entry of await readdir(current)) {
    const path = join(current, entry);
    const metadata = await lstat(path);
    if (metadata.isSymbolicLink()) throw new Error(`WASM package contains a symbolic link: ${relative(root, path)}`);
    if (metadata.isDirectory()) await validateTree(root, path);
    else if (!metadata.isFile()) throw new Error(`WASM package contains a special file: ${relative(root, path)}`);
  }
}

async function validatePackage(root) {
  const metadata = await lstat(root);
  if (!metadata.isDirectory() || metadata.isSymbolicLink()) throw new Error("WASM package must be a real directory");
  await validateTree(root);
  for (const filename of required) {
    const file = await lstat(join(root, filename));
    if (!file.isFile() || file.isSymbolicLink()) throw new Error(`WASM package is missing ${filename}`);
  }
}

if (!artifact) {
  await validatePackage(destination);
} else {
  const source = await realpath(artifact);
  if (source === destination) {
    await validatePackage(source);
  } else {
    await validatePackage(source);
    await rm(destination, { recursive: true, force: true });
    await mkdir(destination, { recursive: true });
    for (const entry of await readdir(source)) {
      await cp(join(source, entry), join(destination, basename(entry)), { recursive: true, force: false, errorOnExist: true });
    }
    await validatePackage(destination);
  }
}
