//! vmr-installer: SDK installer (plan.md §3.6, requirement 5).
//!
//! - `common`: directory conventions `<versions>/<sdk>_versions/<plugin>-<version>` +
//!   `<versions>/<sdk>_versions/<sdk>` symlink (no-manifest-file disk contract).
//! - `installer`: dispatches by `Item.Installer` to unarchiver/executable/coursier/conda;
//!   `Install()` orchestration (prerequisite checks, post processing, symlink,
//!   global/session/lock modes).
//! - `download`: cached downloads `<cache>/<plugin>/<version>/<file>` (idempotent + validated).
//! - `locker`: `.vmr.lock` project lock (searches upward, JSON).
//! - `finder`: installed/current version discovery, cache cleanup.
//! - `post`: registry of per-SDK post-processing.

pub mod common;
pub mod download;
pub mod finder;
pub mod installer;
pub mod locker;
pub mod post;
