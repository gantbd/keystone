import { execFileSync } from "node:child_process";
import { readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, relative, resolve } from "node:path";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repository = "https://github.com/gantbd/keystone";
const expected = new Set([
  "crates/keystone-bounds",
  "crates/keystone-codec",
  "crates/keystone-effects",
  "crates/keystone-framing",
  "crates/keystone-lifecycle",
  "crates/keystone-observe",
  "crates/keystone-redact",
  "crates/keystone-replay",
  "crates/keystone-storage",
  "crates/keystone-storage-directory",
  "crates/keystone-storage-memory",
  "crates/keystone-testkit",
  "crates/keystone-unix-listener",
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

const qualificationStatus = JSON.parse(
  readFileSync(resolve(root, "qualification-status.json"), "utf8"),
);
if (qualificationStatus.schema !== 2) {
  throw new Error("qualification status must use profile schema 2");
}
const allowedStatuses = new Set(["qualified", "adopted", "reference"]);
const profileName = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
const packageNames = new Set(metadata.packages.map((pkg) => pkg.name));
const classifiedNames = new Set(Object.keys(qualificationStatus.crates ?? {}));
const unclassified = [...packageNames].filter((name) => !classifiedNames.has(name));
const stale = [...classifiedNames].filter((name) => !packageNames.has(name));
if (unclassified.length || stale.length) {
  throw new Error(
    `qualification classification mismatch; unclassified=${unclassified.join(",") || "none"}; stale=${stale.join(",") || "none"}`,
  );
}
const counts = { qualified: 0, adopted: 0, reference: 0 };
for (const [name, classification] of Object.entries(qualificationStatus.crates)) {
  if (
    !classification ||
    typeof classification !== "object" ||
    Array.isArray(classification) ||
    Object.hasOwn(classification, "status")
  ) {
    throw new Error(`${name} must classify named profiles, not the whole crate`);
  }
  const profiles = classification.profiles;
  if (!profiles || typeof profiles !== "object" || Array.isArray(profiles)) {
    throw new Error(`${name} must declare a profiles object`);
  }
  const entries = Object.entries(profiles);
  if (entries.length === 0) {
    throw new Error(`${name} must classify at least one named profile`);
  }
  for (const [profile, claim] of entries) {
    if (!profileName.test(profile)) {
      throw new Error(`${name} has invalid profile name ${profile}`);
    }
    if (!claim || typeof claim !== "object" || Array.isArray(claim)) {
      throw new Error(`${name}/${profile} must be a claim object`);
    }
    if (!allowedStatuses.has(claim.status)) {
      throw new Error(`${name}/${profile} has unknown status ${claim.status}`);
    }
    if (!Array.isArray(claim.evidence) || claim.evidence.length === 0) {
      throw new Error(`${name}/${profile} must cite repository evidence`);
    }
    for (const evidence of claim.evidence) {
      if (
        typeof evidence !== "string" ||
        evidence.startsWith("/") ||
        evidence.split("/").includes("..")
      ) {
        throw new Error(`${name}/${profile} has invalid evidence path ${evidence}`);
      }
      let evidenceFile;
      try {
        evidenceFile = statSync(resolve(root, evidence));
      } catch {
        throw new Error(`${name}/${profile} cites missing evidence ${evidence}`);
      }
      if (!evidenceFile.isFile()) {
        throw new Error(`${name}/${profile} evidence is not a file: ${evidence}`);
      }
    }
    counts[claim.status] += 1;
  }
}

const storage = metadata.packages.find((pkg) => pkg.name === "keystone-storage");
const storageFeatures = storage?.features ?? {};
if (!Array.isArray(storageFeatures.default) || storageFeatures.default.length !== 0) {
  throw new Error("keystone-storage default features must remain empty");
}
if (!Object.hasOwn(storageFeatures, "incubating-append-blob")) {
  throw new Error("keystone-storage must isolate append/blob candidates behind an explicit feature");
}
const appendBlob = qualificationStatus.crates["keystone-storage"]?.profiles?.[
  "append-and-immutable-blob-contracts"
];
if (appendBlob?.status !== "reference") {
  throw new Error("incubating append/blob contracts must remain a reference profile");
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
  `Keystone boundaries closed; ${metadata.packages.length} mechanism crates with ${counts.qualified + counts.adopted + counts.reference} named profiles (${counts.qualified} qualified, ${counts.adopted} adopted, ${counts.reference} reference), 1 explicit cross-project qualification`,
);
