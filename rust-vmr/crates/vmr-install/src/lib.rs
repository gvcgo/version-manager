//! VMR SDK installer — archive / coursier / executable install strategies.
//!
//! This crate ports the Go `internal/installer/install` package and provides
//! three main installer types:
//!
//! - [`ArchiverInstaller`] — downloads compressed archives, extracts them, and
//!   copies the SDK home directory.
//! - [`ExeInstaller`] — handles standalone executables, Miniconda `.sh`
//!   installers, VSCode platform packages, and Windows EXE installers.
//! - [`CoursierInstaller`] — installs JVM-based SDKs via the `cs` command.

pub mod common;
pub mod archiver;
pub mod executable;
pub mod coursier;

// Placeholder modules — to be implemented in future iterations.
pub mod installed;
pub mod cached;
pub mod locker;
pub mod prequisite;
