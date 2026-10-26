use std::collections::HashMap;

use serde::Deserialize;

use crate::Node;

#[derive(Deserialize)]
struct File {
    // An enum, not a String, so a wrong or missing type is a serde error like any other broken file.
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
}

/// The Nodes of an Excalidraw file. Arrows are left out until they become Edges.
pub fn parse(bytes: &[u8]) -> serde_json::Result<Vec<Node>> {
    let file: File = serde_json::from_slice(bytes)?;
    let elements: Vec<Element> = file
        .elements
        .into_iter()
        .filter(|e| !e.is_deleted)
        .collect();
    // Bound text is not a Node of its own: it is its container's Label.
    // Files from before originalText have only text, which Excalidraw itself falls back to.
    let labels: HashMap<&str, &str> = elements
        .iter()
        .filter_map(|e| {
            let text = e.original_text.as_deref().or(e.text.as_deref())?;
            Some((e.container_id.as_deref()?, text))
        })
        .collect();
    Ok(elements
        .iter()
        .filter(|e| e.container_id.is_none() && e.kind != "arrow")
        .map(|e| Node {
            key: e.id.clone(),
            kind: e.kind.clone(),
            label: labels.get(e.id.as_str()).map(|label| label.to_string()),
        })
        .collect())
}
