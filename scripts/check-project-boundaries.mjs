import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, relative, resolve } from "node:path";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repository = "https://github.com/gantbd/keystone";
const expected = new Set([
  "crates/keystone-bounds",
  "crates/keystone-codec",
  "crates/keystone-effects",
  "crates/keystone-lifecycle",
  "crates/keystone-observe",
  "crates/keystone-redact",
  "crates/keystone-storage",
  "crates/keystone-storage-directory",
  "crates/keystone-storage-memory",
  "crates/keystone-testkit",
]);

const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--offline", "--locked", "--no-deps", "--format-version", "1"],
    { cwd: root, encoding: "utf8" },
  ),
);

const actual = new Set(
  metadata.packages.map((pkg) =>
    relative(root, dirname(pkg.manifest_path)).split("\\").join("/"),
  ),
);
const missing = [...expected].filter((path) => !actual.has(path));
const extra = [...actual].filter((path) => !expected.has(path));
if (missing.length || extra.length) {
  throw new Error(
    `production workspace mismatch; missing=${missing.join(",") || "none"}; extra=${extra.join(",") || "none"}`,
  );
}

for (const pkg of metadata.packages) {
  if (pkg.repository !== repository) {
    throw new Error(`${pkg.name} must inherit ${repository}`);
  }
  if (!Array.isArray(pkg.publish) || pkg.publish.length !== 0) {
    throw new Error(`${pkg.name} must remain unpublished`);
  }
  for (const dependency of pkg.dependencies) {
    if (dependency.path && !resolve(dependency.path).startsWith(`${root}/crates/`)) {
      throw new Error(`${pkg.name} has an outward production path dependency`);
    }
  }
}

const qualificationManifest = readFileSync(
  resolve(root, "qualification/chronos-moira/Cargo.toml"),
  "utf8",
);
for (const required of [
  "[workspace]",
  'members = ["."]',
  "publish = false",
  `repository = "${repository}"`,
]) {
  if (!qualificationManifest.includes(required)) {
    throw new Error(`standalone qualification is missing ${required}`);
  }
}

console.log(
  `Keystone boundaries closed; ${metadata.packages.length} production mechanism crates, 1 explicit cross-project qualification`,
);
