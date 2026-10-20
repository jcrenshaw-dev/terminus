# Changelog

All notable changes to Terminus are recorded here, starting with 1.0.0. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Each release bumps the middle number of the version, a release with only fixes
bumps the last, and the first moves only for a large change. The number does not
promise compatibility; the entry for each release says what it changes.

## [1.0.0] - Unreleased

The first release of Terminus as a product. It keeps every Claude Code session
you run and gives that history back to Claude and to you.

### What it does

- Archives every session and every subagent's transcript, byte for byte under
  the default capture mode, in one local SQLite file readable only by your user.
- Starts each session with a brief of where the last session in the project left
  off.
- Hands Claude earlier turns when a prompt names the same paths or identifiers,
  and after a compaction, the turns the compaction dropped.
- Gives Claude three MCP tools, `recall_search`, `recall_context` and
  `recall_get`, to search the whole history and read it back.
- Searches from the command line with `terminus search`, `show`, `sessions` and
  `observations`.
- Checks and rebuilds itself with `doctor`, `verify`, `reindex` and rolling
  snapshots.
- Installs with one line that checks the release's signature before it installs
  anything, and updates by running the same line again.

It calls no model, runs no daemon and opens no network connection.

[1.0.0]: https://github.com/jcrenshaw-dev/terminus/releases/tag/v1.0.0
