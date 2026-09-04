//! vmr-net：请求 + 下载 + GitHub API（plan.md §3.3，要求 3/8）。
//!
//! - `fetcher`：reqwest 封装（无默认 UA/重试/超时，对齐 Go），镜像/反代/代理
//!   优先级链（镜像替换先于反代；仅镜像未改 URL 才叠加反代；gitee 不反代也不用
//!   本地代理）。
//! - `download`：多线程分片下载（Range → `.part%v` → 合并）+ 校验和/大小校验。
//! - `github`：GitHub REST API 客户端（releases 分页 + contents 文件列表）。

pub mod download;
pub mod fetcher;
pub mod github;

pub use download::{Checksum, SumType, checksum_of_file, download_file};
pub use fetcher::{Fetcher, chain_url, resolve_chain};
pub use github::{Asset, Gh, ReleaseItem, RepoFile};
