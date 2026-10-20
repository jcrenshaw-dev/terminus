# Phase 4: Steady-state ingest pass - Measurement

Taken 2026-10-20 between 09:02 and 09:15 EDT (13:02 to 13:15 UTC) on a `sqlite3 .backup` copy of the live store. Both builds ran with no timing code added (D-01).

## Machine and cache

`free -g` just before the timed series, and again just after it:

```
               total        used        free      shared  buff/cache   available
Mem:              60          20           8           3          35          40
Mem:              60          17           6           3          40          42
```

60 GB RAM, with 35 to 40 GB in the page cache. Every number below was taken with a warm page cache, because dropping caches needs root (D-06). The copy fits in the cache several times over, so these numbers understate what a cold pass saves, since a cold pass reads its share of the store's pages from disk.

## Builds

| Build | Commit | Binary |
|---|---|---|
| Baseline | `57c036e2c8fe780d34d6975468b2fbeacf90a4fb` (`57c036e`) | `/home/john/terminus-measure-4/baseline-src/target/release/terminus` |
| Final | `fb886586895f8b1aacb99f0c801522249204a31a` (`fb88658`, PLAN-2 Task 3) | `/code/terminus/target/release/terminus` |

Both binaries report `0.1.1`. Both builds use `ARCHIVE_FORMAT` 2 and `DERIVED_SCHEMA` 5 (D-04).

```
mkdir -p /home/john/terminus-measure-4/baseline-src /home/john/terminus-measure-4/copy
git -C /code/terminus archive 57c036e | tar -x -C /home/john/terminus-measure-4/baseline-src
cd /home/john/terminus-measure-4/baseline-src && cargo build --release --locked -p terminus
cd /code/terminus && cargo build --release --locked -p terminus
```

The baseline came from `git archive` rather than `git worktree`, so the measurement leaves no git state behind.

## The copy

- Path: `/home/john/terminus-measure-4/copy/terminus.db`. It is on `/dev/nvme0n1p2`, the same disk as the live store, and not under `/tmp`, which is a tmpfs on this machine.
- Size: 3,094,274,048 bytes at the backup, and 3,134,840,832 bytes after the warm-up rebuilt the derived tables.
- The live store's session count, read off the copy right after the backup: 5,100 sessions. At that point the copy read `archive_format` 2, `derived_schema` 4 and `last_snapshot_at` 1792417196, which is 2026-10-19.
- No `terminus.toml` exists on this machine (`~/.config/terminus/` is absent and `XDG_CONFIG_HOME` is unset). Both builds therefore ran on the defaults: roots under `~/.claude` and daily snapshots. They read the live transcripts in place (D-05).
- The backup wrote the copy 0644 into a 0755 directory. Because it holds every archived transcript, both were made owner-only before anything ran on them.

```
sqlite3 -readonly ~/.local/share/terminus/terminus.db ".backup '/home/john/terminus-measure-4/copy/terminus.db'"
chmod 700 /home/john/terminus-measure-4 /home/john/terminus-measure-4/copy
chmod 600 /home/john/terminus-measure-4/copy/terminus.db
sqlite3 -readonly /home/john/terminus-measure-4/copy/terminus.db "SELECT count(*) FROM sessions; SELECT key, value FROM meta ORDER BY key; PRAGMA journal_mode;"
```

`-readonly` did not refuse `.backup`, so the fallback command D-03 allows was never needed. The `.backup` above is the only command in this record that names the live store. No terminus command from either build ran with `TERMINUS_DATA_DIR` unset or pointed at the live directory.

The copy is left at `/home/john/terminus-measure-4/copy` so UAT can re-run AC4 on it. Removing it, and `/home/john/terminus-measure-4` with it, is John's call.

## Warm-up (discarded)

```
TERMINUS_DATA_DIR=/home/john/terminus-measure-4/copy /code/terminus/target/release/terminus ingest
```

This was the final build's first pass on the copy. It brought the copy from `derived_schema` 4 to 5 and exited 0 after 111,981 ms of wall clock. Its `runs` row is id 1615: `duration_ms` 111973, `files_seen` 2910, `files_committed` 7 and `files_failed` 0. It took no snapshot, because the copied stamp was not yet 24 hours old.

The copy's stamp was then set to SQLite's current time, so that no timed pass would fall due mid-series. The value read back was 1792502023.

```
sqlite3 /home/john/terminus-measure-4/copy/terminus.db "UPDATE meta SET value = strftime('%s', 'now') WHERE key = 'last_snapshot_at'; SELECT value FROM meta WHERE key = 'last_snapshot_at';"
```

## Timed passes

The two builds took turns on the same copy (D-04), in 6 rounds. Odd rounds ran the baseline first and even rounds ran the final build first. Each timed pass ran with no tracer, and its wall clock spans the whole process, from `date +%s%N` before to after (D-02). Its `runs` row was read straight after it. Each timed pass was then followed at once by one pass of the same build under `strace -c -f`, and that pass's `runs` row was read too. No wall clock was ever taken under strace. `strace -c` stops the process on every system call, and the baseline makes four times the calls, so a traced wall clock would inflate the baseline far more than the final build.

