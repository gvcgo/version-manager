//! 解压产物中查找"家目录"（对齐 Go `vmr-go/internal/utils/find_dir.go`）。
//!
//! 安装器把压缩包解压到 temp 后，产物可能是 `xxx-1.0/bin/...` 一层目录，
//! 需要找到真正的安装根。判定规则（与 Go 完全一致，含 quirk）：
//! - 以**目录项名字符串拼接**为检测对象，flag 是**子串**匹配（非相等）。
//! - 拼接包含全部条目名；`except_dir` 置位时只拼**非目录**条目（含符号链接，
//!   因其按类型不算目录）。
//! - 每个目录先自测，全中即命中（DFS 前序、**首中即停**）；未命中递归子目录，
//!   跳过 `__MACOSX`。
//! - 目录遍历**按文件名排序**（对齐 Go `os.ReadDir` 的排序行为），保证确定性。
//! - 符号链接目录不展开（`file_type` 不跟随链接，对齐 Go `DirEntry.IsDir`）。
//! - flag 列表为空时根目录自身即命中（Go 同：空 flag 全中）。
//! - `find` 前已命中则直接返回（Go `if h.home != "" { return }` 守卫）。

use std::fs;
use std::path::{Path, PathBuf};

/// 家目录查找器。
pub struct HomeDirFinder {
    home: Option<PathBuf>,
    flag_files: Vec<String>,
    except_dir: bool,
}

impl HomeDirFinder {
    pub fn new(flag_files: Vec<String>) -> Self {
        HomeDirFinder {
            home: None,
            flag_files,
            except_dir: false,
        }
    }

    /// 重新设置标志文件（对齐 Go `SetFlags`）。
    pub fn set_flags(&mut self, flag_files: Vec<String>) {
        self.flag_files = flag_files;
    }

    /// 置位后只以**文件**名（非目录条目）参与检测（对齐 Go `SetFlagDirExcepted`）。
    pub fn set_flag_dir_excepted(&mut self, ok: bool) {
        self.except_dir = ok;
    }

    /// 复位为初始状态（对齐 Go `Clear`）。
    pub fn clear(&mut self) {
        self.home = None;
        self.flag_files.clear();
        self.except_dir = false;
    }

    /// 从 `start_dir` 开始 DFS 查找；命中后再次调用不改变结果（首中即停）。
    pub fn find(&mut self, start_dir: &Path) {
        if self.home.is_some() {
            return;
        }
        let entries = match fs::read_dir(start_dir) {
            Ok(iter) => iter,
            Err(_) => return,
        };
        // 只关心名字与是否目录；Go `os.ReadDir` 按文件名排序，这里复刻以保证确定性。
        let mut names: Vec<(String, bool)> = entries
            .filter_map(|e| e.ok())
            .map(|e| {
                let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let name = e.file_name().to_string_lossy().into_owned();
                (name, is_dir)
            })
            .collect();
        names.sort_by(|a, b| a.0.cmp(&b.0));

        // 拼接当前目录的全部条目名（子串匹配对象）。
        let mut concat = String::new();
        for (name, is_dir) in &names {
            if self.except_dir && *is_dir {
                continue;
            }
            concat.push_str(name);
        }

        // 自测当前目录：全部 flag 命中（子串匹配，含跨名字边界的拼接 quirk）。
        if self.flag_files.iter().all(|f| concat.contains(f.as_str())) {
            self.home = Some(start_dir.to_path_buf());
            return;
        }
        // 未命中则递归子目录（跳过 __MACOSX，与 Go 一致）。
        for (name, is_dir) in &names {
            if *is_dir && name != "__MACOSX" {
                self.find(&start_dir.join(name));
                if self.home.is_some() {
                    return;
                }
            }
        }
    }

