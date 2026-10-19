{
  "findings": [
    {
      "file": "crates/terminus-core/src/recover.rs",
      "line": 541,
      "severity": "low",
      "claim": "When a longer twin has no watermark row, merging it leaves the survivor's old, shorter watermark in place even though the survivor now holds the longer stream.",
      "failure_scenario": "Create a valid prefix survivor of length S with watermark S and a longer twin of length L whose blocks and metadata are intact but whose watermark is absent. `one_stream` accepts the pair, `merge` replaces the survivor's blocks and metadata with the L-byte stream, but the `INSERT ... SELECT` copies zero rows and leaves watermark S untouched. The subsequent sweep only lowers overlarge watermarks, so ingest resumes at S and appends bytes S..L a second time to an archive that already contains them, corrupting the transcript stream and potentially leaving verification or derived rows inconsistent."
    }
  ]
}
