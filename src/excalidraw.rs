use std::collections::HashMap;

use serde::Deserialize;

use crate::Node;

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
}

pub fn parse(bytes: &[u8]) -> serde_json::Result<Vec<Node>> {
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
    Ok(elements
        .into_iter()
        .filter(|element| element.kind != "arrow")
        .map(|element| {
            let label = labels.remove(&element.id);
            Node {
                key: element.id,
                kind: element.kind,
                label,
            }
        })
        .collect())
}
