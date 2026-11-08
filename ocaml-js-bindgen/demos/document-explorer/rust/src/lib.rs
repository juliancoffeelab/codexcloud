use ocamljs_bindgen::{
    HostError,
    host::{CopyError, JsString, Promise},
    ocamljs_bindgen,
    views::{SharedView, ViewOwner},
};
use serde_json::Value;
use std::cell::Cell;
thread_local! { static NEXT:Cell<u32>=const {Cell::new(1)}; }
#[ocamljsify::ocamljsify]
#[derive(Debug)]
pub enum DocumentError {
    InvalidJson,
    StaleNode,
    MissingNode,
    ViewsPresent,
    TooLarge,
    NoFile,
    Browser(HostError),
}
#[ocamljsify::ocamljsify]
#[derive(Clone)]
pub struct NodeId {
    pub document: u32,
    pub revision: u32,
    pub path: Vec<String>,
}
/// The six JSON value kinds; this crosses the binding as a scalar enum.
#[ocamljsify::ocamljsify]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Object,
    Array,
    String,
    Number,
    Bool,
    Null,
}

#[ocamljsify::ocamljsify]
pub struct Summary {
    pub id: NodeId,
    pub label: String,
    pub kind: NodeKind,
}
#[ocamljs_bindgen(opaque)]
pub struct Document {
    id: u32,
    revision: u32,
    tree: ViewOwner<Value>,
    source: Option<Blob>,
}
#[ocamljs_bindgen(opaque)]
pub struct NodeView {
    tree: SharedView<Value>,
    path: Vec<String>,
}
fn parse(text: &str) -> Result<Value, DocumentError> {
    if text.len() > 65536 {
        return Err(DocumentError::TooLarge);
    }
    serde_json::from_str(text).map_err(|_| DocumentError::InvalidJson)
}
fn at<'a>(tree: &'a Value, path: &[String]) -> Option<&'a Value> {
    let mut node = tree;
    for key in path {
        node = match node {
            Value::Array(a) => a.get(key.parse::<usize>().ok()?)?,
            Value::Object(o) => o.get(key)?,
            _ => return None,
        };
    }
    Some(node)
}
fn kind(value: &Value) -> NodeKind {
    match value {
        Value::Object(_) => NodeKind::Object,
        Value::Array(_) => NodeKind::Array,
        Value::String(_) => NodeKind::String,
        Value::Number(_) => NodeKind::Number,
        Value::Bool(_) => NodeKind::Bool,
        Value::Null => NodeKind::Null,
    }
}

fn with_node<T>(
    document: &Document,
    node: &NodeId,
    read: impl FnOnce(&Value) -> T,
) -> Result<T, DocumentError> {
    if node.document != document.id || node.revision != document.revision {
        return Err(DocumentError::StaleNode);
    }
    document.tree.read(|tree| {
        at(tree, &node.path)
            .map(read)
            .ok_or(DocumentError::MissingNode)
    })
}
#[ocamljs_bindgen(module = Document, name = "create")]
pub fn document(text: &str) -> Result<Document, DocumentError> {
    let tree = parse(text)?;
    let id = NEXT.get();
    if id == u32::MAX {
        return Err(DocumentError::TooLarge);
    }
    NEXT.set(id + 1);
    Ok(Document {
        id,
        revision: 1,
        tree: ViewOwner::new(tree),
        source: None,
    })
}
#[ocamljs_bindgen(module = Document)]
pub fn nodes(document: &Document) -> Vec<Summary> {
    fn walk(v: &Value, path: &mut Vec<String>, out: &mut Vec<Summary>, d: &Document) {
        if out.len() >= 256 {
            return;
        }
        out.push(Summary {
            id: NodeId {
                document: d.id,
                revision: d.revision,
                path: path.clone(),
            },
            label: if path.is_empty() {
                "root".into()
            } else {
                path.join(" / ")
            },
            kind: kind(v),
        });
        match v {
            Value::Object(o) => {
                for (k, v) in o {
                    path.push(k.clone());
                    walk(v, path, out, d);
                    path.pop();
                }
            }
            Value::Array(a) => {
                for (i, v) in a.iter().enumerate() {
                    path.push(i.to_string());
                    walk(v, path, out, d);
                    path.pop();
                }
            }
            _ => {}
        }
    }
    document.tree.read(|v| {
        let mut out = vec![];
        walk(v, &mut vec![], &mut out, document);
        out
    })
}
#[ocamljs_bindgen(module = Document)]
pub fn inspect(document: &Document, node: NodeId) -> Result<String, DocumentError> {
    with_node(document, &node, Value::to_string)
}
#[ocamljs_bindgen(module = Document)]
pub fn replace(document: &mut Document, text: &str) -> Result<(), DocumentError> {
    let value = parse(text)?;
    let revision = document
        .revision
        .checked_add(1)
        .ok_or(DocumentError::TooLarge)?;
    document
        .tree
        .mutate(|v| *v = value)
        .map_err(|_| DocumentError::ViewsPresent)?;
    document.revision = revision;
    document.source = None;
    Ok(())
}
#[ocamljs_bindgen(module = NodeView, name = "create")]
pub fn node_view(document: &Document, node: NodeId) -> Result<NodeView, DocumentError> {
    with_node(document, &node, |_| ())?;
    Ok(NodeView {
        tree: document.tree.view(),
        path: node.path,
    })
}
#[ocamljs_bindgen(module = NodeView, name = "text")]
pub fn view_text(view: &NodeView) -> String {
    view.tree
        .read(|v| at(v, &view.path).expect("pinned view path").to_string())
}
#[ocamljs_bindgen(module = NodeView, name = "child")]
pub fn child_view(view: &NodeView, key: String) -> Result<NodeView, DocumentError> {
    let mut path = view.path.clone();
    path.push(key);
    if !view.tree.read(|v| at(v, &path).is_some()) {
        return Err(DocumentError::MissingNode);
    }
    Ok(NodeView {
        tree: view.tree.child(),
        path,
    })
}

