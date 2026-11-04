//! HTML selector bindings (vmrInitSelection/Find/Eq/Attr/Text/Each).
//!
//! Go's goquery `Selection` is a set of nodes inside one document; operations are node-relative
//! (`Find` looks at descendants of the selected nodes, `Eq` picks by position, `Attr` reads the
//! first node, `Text` concatenates descendant text). Storing serialized HTML fragments breaks
//! this for table rows (`<tr>`/`<td>` fragments are invalid outside their parent context and are
//! dropped by the HTML parser), so this implementation stores the parsed document plus the
//! child-index path of every selected node. Every operation then walks the same tree, which keeps
//! `Find`/`Eq`/`Attr`/`Text` exactly node-relative.

use std::rc::Rc;

use mlua::{Function, Lua, UserData, Value};
use scraper::{ElementRef, Html, Selector};

use crate::bindings::register_fn;
use crate::req::StringUD;

/// HTML selection: parsed document plus child-index paths (document root → node).
#[derive(Clone)]
pub struct SelectionUD {
    doc: Rc<Html>,
    paths: Vec<Vec<usize>>,
}

impl UserData for SelectionUD {}

/// Child-index path from the document root to `element`.
fn path_of(element: ElementRef<'_>) -> Vec<usize> {
    let mut path = Vec::new();
    let mut node = *element; // ElementRef derefs to NodeRef
    while let Some(parent) = node.parent() {
        let mut index = 0usize;
        let mut prev = node.prev_sibling();
        while let Some(p) = prev {
            index += 1;
            prev = p.prev_sibling();
        }
        path.push(index);
        node = parent;
    }
    path.reverse();
    path
}

/// Resolves a child-index path against the document.
fn element_at<'a>(doc: &'a Html, path: &[usize]) -> Option<ElementRef<'a>> {
    let mut node = doc.tree.root();
    for &index in path {
        node = node.children().nth(index)?;
    }
    ElementRef::wrap(node)
}

/// Selects elements matching `selector` among the descendants of every selected node.
fn find_under(doc: &Html, paths: &[Vec<usize>], selector: &str) -> Vec<Vec<usize>> {
    let Ok(sel) = Selector::parse(selector) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for path in paths {
        if let Some(element) = element_at(doc, path) {
            for found in element.select(&sel) {
                out.push(path_of(found));
            }
        }
    }
    out
}

fn selection_from(lua: &Lua, doc: Rc<Html>, paths: Vec<Vec<usize>>) -> mlua::Result<Value> {
    lua.create_userdata(SelectionUD { doc, paths })
        .map(Value::UserData)
}

fn borrow_selection(v: &Value) -> Option<(Rc<Html>, Vec<Vec<usize>>)> {
    match v {
        Value::UserData(u) => u
            .borrow::<SelectionUD>()
            .ok()
            .map(|s| (s.doc.clone(), s.paths.clone())),
        _ => None,
    }
}

pub fn register(lua: &Lua) -> mlua::Result<()> {
    register_fn(
        lua,
        "vmrInitSelection",
        |lua, (ud, selector): (Value, String)| {
            let html = match &ud {
                Value::UserData(u) => u
                    .borrow::<StringUD>()
                    .map(|s| s.0.clone())
                    .unwrap_or_default(),
                Value::String(s) => crate::req::lua_string_to_owned(s),
                _ => String::new(),
            };
            if html.is_empty() {
                return selection_from(lua, Rc::new(Html::new_document()), Vec::new());
            }
            let doc = Rc::new(Html::parse_document(&html));
            let Ok(sel) = Selector::parse(&selector) else {
                return selection_from(lua, doc, Vec::new());
            };
            let paths: Vec<Vec<usize>> = doc.select(&sel).map(path_of).collect();
            selection_from(lua, doc, paths)
        },
    )?;

    register_fn(lua, "vmrFind", |lua, (ud, selector): (Value, String)| {
        let Some((doc, paths)) = borrow_selection(&ud) else {
            return selection_from(lua, Rc::new(Html::new_document()), Vec::new());
        };
        let found = find_under(&doc, &paths, &selector);
        selection_from(lua, doc, found)
    })?;

    register_fn(lua, "vmrEq", |lua, (ud, index): (Value, i64)| {
        let Some((doc, paths)) = borrow_selection(&ud) else {
            return selection_from(lua, Rc::new(Html::new_document()), Vec::new());
        };
        let one = if index >= 0 {
            paths
                .get(index as usize)
                .cloned()
                .map(|p| vec![p])
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        selection_from(lua, doc, one)
    })?;

    register_fn(lua, "vmrAttr", |_, (ud, name): (Value, String)| {
        let Some((doc, paths)) = borrow_selection(&ud) else {
            return Ok(String::new());
        };
        Ok(paths
            .iter()
            .find_map(|p| element_at(&doc, p))
            .and_then(|e| e.attr(&name).map(str::to_string))
            .unwrap_or_default())
    })?;

    register_fn(lua, "vmrText", |_, (ud,): (Value,)| {
        let Some((doc, paths)) = borrow_selection(&ud) else {
            return Ok(String::new());
        };
        let mut out = String::new();
        for path in &paths {
            if let Some(element) = element_at(&doc, path) {
                out.push_str(&element.text().collect::<Vec<_>>().concat());
            }
        }
        Ok(out)
    })?;

    register_fn(lua, "vmrEach", |lua, (ud, cb): (Value, Function)| {
        let Some((doc, paths)) = borrow_selection(&ud) else {
            return Ok(());
        };
        for (i, path) in paths.iter().enumerate() {
            let node = lua.create_userdata(SelectionUD {
                doc: doc.clone(),
                paths: vec![path.clone()],
            })?;
            cb.call::<()>((i as i64, node))?;
        }
        Ok(())
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Html {
        Html::parse_document(
            r#"<ul class="toggle"><li id="go1.21"><table class="downloadtable"><tr><td><a href="/dl/go1.21.linux-amd64.tar.gz">x</a></td><td>Archive</td><td>Linux</td><td>x86-64</td><td>sha</td></tr></table></li><li id="go1.20">v</li></ul>"#,
        )
    }

    #[test]
    fn selects_finds_and_reads_attrs() {
        let doc = sample();
        let top = doc
            .select(&Selector::parse(".toggle > li").unwrap())
            .map(path_of)
            .collect::<Vec<_>>();
        assert_eq!(top.len(), 2);

        let tds = find_under(&doc, &top[..1], "td");
        assert_eq!(tds.len(), 5);
        let a = find_under(&doc, &tds[..1], "a");
        assert_eq!(
            element_at(&doc, &a[0])
                .and_then(|e| e.attr("href"))
                .unwrap(),
            "/dl/go1.21.linux-amd64.tar.gz"
        );
    }

    #[test]
    fn tr_and_td_fragments_keep_their_children() {
        // The old fragment-based implementation dropped <tr>/<td> children because they are
        // invalid outside a table context. Path-based lookups must not.
        let doc = sample();
        let table = doc
            .select(&Selector::parse("table.downloadtable").unwrap())
            .map(path_of)
            .collect::<Vec<_>>();
        let rows = find_under(&doc, &table, "tr");
        assert_eq!(rows.len(), 1);
        let cells = find_under(&doc, &rows, "td");
        assert_eq!(cells.len(), 5);
        let text: String = [1usize, 2, 3]
            .iter()
            .map(|i| {
                element_at(&doc, &cells[*i])
                    .unwrap()
                    .text()
                    .collect::<String>()
                    .trim()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("|");
        assert_eq!(text, "Archive|Linux|x86-64");
    }
}
