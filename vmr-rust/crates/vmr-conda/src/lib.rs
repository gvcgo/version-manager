//! vmr-conda: conda channel client (plan.md §3.5, requirement 2/4; does not
//! depend on a local conda/miniconda install).
//!
//! - channel: default `https://conda.anaconda.org/conda-forge`.
//! - platform: derives the subdir from os/arch (linux-64/osx-64/win-64/…arm64 variants).
//! - repodata: fetches `<channel>/<subdir>/repodata.json` (falls back to `.zst` on 404),
//!   disk-cached under `<cache>/conda_repodata/…` (expired by mtime + retention time).
//! - query: all package versions (deduplicated); records for install (name/version exact
//!   + highest build).
//! - install: stage one **single package, no recursive dependencies** (plan D4 default) —
//!   downloads `.conda` (`pkg-*.tar.zst` inside a zip container) or `.tar.bz2` and
//!   extracts it into the vmr version directory (prefix).
//! - note: the D1 decision point takes plan's **lightweight self-written** fallback branch
//!   (serde_json + hand-written zst/zip extraction), not introducing the rattler family
//!   (heavy compile burden); this module's semantics are authoritative.

pub mod platform;
pub mod repodata;

pub use platform::{PlatformParse, current_subdir, platform_for};
pub use repodata::{RepoPackage, install_package, query_packages, query_versions, select_package};
