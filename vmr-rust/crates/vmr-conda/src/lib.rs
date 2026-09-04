//! vmr-conda：conda 源客户端（plan.md §3.5，要求 2/4；不依赖本机 conda/miniconda）。
//!
//! - channel：默认 `https://conda.anaconda.org/conda-forge`。
//! - platform：由 os/arch 推导 subdir（linux-64/osx-64/win-64/…arm64 变体）。
//! - repodata：拉 `<channel>/<subdir>/repodata.json`（404 时回退 `.zst`），
//!   磁盘缓存 `<cache>/conda_repodata/…`（按 mtime + 保留时间过期）。
//! - 查询：包全部版本（去重）；安装用记录（name/version 精确 + 最高 build）。
//! - 安装：阶段一**单包、不递归依赖**（plan D4 默认）——下载 `.conda`
//!   （zip 容器内 `pkg-*.tar.zst`）或 `.tar.bz2`，提取到 vmr 版本目录（prefix）。
//! - 注意：D1 决策点采用 plan 的**自研轻量**回退分支（serde_json + 手写
//!   zst/zip 提取），不引入 rattler 系列（编译负担重）；语义以本模块为准。

pub mod platform;
pub mod repodata;

pub use platform::{PlatformParse, current_subdir, platform_for};
pub use repodata::{RepoPackage, install_package, query_packages, query_versions, select_package};
