//! vmr-pixi: pixi/rattler-based package backend for VMR.
//!
//! VMR no longer shells out to a local conda/miniconda installation for SDK packages. Instead it
//! reuses the Rust components pixi is built on:
//!
//! - `rattler_solve` resolves a requested package and its full dependency closure;
//! - `rattler::install::Installer` downloads/extracts/links packages into the VMR version prefix
//!   (`<versions>/<sdk>_versions/<plugin>-<version>`), including symlinks, noarch handling and
//!   prefix rewriting;
//! - repodata is fetched from a pixi-compatible channel (conda-forge by default) and cached
//!   zstd-compressed under `<cache>/pixi_repodata/`.
//!
//! The channel/arch mapping mirrors pixi's Linux/macOS/Windows subdirs.

pub mod platform;
pub mod repodata;

pub use platform::{PlatformParse, current_subdir, platform_for};
pub use repodata::{
    RecordMeta, install_packages, query_versions, query_versions_cached, resolve_packages,
};