#[ocamljs_bindgen(host)]
pub struct Blob;
#[ocamljs_bindgen(host)]
pub struct FileInput;
#[ocamljs_bindgen(host)]
pub struct FileList;
#[ocamljs_bindgen]
unsafe extern "C" {
    #[ocamljs_bindgen(method, js_name = "text")]
    fn blob_text(blob: &Blob) -> Result<Promise<JsString>, HostError>;
    #[ocamljs_bindgen(getter, js_name = "size")]
    fn blob_size(blob: &Blob) -> Result<f64, HostError>;
    #[ocamljs_bindgen(js_namespace = "document", js_name = "getElementById")]
    fn file_input(id: &str) -> Result<FileInput, HostError>;
    #[ocamljs_bindgen(getter, js_name = "files")]
    fn files(input: &FileInput) -> Result<FileList, HostError>;
    #[ocamljs_bindgen(getter, js_name = "length")]
    fn file_count(files: &FileList) -> Result<u32, HostError>;
    #[ocamljs_bindgen(indexing_getter)]
    fn file_at(files: &FileList, index: u32) -> Result<Blob, HostError>;
}
#[ocamljs_bindgen(module=Blob,name="from_input")]
pub fn selected_file(id: &str) -> Result<Blob, DocumentError> {
    let list =
        files(&file_input(id).map_err(DocumentError::Browser)?).map_err(DocumentError::Browser)?;
    if file_count(&list).map_err(DocumentError::Browser)? == 0 {
        return Err(DocumentError::NoFile);
    }
    file_at(&list, 0).map_err(DocumentError::Browser)
}
#[ocamljs_bindgen(module=Document,name="of_blob")]
pub async fn from_blob(source: &Blob) -> Result<Document, DocumentError> {
    if blob_size(source).map_err(DocumentError::Browser)? > 65536.0 {
        return Err(DocumentError::TooLarge);
    }
    let text = blob_text(source)
        .map_err(DocumentError::Browser)?
        .await
        .map_err(DocumentError::Browser)?;
    let text = text.to_rust_string(65536).map_err(|error| match error {
        CopyError::TooLarge => DocumentError::TooLarge,
        CopyError::Host(error) => DocumentError::Browser(error),
    })?;
    let mut document = document(&text)?;
    document.source = Some(source.clone());
    Ok(document)
}
#[ocamljs_bindgen(module=Document)]
pub fn source(document: &Document) -> Option<Blob> {
    document.source.clone()
}

#[cfg(feature = "conformance")]
include!("../../tests/rust/conformance.rs");
