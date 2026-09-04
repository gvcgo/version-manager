//! github bindings (`vmrGetGithubRelease`, requirement 3) — fetch GitHub releases with pagination,
//! filtered through 6 Lua callbacks (tagFilter/versionParser/fileFilter/archParser/osParser/
//! installerGetter), mirroring Go `lua_global/github.go` semantics:
//! error/non-bool/non-string return values inside callbacks are treated as false/"" (Go recover fallback);
//! `archive/refs/` assets are filtered out. The result shares the shape of other version tables (VersionListUD),
//! so it can be fed straight into vmrMergeVersionList / used as the crawl return value.

use mlua::{Function, Lua, Value};

use crate::bindings::register_fn;
use crate::version::VersionListUD;

fn call_bool(f: &Function, arg: &str) -> bool {
    f.call::<bool>(arg).unwrap_or(false)
}
fn call_str(f: &Function, arg: &str) -> String {
    f.call::<String>(arg).unwrap_or_default()
}

pub fn register(lua: &Lua) -> mlua::Result<()> {
    register_fn(
        lua,
        "vmrGetGithubRelease",
        |lua, (repo, tag_filter, ver_parser, file_filter, arch_parser, os_parser, installer_getter): (
            String,
            Function,
            Function,
            Function,
            Function,
            Function,
            Function,
        )| {
            let result = fetch_and_filter(
                &repo,
                &tag_filter,
                &ver_parser,
                &file_filter,
                &arch_parser,
                &os_parser,
                &installer_getter,
            );
            lua.create_userdata(VersionListUD(result)).map(Value::UserData)
        },
    )?;
    Ok(())
}

pub fn fetch_and_filter(
    repo: &str,
    tag_filter: &Function,
    ver_parser: &Function,
    file_filter: &Function,
    arch_parser: &Function,
    os_parser: &Function,
    installer_getter: &Function,
) -> crate::types::VersionList {
    let mut result = crate::types::VersionList::new();
    if repo.is_empty() {
        return result;
    }
    let gh = match vmr_net::github::Gh::new() {
        Ok(g) => g,
        Err(_) => return result,
    };
    let releases = match gh.releases(repo) {
        Ok(r) => r,
        Err(_) => return result,
    };
    for r in releases {
        if !call_bool(tag_filter, &r.tag) {
            continue;
        }
        let vstr = call_str(ver_parser, &r.tag);
        if vstr.is_empty() {
            continue;
        }
        for a in &r.assets {
            if a.url.contains("archive/refs/") {
                continue;
            }
            if !call_bool(file_filter, &a.name) {
                continue;
            }
            let arch = call_str(arch_parser, &a.name);
            let os = call_str(os_parser, &a.name);
            if arch.is_empty() || os.is_empty() {
                continue;
            }
            let mut item = crate::types::Item {
                arch,
                os,
                ..Default::default()
            };
            item.installer = call_str(installer_getter, &a.name);
            item.url = a.url.clone();
            item.size = a.size.unwrap_or(0);
            let items = result.entry(vstr.clone()).or_default();
            items.push(item);
        }
    }
    result
}
