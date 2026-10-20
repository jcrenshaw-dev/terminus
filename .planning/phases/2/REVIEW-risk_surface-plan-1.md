{
  "findings": [
    {
      "file": "crates/terminus-core/src/store/open.rs",
      "line": 422,
      "severity": "medium",
      "claim": "The purportedly non-writing read path opens a WAL database without SQLite's immutable mode, so SQLite may create or update `terminus.db-shm` and `terminus.db-wal`; this contradicts the claim that read commands create and write nothing.",
      "failure_scenario": "Start with an existing WAL-mode `terminus.db`—including the no-table database handled by `holds_no_tables`—while its WAL sidecars are absent and the directory is writable, then run `status` or `verify`. Opening the database read-only can create the shared-memory file and an empty WAL file. The new tests only compare tables/schema and therefore cannot detect these filesystem writes; the diff's own uninstall comments acknowledge that a read-only status open leaves both sidecars behind."
    }
  ]
}