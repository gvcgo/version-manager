//! HTML selector bindings (vmrInitSelection/Find/Eq/Attr/Text/Each).
//!
//! Go's goquery Selection is a set of nodes within a document. On the Rust side the selection is
//! expressed as **node HTML fragments** (each operation rebuilds and re-selects from the fragment subtree),
//! matching the behavior:
//! - `Find(sel)`: re-select descendants within each fragment (flattened in document order);
//! - `Eq(i)`: take the i-th one (0-based);
//! - `AttrOr(name, "")`: read the attribute of the **first** node (fragment's top-level element);
//! - `Text`: concatenate text of all fragments (including descendant text, mirroring goquery);
//! - `Each(cb)`: invoke cb with i from 0, arguments (i, single-node selection).

use mlua::{Function, Lua, UserData, Value};

use crate::bindings::register_fn;
use crate::req::StringUD;

/// HTML selection (a list of node HTML fragments).
#[derive(Clone, Default)]
pub struct SelectionUD {
    pub frags: Vec<String>,
}
impl UserData for SelectionUD {}

fn selector_ok(sel: &str) -> bool {
    !sel.is_empty() && scraper::Selector::parse(sel).is_ok()
}

/// Select elements matching selector in html (returns outer HTML fragments, in document order).
fn select_frags(html: &str, selector: &str) -> Vec<String> {
    let Ok(sel) = scraper::Selector::parse(selector) else {
        return Vec::new();
    };
    let doc = scraper::Html::parse_fragment(html);
    doc.root_element().select(&sel).map(|e| e.html()).collect()
}

fn find_in_frags(frags: &[String], selector: &str) -> Vec<String> {
    let mut out = Vec::new();
    for f in frags {
        out.extend(select_frags(f, selector));
    }
    out
}

/// Fragment text (all text within the top-level wrapper, mirroring goquery's descendant text of nodes).
fn text_of_frags(frags: &[String]) -> String {
    let mut out = String::new();
    for f in frags {
        let doc = scraper::Html::parse_fragment(f);
        out.push_str(&doc.root_element().text().collect::<Vec<_>>().concat());
    }
    out
}

/// Attribute value of the first top-level element.
fn attr_of_first(frags: &[String], name: &str) -> String {
    for f in frags {
        let doc = scraper::Html::parse_fragment(f);
        if let Some(e) = doc.root_element().child_elements().next() {
            if let Some(v) = e.attr(name) {
                return v.to_string();
            }
        }
    }
    String::new()
}

pub fn register(lua: &Lua) -> mlua::Result<()> {
    register_fn(
        lua,
        "vmrInitSelection",
        |lua, (ud, selector): (Value, String)| {
            let html: String = match &ud {
                Value::UserData(u) => u
                    .borrow::<StringUD>()
                    .map(|s| s.0.clone())
                    .unwrap_or_default(),
                Value::String(s) => crate::req::lua_string_to_owned(s),
                _ => String::new(),
            };
            let frags = if html.is_empty() || !selector_ok(&selector) {
                Vec::new()
            } else {
                select_frags(&html, &selector)
            };
            push_selection(lua, frags)
        },
    )?;

    register_fn(lua, "vmrFind", |lua, (ud, selector): (Value, String)| {
        let frags = sel_frags(ud);
        let out = if selector_ok(&selector) {
            find_in_frags(&frags, &selector)
        } else {
            Vec::new()
        };
        push_selection(lua, out)
    })?;

    register_fn(lua, "vmrEq", |lua, (ud, index): (Value, i64)| {
        let frags = sel_frags(ud);
        let one = frags
            .into_iter()
            .nth(index.max(0) as usize)
            .map(|f| vec![f])
            .unwrap_or_default();
        push_selection(lua, one)
    })?;

    register_fn(lua, "vmrAttr", |_, (ud, name): (Value, String)| {
        let frags = sel_frags(ud);
        Ok(attr_of_first(&frags, &name))
    })?;

    register_fn(lua, "vmrText", |_, (ud,): (Value,)| {
        let frags = sel_frags(ud);
        Ok(text_of_frags(&frags))
    })?;

    register_fn(lua, "vmrEach", |lua, (ud, cb): (Value, Function)| {
        let frags = sel_frags(ud);
        for (i, frag) in frags.iter().enumerate() {
            let node = lua.create_userdata(SelectionUD {
                frags: vec![frag.clone()],
            })?;
            cb.call::<()>((i as i64, node))?;
        }
        Ok(())
    })?;
    Ok(())
}

fn sel_frags(v: Value) -> Vec<String> {
    match v {
        Value::UserData(u) => u
            .borrow::<SelectionUD>()
            .map(|s| s.frags.clone())
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn push_selection(lua: &Lua, frags: Vec<String>) -> mlua::Result<Value> {
    lua.create_userdata(SelectionUD { frags })
        .map(Value::UserData)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> String {
        r#"<ul class="toggle"><li id="go1.21"><table class="downloadtable"><tr><td><a href="/dl/go1.21.linux-amd64.tar.gz">x</a></td><td>Archive</td><td>Linux</td><td>x86-64</td><td>sha</td></tr></table></li><li id="go1.20">v</li></ul>"#
            .to_string()
    }

    #[test]
    fn select_and_find_attr_text() {
        let html = sample();
        let top = select_frags(&html, ".toggle > li");
        assert_eq!(top.len(), 2);
        let tds = find_in_frags(&top[..1], "td");
        assert_eq!(tds.len(), 5);
        let a = find_in_frags(&tds[..1], "a");
        assert_eq!(attr_of_first(&a, "href"), "/dl/go1.21.linux-amd64.tar.gz");
        assert!(text_of_frags(&tds[1..2]).contains("Archive"));
        assert!(text_of_frags(&tds[2..3]).contains("Linux"));
    }

    #[test]
    fn text_only_cells() {
        let frags = vec!["Archive".to_string()];
        assert_eq!(text_of_frags(&frags), "Archive");
    }

    #[test]
    fn wrapper_text_includes_nested() {
        let frags = vec!["<td><a href=\"/x\">link</a></td>".to_string()];
        assert_eq!(text_of_frags(&frags), "link");
    }
}
