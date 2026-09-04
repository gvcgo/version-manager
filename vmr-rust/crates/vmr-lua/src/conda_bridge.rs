//! conda bindings (`vmrSearchByConda`, requirement 4): query the conda source repodata (vmr-conda)
//! and append every version for the current platform to the passed-in version table, with Item.installer="conda".
//!
//! The Go version runs the local `conda search` command (depends on Miniconda) — after the rewrite it
//! queries the conda source directly instead, **without depending on a local conda**.

use mlua::AnyUserData;

use crate::bindings::register_fn;
use crate::types::Item;
use crate::version::VersionListUD;

pub fn register(lua: &mlua::Lua) -> mlua::Result<()> {
    register_fn(
        lua,
        "vmrSearchByConda",
        |_lua, (ud, sdk_name): (AnyUserData, String)| {
            let mut guard = ud.borrow_mut::<VersionListUD>()?;
            let vl = &mut guard.0;
            if !sdk_name.is_empty() {
                if let Ok(versions) = vmr_conda::query_versions(&sdk_name) {
                    for ver in versions {
                        let items = vl.entry(ver).or_default();
                        items.push(conda_item());
                    }
                }
            }
            drop(guard);
            Ok(mlua::Value::UserData(ud))
        },
    )?;
    Ok(())
}

fn conda_item() -> Item {
    let os = vmr_conda::platform::os_name();
    let arch = vmr_conda::platform::arch_name();
    Item {
        os,
        arch,
        installer: crate::types::installer_kind::CONDA.to_string(),
        ..Default::default()
    }
}

/// Compatibility placeholder mapping a table value to a no-op (kept for possible future extension).
#[allow(dead_code)]
fn _placeholder(_: &mlua::Table) {}
