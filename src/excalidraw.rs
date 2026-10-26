use std::collections::HashMap;

use serde::Deserialize;

use crate::{Diagram, Edge, Node};

#[derive(Deserialize)]
struct File {
    #[serde(rename = "type")]
    _type: Type,
    elements: Vec<Element>,
}

#[derive(Deserialize)]
enum Type {
    #[serde(rename = "excalidraw")]
    Excalidraw,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Element {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    is_deleted: bool,
    container_id: Option<String>,
    original_text: Option<String>,
    text: Option<String>,
    start_binding: Option<Binding>,
    end_binding: Option<Binding>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Binding {
    element_id: Option<String>,
}

pub(crate) fn parse(bytes: &[u8]) -> serde_json::Result<Diagram> {
    let file: File = serde_json::from_slice(bytes)?;
    let (bound_texts, elements): (Vec<Element>, Vec<Element>) = file
        .elements
        .into_iter()
        .filter(|element| !element.is_deleted)
        .partition(|element| element.container_id.is_some());
    let mut labels: HashMap<String, String> = bound_texts
        .into_iter()
        .filter_map(|text| Some((text.container_id?, text.original_text.or(text.text)?)))
        .collect();
    let (arrows, elements): (Vec<Element>, Vec<Element>) = elements
        .into_iter()
        .partition(|element| element.kind == "arrow");
    let nodes: Vec<Node> = elements
        .into_iter()
        .map(|element| {
            let label = labels.remove(&element.id);
            Node {
                key: element.id,
                kind: element.kind,
                label,
            }
        })
        .collect();
    let bound_node = |binding: Option<Binding>| {
        let key = binding?.element_id?;
        nodes.iter().find(|node| node.key == key).cloned()
    };
    let edges = arrows
        .into_iter()
        .map(|arrow| {
            let label = labels.remove(&arrow.id);
            Edge {
                from: bound_node(arrow.start_binding),
                to: bound_node(arrow.end_binding),
                key: arrow.id,
                kind: arrow.kind,
                label,
            }
        })
        .collect();
    Ok(Diagram { nodes, edges })
}
