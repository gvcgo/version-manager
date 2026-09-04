//! vmr-utils: base utility library (a leaf crate with no internal vmr dependencies).
//!
//! Mirrors `vmr-go/internal/utils` (see plan.md §3.2 and description.md §4.3).
//! Implemented: version parsing and sorting (`semver`), home-directory lookup among extraction
//! results (`find_dir`), command execution (`exec`), symlinks (`symlink`), copying (`copy`),
//! and compression/archive extraction (`extract`).

pub mod copy;
pub mod exec;
pub mod extract;
pub mod find_dir;
pub mod semver;
pub mod symlink;
