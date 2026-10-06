import { createHash } from "node:crypto";
import { closeSync, openSync, readSync, writeFileSync } from "node:fs";
import { basename, resolve } from "node:path";

export interface DetailRecord {
  file: string;
  offset: number;
  bytes: number;
  sha256: string;
}

/** Append each full result once; retain only small indexes while simulating. */
export class ResultJournal {
  private readonly descriptor: number;
  private offset = 0;
  readonly file: string;
  constructor(path: string) {
    this.file = basename(path);
    this.descriptor = openSync(path, "w");
  }
  append(value: unknown): DetailRecord {
    const bytes = Buffer.from(JSON.stringify(value) + "\n");
    writeFileSync(this.descriptor, bytes);
    const record = {
      file: this.file,
      offset: this.offset,
      bytes: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
    };
    this.offset += bytes.length;
    return record;
  }
  close(): void {
    closeSync(this.descriptor);
  }
}

/** Read one match without loading multi-gigabyte tournament traces into memory. */
export function readDetail<T>(directory: string, record: DetailRecord): T {
  if (
    basename(record.file) !== record.file ||
    !Number.isSafeInteger(record.offset) ||
    record.offset < 0 ||
    !Number.isSafeInteger(record.bytes) ||
    record.bytes < 1
  )
    throw new Error("Invalid result journal reference");
  const descriptor = openSync(resolve(directory, record.file), "r");
  try {
    const bytes = Buffer.alloc(record.bytes);
    let read = 0;
    while (read < bytes.length) {
      const count = readSync(descriptor, bytes, read, bytes.length - read, record.offset + read);
      if (!count) throw new Error(`Incomplete result journal record: ${record.file}`);
      read += count;
    }
    if (createHash("sha256").update(bytes).digest("hex") !== record.sha256)
      throw new Error(`Result journal checksum mismatch: ${record.file}`);
    return JSON.parse(bytes.toString("utf8")) as T;
  } finally {
    closeSync(descriptor);
  }
}
