//! `vmrSearchByConda` compatibility binding.
//!
//! The function name is kept so existing Lua plugins keep working, but the implementation no longer
//! invokes a local conda/miniconda executable. It queries the repodata used by the pixi backend
//! (`vmr-pixi`), and the returned items are installed through pixi/rattler on `vmr use`.

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
                match vmr_pixi::query_versions(&sdk_name) {
                    Ok(versions) => {
                        for ver in versions {
                            let items = vl.entry(ver).or_default();
                            items.push(conda_item());
                        }
                    }
                    Err(e) => eprintln!("warning: conda search failed for {sdk_name}: {e}"),
                }
            }
            drop(guard);
            Ok(mlua::Value::UserData(ud))
        },
    )?;
    // New name for the pixi-backed backend; keep the old binding for upstream plugins.
    if let Ok(f) = lua.globals().get::<mlua::Function>("vmrSearchByConda") {
        let _ = lua.globals().set("vmrSearchByPixi", f);
    }
    Ok(())
}

fn conda_item() -> Item {
    let os = vmr_pixi::platform::os_name();
    let arch = vmr_pixi::platform::arch_name();
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
