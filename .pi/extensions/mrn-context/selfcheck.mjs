// Self-check for the porcelain-v2 parser in mrn-context.ts. Runnable via:
//   node --experimental-strip-types .pi/extensions/mrn-context.selfcheck.mjs
// No test framework, no fixtures — five cases, one assert each.

import { parseGitPorcelainV2 } from "./index.ts";

const cases = [
  {
    name: "clean repo",
    input: [
      "# branch.oid abc1234567890abcdef1234567890abcdef123456",
      "# branch.head main",
      "# branch.upstream origin/main",
      "# branch.ab +0 -0",
    ].join("\n"),
    expected: "Git: branch main, at abc12345, 0 uncommitted changes",
  },
  {
    name: "dirty (mixed 1/2/u)",
    input: [
      "# branch.oid def56789012345678901234567890abcdef123456",
      "# branch.head feature/foo",
      "# branch.upstream origin/feature/foo",
      "# branch.ab +0 -0",
      '1 .M N... 100644 100644 100644 aaaaaaaa aaaaaaaa file1.txt',
      '2 R. N... 100644 100644 100644 aaaaaaaa aaaaaaaa file2.txt\0file2-renamed.txt',
      'u UU N... 100644 100644 100644 aaaaaaaa aaaaaaaa file3.txt',
    ].join("\n"),
    expected: "Git: branch feature/foo, at def56789, 3 uncommitted changes",
  },
  {
    name: "detached HEAD",
    input: [
      "# branch.oid 1111111111111111111111111111111111111111",
      "# branch.head HEAD",
    ].join("\n"),
    expected: "Git: at 11111111, 0 uncommitted changes",
  },
  {
    name: "unborn branch",
    input: [
      "# branch.oid 0000000000000000000000000000000000000000",
      "# branch.head (unborn)",
    ].join("\n"),
    expected: "Git: at 00000000, 0 uncommitted changes",
  },
  {
    name: "no upstream",
    input: [
      "# branch.oid 5555555555555555555555555555555555555555",
      "# branch.head main",
    ].join("\n"),
    expected: "Git: branch main, at 55555555, 0 uncommitted changes",
  },
];

let passed = 0;
for (const c of cases) {
  const actual = parseGitPorcelainV2(c.input);
  if (actual === c.expected) {
    passed += 1;
    console.log(`PASS: ${c.name}`);
  } else {
    console.log(`FAIL: ${c.name}`);
    console.log(`  expected: ${c.expected}`);
    console.log(`  actual:   ${actual}`);
  }
}
console.log(`\n${passed}/${cases.length} passed`);
process.exit(passed === cases.length ? 0 : 1);