The series ran from 09:13:52 to 09:14:10 EDT and was run as `bash series.sh`. The script, verbatim:

```
#!/usr/bin/env bash
# Timed steady-state passes, alternating builds, each followed by a strace -c pass.
set -u
M=/home/john/terminus-measure-4
C=$M/copy
OLD=$M/baseline-src/target/release/terminus
NEW=/code/terminus/target/release/terminus
OUT=$M/series
ROUNDS=${ROUNDS:-6}
mkdir -p "$OUT"

newest() {
  sqlite3 -readonly "$C/terminus.db" \
    "SELECT id, duration_ms, files_seen, files_committed, files_failed FROM runs ORDER BY id DESC LIMIT 1"
}

one() {
  local name=$1 bin=$2 i=$3 s e rc
  s=$(date +%s%N)
  TERMINUS_DATA_DIR=$C "$bin" ingest >"$OUT/$name-$i.out" 2>&1
  rc=$?
  e=$(date +%s%N)
  echo "$name timed $i rc=$rc wall_ms=$(( (e - s) / 1000000 )) run=$(newest)"
  TERMINUS_DATA_DIR=$C strace -c -f -o "$OUT/$name-$i.strace" "$bin" ingest >"$OUT/$name-$i.traced.out" 2>&1
  rc=$?
  echo "$name traced $i rc=$rc run=$(newest)"
}

for i in $(seq 1 "$ROUNDS"); do
  if (( i % 2 )); then
    one old "$OLD" "$i"; one new "$NEW" "$i"
  else
    one new "$NEW" "$i"; one old "$OLD" "$i"
  fi
done
```

Every pass, timed or traced, exited 0. The raw log is `/home/john/terminus-measure-4/series.log`, and each `strace -c` summary is `/home/john/terminus-measure-4/series/<old|new>-<n>.strace`.

### Baseline, `57c036e`

| Pass | Wall ms | `duration_ms` | `files_seen` | `files_committed` | `files_failed` | Traced pass `files_committed` | strace calls | strace seconds |
|---|---|---|---|---|---|---|---|---|
| 1 | 355 | 345 | 2910 | 3 | 0 | 0 | 372,448 | 0.225995 |
| 2 | 316 | 309 | 2910 | 0 | 0 | 0 | 372,448 | 0.234473 |
| 3 | 316 | 308 | 2910 | 0 | 0 | 0 | 372,448 | 0.226412 |
| 4 | 320 | 312 | 2910 | 0 | 0 | 0 | 372,448 | 0.228115 |
| 5 | 321 | 311 | 2910 | 0 | 0 | 0 | 372,448 | 0.229591 |
| 6 | 316 | 308 | 2910 | 0 | 0 | 0 | 372,448 | 0.229813 |
| Median | 318 | 310 | | | | | 372,448 | 0.228853 |
| Mean | 324.0 | 315.5 | | | | | 372,448 | 0.229067 |

The `runs` rows for the timed passes are ids 1616, 1622, 1624, 1630, 1632 and 1638, and for the traced passes 1617, 1623, 1625, 1631, 1633 and 1639. The traced passes' `duration_ms` were 1822, 1898, 2015, 1981, 2015 and 1986.

The per-call lines were identical in all six summaries:

| Syscall | Calls | Errors |
|---|---|---|
| `openat` | 4,484 | 12 |
| `statx` | 11,644 | 8 |
| `newfstatat` | 70 | 7 |
| `fstat` | 21,946 | |
| `lseek` | 5,802 | |
| `read` | 2,908 | |
| `pread64` | 276,619 | |
| `pwrite64` | 52 | |
| `fsync` | 4 | |
| `fdatasync` | no line | |
| `close` | 4,472 | |

### Final, `fb88658`

| Pass | Wall ms | `duration_ms` | `files_seen` | `files_committed` | `files_failed` | Traced pass `files_committed` | strace calls | strace seconds |
|---|---|---|---|---|---|---|---|---|
| 1 | 163 | 156 | 2910 | 0 | 0 | 0 | 92,756 | 0.049851 |
| 2 | 164 | 156 | 2910 | 0 | 0 | 0 | 92,757 | 0.050160 |
| 3 | 167 | 159 | 2910 | 0 | 0 | 0 | 92,756 | 0.054832 |
| 4 | 167 | 159 | 2910 | 0 | 0 | 0 | 92,757 | 0.059363 |
| 5 | 165 | 158 | 2910 | 0 | 0 | 0 | 92,756 | 0.045541 |
| 6 | 164 | 156 | 2910 | 0 | 0 | 0 | 92,756 | 0.045386 |
| Median | 164.5 | 157 | | | | | 92,756 | 0.050006 |
| Mean | 165.0 | 157.3 | | | | | 92,756.3 | 0.050855 |

