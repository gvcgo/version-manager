//! conda 绑定（vmrSearchByConda，要求 4）：查 conda 源 repodata（vmr-conda），
//! 把当前平台全部版本追加进传入的版本表，Item.installer="conda"。
//!
//! Go 版走本机 `conda search` 命令（依赖 Miniconda）——重写后按要求改为
//! 直接查 conda 源，**不依赖本机 conda**。

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

/// 表值→空转的兼容占位（保留引用以防未来扩展）。
#[allow(dead_code)]
fn _placeholder(_: &mlua::Table) {}
