//! vmr-installer：SDK 安装器（plan.md §3.6，要求 5）。
//!
//! - `common`：目录约定 `<versions>/<sdk>_versions/<plugin>-<version>` +
//!   `<versions>/<sdk>_versions/<sdk>` 符号链接（无清单文件磁盘契约）。
//! - `installer`：按 `Item.Installer` 分派 unarchiver/executable/coursier/conda，
//!   Install() 调度（前置检查、post 后处理、符号链接、全局/会话/锁模式）。
//! - `download`：缓存下载 `<cache>/<plugin>/<version>/<file>`（幂等 + 校验）。
//! - `locker`：`.vmr.lock` 项目锁（向上查找，JSON）。
//! - `finder`：已装版本/当前版本发现、缓存清理。
//! - `post`：SDK 差异化后处理注册表。

pub mod common;
pub mod download;
pub mod finder;
pub mod installer;
pub mod locker;
pub mod post;