    /// 命中目录；未命中返回 `None`（对齐 Go `GetDirName`，空串表示未命中）。
    pub fn get_dir_name(&self) -> Option<PathBuf> {
        self.home.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    /// 测试用临时目录，析构时自动清理。
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let p =
                std::env::temp_dir().join(format!("vmr-utils-finddir-{}-{name}", process::id()));
            let _ = fs::remove_dir_all(&p);
            fs::create_dir_all(&p).unwrap();
            TempDir(p)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// 创建空文件（自动补父目录）。
    fn touch(p: &Path) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, "").unwrap();
    }

    fn flags(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn finds_home_in_top_level_subdir() {
        let t = TempDir::new("top-subdir");
        let home = t.path().join("somepkg-1.0");
        touch(&home.join("bin/tool"));
        touch(&home.join("README"));

        let mut f = HomeDirFinder::new(flags(&["bin"]));
        f.find(t.path());
        assert_eq!(f.get_dir_name().unwrap(), home);
    }

    #[test]
    fn requires_all_flags_present() {
        let t = TempDir::new("all-flags");
        // bin 在，lib 不在 → 顶层子目录不命中，继续下钻到 bin 自身也不满足 lib。
        touch(&t.path().join("pkg/bin/x"));
        let mut f = HomeDirFinder::new(flags(&["bin", "lib"]));
        f.find(t.path());
        assert_eq!(f.get_dir_name(), None);
    }

    #[test]
    fn skips_macosx() {
        let t = TempDir::new("macosx");
        // 唯一命中点藏在 __MACOSX 里 → 必须跳过。
        touch(&t.path().join("__MACOSX/zzz"));
        let mut f = HomeDirFinder::new(flags(&["zzz"]));
        f.find(t.path());
        assert_eq!(f.get_dir_name(), None);
    }

    #[test]
    fn except_dir_matches_files_only() {
        let t = TempDir::new("except-dir");
        // root 下 LICENSE 是**目录**：except_dir 置位时不计入拼接 → root 不命中；
        // 不置位时 root 命中。
        fs::create_dir_all(t.path().join("LICENSE")).unwrap();
        let mut f = HomeDirFinder::new(flags(&["LICENSE"]));
        f.find(t.path());
        assert_eq!(f.get_dir_name().unwrap(), t.path());

        let mut f2 = HomeDirFinder::new(flags(&["LICENSE"]));
        f2.set_flag_dir_excepted(true);
        f2.find(t.path());
        assert_eq!(f2.get_dir_name(), None);
    }

    #[test]
    fn recursion_visits_dirs_in_sorted_order() {
        let t = TempDir::new("sorted");
        // 两个候选子目录都含 mark；按名字排序 DFS，a 先于 b 命中。
        touch(&t.path().join("b/mark"));
        touch(&t.path().join("a/mark"));
        let mut f = HomeDirFinder::new(flags(&["mark"]));
        f.find(t.path());
        assert_eq!(f.get_dir_name().unwrap(), t.path().join("a"));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_dir_is_not_followed() {
        let t = TempDir::new("symlink");
        let target = t.path().join("real");
        touch(&target.join("mark"));
        std::os::unix::fs::symlink(&target, t.path().join("ln")).unwrap();
        let mut f = HomeDirFinder::new(flags(&["mark"]));
        f.find(t.path());
        // 根目录只含 real/ln 两个目录条目：real 子目录含 mark → 命中的是 real。
        assert_eq!(f.get_dir_name().unwrap(), target);
    }

    #[test]
    fn empty_flags_match_start_dir_itself() {
        let t = TempDir::new("empty-flags");
        touch(&t.path().join("x"));
        let mut f = HomeDirFinder::new(flags(&[]));
        f.find(t.path());
        assert_eq!(f.get_dir_name().unwrap(), t.path());
    }

    #[test]
    fn first_hit_is_sticky_and_clear_resets() {
        let t = TempDir::new("sticky");
        touch(&t.path().join("pkg/mark"));
        let mut f = HomeDirFinder::new(flags(&["mark"]));
        f.find(t.path());
        assert!(f.get_dir_name().is_some());
        let first = f.get_dir_name().unwrap().clone();
        // 已命中后再次 find（换个根目录）不改结果。
        let t2 = TempDir::new("sticky-other");
        f.find(t2.path());
        assert_eq!(f.get_dir_name().unwrap(), first);

        // clear 复位：换 flag 后重新查找。
        f.clear();
        f.set_flags(flags(&["other"]));
        f.find(t2.path());
        assert_eq!(f.get_dir_name(), None);
    }
}