The `runs` rows for the timed passes are ids 1618, 1620, 1626, 1628, 1634 and 1636, and for the traced passes 1619, 1621, 1627, 1629, 1635 and 1637. The traced passes' `duration_ms` were 531, 530, 591, 611, 512 and 527.

The per-call lines were identical in all six summaries; only the total moved, by one call:

| Syscall | Calls | Errors |
|---|---|---|
| `openat` | 4,471 | 8 |
| `statx` | 14,533 | 3 |
| `newfstatat` | 37 | 3 |
| `fstat` | 1,605 | |
| `lseek` | 5,802 | |
| `read` | 2,908 | |
| `pread64` | 55,163 | |
| `pwrite64` | 28 | |
| `fsync` | 4 | |
| `fdatasync` | no line | |
| `close` | 4,463 | |

### What the two tables say

On this copy, with a warm cache and 2,910 files walked, the median steady-state pass fell from 318 ms to 164.5 ms of wall clock and from 310 ms to 157 ms of `duration_ms`. Total system calls fell from 372,448 to about 92,756. Most of that is `pread64`, which fell from 276,619 to 55,163, and `fstat`, which fell from 21,946 to 1,605. The `pread64` drop is consistent with unchanged files no longer reading their block rows (D-08), but this record did not trace which statements issued the reads. `statx` rose by 2,889, from 11,644 to 14,533. That is close to the 2,910 files walked, and it matches the one length check per file that D-08 added. `read` stayed at 2,908 and `lseek` at 5,802 in both builds.

Wall clock minus `duration_ms` is 7 to 10 ms for the baseline and 7 to 8 ms for the final build. That gap covers process start, the config load and, for the baseline, the second store open. On this copy the second open costs too little to separate from the rest of that gap by timing alone. The AC4 trace below is what shows it is gone.

### Caveats

- `files_committed` varies because the transcripts are live: baseline pass 1 committed 3 files and every other pass committed 0. Hook-triggered passes on the live store ran beside the series with the installed binary, and the copy never saw them.
- A session that goes idle during the series has its one observation read in whichever pass marks it final, which can land in either build's column. The plan records that the live store had 0 sessions waiting for observation on 2026-10-20. This series did not re-check that on the copy.
- All numbers are warm-cache numbers (D-06).

## AC4 on the copy

### No snapshot due

The stamp was still the 1792502023 written after the warm-up. Each build ran once under `strace -f -e trace=openat`. The count rule is the one Task 2 used: the exact path `copy/terminus.db"` (not `-wal` or `-shm`), with `O_RDWR`, and a return value that is not `-1`.

```
TERMINUS_DATA_DIR=/home/john/terminus-measure-4/copy strace -f -e trace=openat -o /home/john/terminus-measure-4/ac4-old.openat /home/john/terminus-measure-4/baseline-src/target/release/terminus ingest
TERMINUS_DATA_DIR=/home/john/terminus-measure-4/copy strace -f -e trace=openat -o /home/john/terminus-measure-4/ac4-new.openat /code/terminus/target/release/terminus ingest
grep '"/home/john/terminus-measure-4/copy/terminus.db"' /home/john/terminus-measure-4/ac4-<old|new>.openat | grep O_RDWR | grep -vc '= -1'
```

| Build | Successful `O_RDWR` opens of `terminus.db` |
|---|---|
| Baseline `57c036e` | 2 |
| Final `fb88658` | 1 |

The baseline's opens of the file, in order: a failed `O_WRONLY|O_CREAT|O_EXCL`, two `O_RDONLY`, then `O_RDWR|O_CREAT`. After the pass, `roll`'s own `Store::open` repeated the same four. The final build made the first four only.

### Snapshot due

```
sqlite3 /home/john/terminus-measure-4/copy/terminus.db "UPDATE meta SET value = '0' WHERE key = 'last_snapshot_at'; SELECT value FROM meta WHERE key = 'last_snapshot_at';"
TERMINUS_DATA_DIR=/home/john/terminus-measure-4/copy /code/terminus/target/release/terminus ingest
ls -la /home/john/terminus-measure-4/copy/snapshots
sqlite3 -readonly /home/john/terminus-measure-4/copy/terminus.db "SELECT value FROM meta WHERE key = 'last_snapshot_at'; SELECT strftime('%s','now');"
```

Before the run, `copy/snapshots` did not exist. The final build exited 0 after 1,199 ms of wall clock and left one new file, `/home/john/terminus-measure-4/copy/snapshots/terminus-20261020T131437.656Z.db` (1,638,612,992 bytes). The stamp afterwards read 1792502077, one second before SQLite's `now` of 1792502078. `tests/snapshot.rs` is what proves the stamp is written only after the copy lands.
