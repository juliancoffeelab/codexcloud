# Native simulation records and recovery

The final `.json` report remains schema 3: compact outcomes appear in canonical
schedule order and retain checksummed byte references into `.details.jsonl`.
Complete phases, actions, combats, ownership, objectives and endpoint states
remain in that detail journal. `--trace` also writes raw typed match results to
`.trace.jsonl`.

During execution, each completed compact outcome is appended once to
`.outcomes.jsonl` as `{"index": <schedule position>, "outcome": <compact outcome>}`.
Lines arrive in worker completion order. The zero-based `index` restores canonical
schedule order independently of completion timing.

`.partial.json` is a small atomic progress snapshot, marked with
`"progressFormat": "indexed-outcomes-jsonl-v1"`. It contains status, completed and
expected counts, elapsed time, fingerprint, and the filenames and complete byte
lengths of the outcome index and detail journals. It does not copy accumulated
outcomes. Progress is refreshed every ten completions and on completion or error;
the append-only index may contain additional completed lines after the latest
progress snapshot.

To recover completed results, parse complete newline-terminated index records and
place each `outcome` at its recorded `index`. Ignore an interrupted trailing line.
`checkpoint::recover_outcomes` performs this operation and rejects duplicate or
out-of-range indices. Missing positions remain unresolved work. Every recovered
outcome retains its full-detail byte range and SHA-256 digest; `records::read_detail`
validates that range and digest before reading it. Recovery exposes completed work;
it does not automatically resume or rerun a batch.

Detailed summaries borrow full trace arrays directly from the typed match result.
Only small composition summaries are allocated before detail serialization.
Comparison summaries omit state hashes entirely, while preserving all actual
states, records, array order, and missing-versus-null distinctions.
