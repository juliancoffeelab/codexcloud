import { test } from "vitest";
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { ResultJournal, readDetail } from "../scripts/ai-records";

test("Journal byte references recover Unicode records independently", () => {
  const directory = mkdtempSync(join(tmpdir(), "hex-ai-records-"));
  try {
    const journal = new ResultJournal(join(directory, "results.jsonl"));
    const first = { speaker: "Mira", text: "High tide — protect the workers 👋", round: 6 };
    const second = { commands: [{ kind: "move", tile: "5,4" }], winner: null };
    const a = journal.append(first);
    const b = journal.append(second);
    journal.close();
    assert.equal(b.offset, a.bytes);
    assert.deepEqual(readDetail(directory, b), second);
    assert.deepEqual(readDetail(directory, a), first);
    const bytes = readFileSync(join(directory, b.file));
    writeFileSync(join(directory, b.file), bytes.subarray(0, bytes.length - 2));
    assert.throws(() => readDetail(directory, b), /Incomplete result/);
    assert.deepEqual(
      readDetail(directory, a),
      first,
      "A later interruption preserves earlier results",
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("Journal records detect corruption and reject references outside the report directory", () => {
  const directory = mkdtempSync(join(tmpdir(), "hex-ai-records-"));
  try {
    const journal = new ResultJournal(join(directory, "results.jsonl"));
    const record = journal.append({ rounds: 12 });
    journal.close();
    writeFileSync(join(directory, record.file), '{"rounds":13}\n');
    assert.throws(() => readDetail(directory, record), /checksum mismatch/);
    assert.throws(
      () => readDetail(directory, { ...record, file: "../results.jsonl" }),
      /Invalid result/,
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
