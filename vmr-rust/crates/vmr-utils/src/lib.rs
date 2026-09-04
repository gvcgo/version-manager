//! vmr-utils：基础工具库（叶子 crate，无 vmr 内部依赖）。
//!
//! 对齐 `vmr-go/internal/utils`（见 plan.md §3.2、description.md §4.3）。
//! 已实现：版本解析与排序（`semver`）、解压家目录查找（`find_dir`）、
//! 命令执行（`exec`）、符号链接（`symlink`）、复制（`copy`）、
//! 压缩归档解压（`extract`）。

pub mod copy;
pub mod exec;
pub mod extract;
pub mod find_dir;
pub mod semver;
pub mod symlink;
