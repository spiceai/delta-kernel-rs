# Spice patches on top of upstream Delta Kernel v0.23.0

This branch (`spiceai-0.23.0-patches`) tracks Spice-specific patches applied on
top of the clean base branch `spiceai-0.23.0` (pristine upstream
[`v0.23.0`](https://github.com/delta-io/delta-kernel-rs/releases/tag/v0.23.0),
commit `6003e3fb6b1f09d5e5233d5cbc6e08d12c2369a4`).

## Patch status

**No Spice-specific code patches are required on top of upstream v0.23.0.**

Both Spice patches that existed on the earlier 0.18.x fork line have been
upstreamed into Delta Kernel and are present, unmodified, in v0.23.0:

| Spice patch (historical) | Status in v0.23.0 |
| --- | --- |
| Re-enable file skipping on timestamp columns | Upstreamed. `kernel/src/scan/data_skipping.rs` truncates timestamp max-stats to millisecond precision via `Scalar::Timestamp(micros.saturating_sub(999))` / `Scalar::TimestampNtz(...)`, equivalent to the Spice `timestamp_subtract(val, 999)` patch. |
| `ParquetObjectReader` Azure suffix-range handling | Upstreamed. `kernel/src/engine/default/parquet.rs` detects Azure object stores and falls back to a HEAD request + `with_file_size`, and also handles zero-size `Remove` actions (delta-io issue #968). |

The historical patches additionally depended on the `spiceai/arrow-rs` fork's
`new_with_meta()` API. That dependency is no longer needed: v0.23.0 builds
against standard upstream `arrow` (58.x), which is what `spice2` resolves to.

## Consuming this in spice2

`spice2` pins the clean base commit directly:

```toml
delta_kernel = { git = "https://github.com/spiceai/delta-kernel-rs.git", rev = "6003e3fb6b1f09d5e5233d5cbc6e08d12c2369a4" } # branch: spiceai-0.23.0
```

Because there are no extra patches, this pin is byte-identical to upstream
`v0.23.0` and is correct. If a future Spice patch becomes necessary, add it as a
commit on this branch and bump the spice2 pin to the patched commit.
