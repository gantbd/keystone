#!/usr/bin/env node

import fs from "node:fs";

const baselinePath = process.argv[2];
if (!baselinePath) {
  throw new Error("usage: check-cargo-license-baseline.mjs <baseline>");
}

let input = "";
for await (const chunk of process.stdin) {
  input += chunk;
}

const metadata = JSON.parse(input);
const actual = metadata.packages
  .map((pkg) => {
    if (!pkg.license) {
      throw new Error(`package has no declared license: ${pkg.name} ${pkg.version}`);
    }
    return `license|${pkg.name}|${pkg.version}|${pkg.license}`;
  })
  .sort();
const expected = fs
  .readFileSync(baselinePath, "utf8")
  .split(/\r?\n/u)
  .filter((line) => line.startsWith("license|"))
  .sort();

if (expected.length === 0) {
  throw new Error(`license baseline is empty: ${baselinePath}`);
}
if (actual.length !== expected.length || actual.some((line, index) => line !== expected[index])) {
  process.stderr.write(`license declaration baseline drifted: ${baselinePath}\n`);
  process.stderr.write(`expected:\n${expected.join("\n")}\nactual:\n${actual.join("\n")}\n`);
  process.exit(1);
}
