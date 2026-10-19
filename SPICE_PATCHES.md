# Spice patches on top of upstream Delta Kernel v0.29.0

`spiceai-0.29` is Spice's fork line for upstream Delta Kernel 0.29.x. It is the previous fork
line (`spiceai-0.27`) with pristine upstream
[`v0.29.0`](https://github.com/delta-io/delta-kernel-rs/tree/v0.29.0) (commit
`5a33384df82bb2c5ae4ca3c43d0d0a46a739cc9d`) merged into it. Apart from this file and the CI-only
change below, the tree is identical to upstream v0.29.0: `git diff --stat v0.29.0 spiceai-0.29`
lists only `.github/workflows/build.yml` and `SPICE_PATCHES.md`.

## Patch status

**No Spice-specific code patches are required on top of upstream v0.29.0.**

Every code patch from earlier fork lines is covered by an upstream change. Upstream v0.21.0 is the
first release that contains all of them.

| Fork PR | Upstream change | First upstream release |
| --- | --- | --- |
| [#1](https://github.com/spiceai/delta-kernel-rs/pull/1) Add `InvalidCheckpoint` error type | [delta-io/delta-kernel-rs#593](https://github.com/delta-io/delta-kernel-rs/pull/593), the same change | v0.6.0 |
| [#2](https://github.com/spiceai/delta-kernel-rs/pull/2) Re-enable file skipping on timestamp columns, and [#6](https://github.com/spiceai/delta-kernel-rs/pull/6) / [#7](https://github.com/spiceai/delta-kernel-rs/pull/7) `>=` timestamp predicates | [delta-io/delta-kernel-rs#2249](https://github.com/delta-io/delta-kernel-rs/pull/2249), an independent implementation: it subtracts 999µs from the literal compared with `maxValues` and covers `>=`. It supersedes Spice's upstream PR [delta-io/delta-kernel-rs#1333](https://github.com/delta-io/delta-kernel-rs/pull/1333) | v0.21.0 |
| [#3](https://github.com/spiceai/delta-kernel-rs/pull/3) Pass the file size to `ParquetObjectReader`, avoiding suffix-range requests | [delta-io/delta-kernel-rs#1006](https://github.com/delta-io/delta-kernel-rs/pull/1006) (Azure HEAD fallback) and [delta-io/delta-kernel-rs#1956](https://github.com/delta-io/delta-kernel-rs/pull/1956) (the reader takes the size from file metadata via `with_file_size`), both in `default-engine/src/parquet.rs` | v0.12.1, v0.21.0 |

## CI-only changes

`.github/workflows/build.yml` makes the Codecov upload best-effort (`continue-on-error: true`,
`fail_ci_if_error: false`), because the fork has no `CODECOV_TOKEN` and Codecov rejects tokenless
uploads. It does not change behavior.

## Consuming this in Spice

The default engine is its own crate, and both crates are patched to the fork. Pin the
`spiceai-0.29` tip:

```toml
[workspace.dependencies]
delta_kernel = { version = "0.29.0", features = ["arrow-59", "internal-api"] }
delta_kernel_default_engine = { version = "0.29.0", default-features = false, features = ["arrow-59", "rustls"] }

[patch.crates-io]
delta_kernel = { git = "https://github.com/spiceai/delta-kernel-rs.git", rev = "<spiceai-0.29 tip>" } # branch: spiceai-0.29
delta_kernel_default_engine = { git = "https://github.com/spiceai/delta-kernel-rs.git", rev = "<spiceai-0.29 tip>" } # branch: spiceai-0.29
```

If a Spice patch becomes necessary, land it on `spiceai-0.29` through a `spiceai-0.29-patches` PR,
add a row to the table above, and record the Spice test that fails without it in Spice's
`docs/dev/fork_patches.md`.
