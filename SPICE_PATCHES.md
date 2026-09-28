# Spice patches on top of upstream Delta Kernel v0.27.1

`spiceai-0.27` is Spice's fork line for upstream Delta Kernel 0.27.x. It is pristine upstream
[`v0.27.1`](https://github.com/delta-io/delta-kernel-rs/tree/v0.27.1) (commit
`728aeb966be250d0520df7aad873818e809e77f2`, tagged `upstream-v0.27.1` in this repo) merged into
the previous fork line (`spiceai-0.23.0`). Apart from this file and two backported upstream CI
fixes (below), the tree is identical to upstream v0.27.1.

## Patch status

**No Spice-specific code patches are required on top of upstream v0.27.1.**

Both Spice patches from the earlier 0.18.x fork line remain upstreamed and are present in
v0.27.1:

| Spice patch (historical) | Status in v0.27.1 |
| --- | --- |
| Re-enable file skipping on timestamp columns | Upstreamed. `kernel/src/scan/data_skipping.rs` truncates timestamp max-stats to millisecond precision via `Scalar::Timestamp(micros.saturating_sub(999))` / `Scalar::TimestampNtz(...)`, equivalent to the Spice `timestamp_subtract(val, 999)` patch. |
| `ParquetObjectReader` Azure suffix-range handling | Upstreamed. Moved with the default engine to `default-engine/src/parquet.rs` (crate `delta_kernel_default_engine`), logic unchanged: it detects Azure object stores and falls back to a HEAD request + `with_file_size`, and also handles zero-size `Remove` actions (delta-io issue #968). |

The historical patches also depended on the `spiceai/arrow-rs` fork's `new_with_meta()` API.
That dependency is no longer needed: v0.27.1 builds against standard upstream `arrow`. Arrow 58
is still supported but is no longer the default (see below).

## Backported upstream CI fixes

The fork's CI runs the latest stable and nightly toolchains, and Rust 1.98 plus current nightly
rustfmt postdate v0.27.1. Two upstream commits from after v0.27.1 are cherry-picked so CI passes.
Neither changes behavior, and both ship in upstream v0.28.0, so the next upstream merge absorbs
them:

- `d265e8f9` chore: fix new warnings from clippy upgrade (#3164). `chunks_exact(2)` ->
  `as_chunks::<2>()` in `kernel/src/expressions/sql.rs`, and three unused test imports in `ffi`.
- `7316cb6e` chore: apply current nightly rustfmt (#3218). Comment re-wrapping only.

## Upstream API changes since v0.23.0 (consumer notes)

`spice2` currently pins the 0.23.0 fork line, so this bump spans upstream v0.24.0, v0.25.0,
v0.26.0, and v0.27.1. See the
[CHANGELOG at v0.27.1](https://github.com/delta-io/delta-kernel-rs/blob/v0.27.1/CHANGELOG.md)
for the full list (the v0.27.1 entry supersedes v0.27.0's). None of them affect this fork, but
three break `spice2`:

- **The default engine is its own crate** (v0.25.0, #2397). `delta_kernel::engine::default::*`
  moves to `delta_kernel_default_engine::*` with the same module paths (`DefaultEngine`,
  `executor::tokio::TokioBackgroundExecutor`, `storage::store_from_url_opts`). The kernel's
  `default-engine-rustls` / `default-engine-native-tls` features are gone; the TLS backend is
  picked with the new crate's `rustls` / `native-tls` features.
- **The default Arrow is now 59** (v0.26.0, #2847). The `arrow` feature resolves to `arrow-59`,
  and `delta_kernel_default_engine` enables it by default. `spice2` is on Arrow 58, so it must
  disable the default engine's default features and select `arrow-58` explicitly (see below).
  `arrow-57` is removed.
- **New `PrimitiveType` variants**: `Void` (v0.25.0), `IntervalYearMonth` and `IntervalDayTime`
  (v0.26.0). The exhaustive match in `map_delta_data_type_to_arrow_data_type`
  (`crates/data_components/src/delta_lake.rs`) needs arms for them. `Geometry` / `Geography`
  exist only behind the `geo-type-in-dev` feature.

Other notable breaking changes, for code that grows into these APIs:

- `MetricEvent` variants wrap per-event structs (v0.24.0) and are split into success/failure
  pairs carrying `table_type` / `correlation_id` (v0.25.0).
- `ScanBuilder::with_stats_columns` is replaced by `with_stats(StatsOptions)`, and
  `Snapshot::create_checkpoint_writer` takes an engine (v0.25.0).
- `StorageHandler` gains a required `delete`, and the default handlers' `with_buffer_size` /
  `with_batch_size` take `NonZero<usize>` (v0.26.0).
- `Expression` gains `Cast` and `Scalar` gains interval variants, so exhaustive matches on
  either break; writes to tables with column defaults require
  `Transaction::ack_column_defaults()`; add-file partition values are validated at commit
  (v0.27.1).

## Consuming this in spice2

The default engine needs its own dependency, and both crates are patched to the fork. Pin the
pristine v0.27.1 commit, which is reachable from `spiceai-0.27` and kept alive by the
`upstream-v0.27.1` tag:

```toml
[workspace.dependencies]
delta_kernel = { version = "0.27.1", features = ["arrow-58", "internal-api"] }
delta_kernel_default_engine = { version = "0.27.1", default-features = false, features = ["arrow-58", "rustls"] }

[patch.crates-io]
delta_kernel = { git = "https://github.com/spiceai/delta-kernel-rs.git", rev = "728aeb966be250d0520df7aad873818e809e77f2" } # branch: spiceai-0.27
delta_kernel_default_engine = { git = "https://github.com/spiceai/delta-kernel-rs.git", rev = "728aeb966be250d0520df7aad873818e809e77f2" } # branch: spiceai-0.27
```

The backported CI fixes don't change behavior, so pinning the pristine upstream commit is
correct. If a future Spice patch becomes necessary, add it as a commit on `spiceai-0.27` and bump
the `spice2` pin to the patched commit.
