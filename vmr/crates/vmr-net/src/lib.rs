//! vmr-net: requests + downloads + GitHub API (plan.md §3.3, requirements 3/8).
//!
//! - `fetcher`: reqwest wrapper (no default UA/retry/timeout, mirrors Go); the reverse-proxy /
//!   proxy priority chain (gitee gets neither a reverse proxy nor a local proxy).
//! - `download`: multithreaded chunked download (Range → `.part%v` → merge) + checksum /
//!   size verification.
//! - `github`: GitHub REST API client (paginated releases + contents file listing).

pub mod download;
pub mod fetcher;
pub mod github;

pub use download::{Checksum, SumType, checksum_of_file, download_file, download_url};
pub use fetcher::{Fetcher, chain_url, resolve_chain};
pub use github::{Asset, Gh, ReleaseItem, RepoFile};
