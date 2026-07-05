# Spice patches on top of upstream Delta Kernel v0.24.0

This branch (`spiceai-0.24.0-patches`) tracks Spice-specific patches applied on
top of the clean base branch `spiceai-0.24.0` (pristine upstream
[`v0.24.0`](https://github.com/delta-io/delta-kernel-rs/releases/tag/v0.24.0),
commit `73f47a6384c10cbf6c9cba8ae14263124c038665`).

## Patch status

**No Spice-specific code patches are required on top of upstream v0.24.0.**

Both Spice patches that existed on the earlier 0.18.x fork line remain
upstreamed into Delta Kernel and are present, unmodified, in v0.24.0:

| Spice patch (historical) | Status in v0.24.0 |
| --- | --- |
| Re-enable file skipping on timestamp columns | Upstreamed. `kernel/src/scan/data_skipping.rs` truncates timestamp max-stats to millisecond precision via `Scalar::Timestamp(micros.saturating_sub(999))` / `Scalar::TimestampNtz(...)`, equivalent to the Spice `timestamp_subtract(val, 999)` patch. |
| `ParquetObjectReader` Azure suffix-range handling | Upstreamed. `kernel/src/engine/default/parquet.rs` detects Azure object stores and falls back to a HEAD request + `with_file_size`, and also handles zero-size `Remove` actions (delta-io issue #968). |

The historical patches additionally depended on the `spiceai/arrow-rs` fork's
`new_with_meta()` API. That dependency is no longer needed: v0.24.0 still
defaults to standard upstream `arrow` (58.x, the `arrow-58` feature), which is
what `spice2` resolves to.

## Upstream API changes in v0.24.0 (consumer notes)

Unlike the byte-clean 0.23.0 bump, upstream v0.24.0 carries several breaking API
changes. None affect the (zero) fork patches above, but `spice2` code that
consumes the kernel may need to adapt. See the
[v0.24.0 release notes](https://github.com/delta-io/delta-kernel-rs/releases/tag/v0.24.0)
for the full list; notable ones include:

- `metrics` module reorganized: `MetricEvent` variants now wrap a per-event
  struct (e.g. `MetricEvent::ScanMetadataCompleted(ScanMetadataCompleted { .. })`);
  new `CrcReadCompleted` variant; `ScanMetadataCompleted` gains
  `active_add_files_bytes`. Exhaustive matches on `MetricEvent` must be updated.
- `plans::Plan` enum renamed to `plans::Operation` (`declarative-plans` feature).
- CRC typed enums: `domain_metadata` -> `domain_metadata_state`
  (`DomainMetadataState`) and `set_transactions` -> `set_transaction_state`
  (`SetTransactionState`); only public under the `test-utils` feature.
- `variantShredding` is now a supported table feature (previously only
  `variantShredding-preview`).

## Consuming this in spice2

`spice2` pins the clean base commit directly:

```toml
delta_kernel = { git = "https://github.com/spiceai/delta-kernel-rs.git", rev = "73f47a6384c10cbf6c9cba8ae14263124c038665" } # branch: spiceai-0.24.0
```

Because there are no extra patches, this pin is byte-identical to upstream
`v0.24.0` and is correct. If a future Spice patch becomes necessary, add it as a
commit on this branch and bump the spice2 pin to the patched commit.
