# Spice patches on top of upstream Delta Kernel v0.28.0

This branch (`spiceai-0.28.0`) tracks Spice-specific patches applied on top of
the clean base branch: pristine upstream
[`v0.28.0`](https://github.com/delta-io/delta-kernel-rs/releases/tag/v0.28.0),
commit `b77d6518b2ae76bcb8796256666ad0de3f7f654b`.

## Why v0.28.0

`v0.28.0` is the newest upstream tag, and it (along with `v0.26.0` and
`v0.27.x`) already carries an `arrow_59`/`parquet_59` optional dependency pair
(`kernel/Cargo.toml`) gated behind the `arrow-59` feature, which every
workspace crate treats as its default `arrow` feature. That satisfies the
Arrow 59.x requirement for Spice's DataFusion 55.1.0 upgrade without needing a
hand-rolled Arrow bump on top of an older tag.

## Patch status

**No Spice-specific code patches are required on top of upstream v0.28.0**
beyond the Arrow 59.2.0 follow-ups below (a dependency floor, a test match,
and replacing APIs that Arrow 59.2.0 deprecates).

The two Spice patches that existed on the earlier 0.18.x fork line remain
upstreamed and present, unmodified, through `v0.23.0`, `v0.24.0`, and now
`v0.28.0` (verified by inspecting `spiceai-0.23.0` and
`lukim/spiceai-0.24.0-patches`, both of which added only this documentation
file and no code diff on top of their respective upstream tags):

| Spice patch (historical) | Status in v0.28.0 |
| --- | --- |
| Re-enable file skipping on timestamp columns | Upstreamed. `kernel/src/scan/data_skipping.rs` truncates timestamp max-stats to millisecond precision via `Scalar::Timestamp(micros.saturating_sub(999))` / `Scalar::TimestampNtz(...)`, equivalent to the Spice `timestamp_subtract(val, 999)` patch. |
| `ParquetObjectReader` Azure suffix-range handling | Upstreamed. `kernel/src/engine/default/parquet.rs` detects Azure object stores and falls back to a HEAD request + `with_file_size`, and also handles zero-size `Remove` actions (delta-io issue #968). |

## Arrow 59.2.0 bump

`kernel/Cargo.toml`'s `arrow_59`/`parquet_59` optional dependencies were
bumped from a bare `"59"` version requirement to `"59.2.0"` (still a caret
requirement, so `<60.0.0`), matching the Arrow 59.2.0 floor for Spice's
DataFusion 55.1.0 upgrade. `Cargo.lock` resolves this to `59.3.0`, the newest
released 59.x patch satisfying that requirement. No other crate in the
workspace declares a raw `arrow`/`parquet` version; every other Cargo.toml
only references the `arrow-58`/`arrow-59` *feature* names.

`cargo build --workspace --all-features` and `cargo test --workspace --no-run
--all-features` both pass unchanged against Arrow 59.3.0/Parquet 59.3.0 — no
source changes were needed to compile.

## Test fix: `log_segment::tests::read_actions_with_null_map_values` (case 10)

`cargo test --workspace` surfaced one failure not present on pristine
`v0.28.0`: `case_10_metadata_configuration_known_issue`. This is a
pre-existing delta-kernel-rs test that deliberately tracks a known upstream
Arrow issue (a non-nullable `StructArray` field surviving with an unmasked
null) via `#[should_panic(expected = "StructArray re-validation failed")]`.

Confirmed by A/B testing the *same* `cargo test --workspace` command against
both dependency states (only the arrow_59/parquet_59 lockfile version
differs):

- Arrow 59.0.0 (pristine `v0.28.0`): `test result: ok. ... 0 failed`, case 10
  panics inside the test's own `StructArray::try_new` re-validation, matching
  the expected string.
- Arrow 59.3.0 (this branch, pre-fix): `test result: FAILED. ... 1 failed`.
  Arrow's own batch iteration now rejects the null earlier, during
  `read_actions`, so the panic fires at a different call site with a
  different message: `Found unmasked nulls for non-nullable StructArray
  field "value"` instead of `StructArray re-validation failed`.

Fixed by loosening the `#[should_panic(expected = ...)]` match to the
substring `"StructArray"`, shared by both messages, in
`kernel/src/log_segment/tests.rs` — the test still asserts the known issue
panics, without pinning the exact Arrow-version-dependent wording or call
site. Verified: `cargo test -p delta_kernel --lib --features arrow-59`
reports `4030 passed; 0 failed` after the fix (previously `4029 passed; 1
failed` before it, `4030 passed; 0 failed` on pristine `v0.28.0`).

## Arrow 59.2.0 deprecations

Built against Spice's `arrow-rs` fork (`spiceai-59-patches`), the workspace
emitted deprecation warnings for APIs that Arrow 59.x retires. They are
replaced rather than silenced:

- `MutableArrayData::extend` / `extend_nulls` → `try_extend` /
  `try_extend_nulls` in `kernel/src/engine/arrow_expression/evaluate_expression.rs`
  (array construction and `coalesce`). An offset overflow now returns an error
  instead of panicking.
- `ParquetObjectReader` (whole type deprecated upstream in 59.2.0, see
  apache/arrow-rs#10308; the fork additionally deprecates `new` in favour of
  `new_with_meta`) → a private `ObjectStoreFileReader` implementing
  `AsyncFileReader` in `default-engine/src/parquet.rs`. It keeps the previous
  behaviour: bounded range reads for the footer when the file size is known,
  a suffix range request otherwise (so the Azure HEAD + file-size workaround
  still applies), and it honours `ArrowReaderOptions` via
  `ParquetMetaDataReader::with_arrow_reader_options`. Delta Kernel used none of
  the fork's reader extensions (object versioning, preload, size hints,
  runtime).
- `ParquetObjectWriter` → `object_store::buffered::BufWriter` passed directly
  to `AsyncArrowWriter` (blanket `AsyncFileWriter` impl for `AsyncWrite`).
- `acceptance/src/data.rs` and `kernel/tests/integration/golden_tables.rs`
  read their small local expected-result files into memory and use the sync
  `ParquetRecordBatchReaderBuilder`.

Verified: `cargo clippy -p delta_kernel -p delta_kernel_default_engine -p
acceptance --all-targets --all-features` reports no warnings;
`cargo test --all-features -p delta_kernel_default_engine -p acceptance`
passes (including the footer test that bounds `get_opts` calls per footer
load), as do the kernel `golden` integration and `arrow_expression` lib tests.

## Consuming this in spiceai/spiceai

```toml
delta_kernel = { git = "https://github.com/spiceai/delta-kernel-rs.git", rev = "<branch head sha>" } # branch: spiceai-0.28.0
```

If a future Spice patch becomes necessary, add it as a commit on this branch
and bump the `spiceai/spiceai` pin to the patched commit.
