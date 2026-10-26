use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::{Diagram, Edge, Node};

#[derive(Deserialize)]
struct File {
    #[serde(rename = "type")]
    _type: Type,
    elements: Vec<Entry>,
}

#[derive(Deserialize)]
enum Type {
    #[serde(rename = "excalidraw")]
    Excalidraw,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    id: Option<String>,
    #[serde(default)]
    is_deleted: bool,
    #[serde(flatten)]
    element: Element,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Element {
    #[serde(rename = "type")]
    kind: String,
    container_id: Option<String>,
    original_text: Option<String>,
    text: Option<String>,
    name: Option<String>,
    start_binding: Option<Binding>,
    end_binding: Option<Binding>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Binding {
    element_id: Option<String>,
}

pub(crate) fn parse(bytes: &[u8]) -> serde_json::Result<(Diagram, Vec<String>)> {
    let file: File = serde_json::from_slice(bytes)?;
    let (elements, warnings) = keyed_elements(file.elements);
    let (texts, mut elements): (Vec<_>, Vec<_>) = elements
        .into_iter()
        .partition(|(_, element)| element.container_id.is_some());
    let keys: HashSet<&str> = elements.iter().map(|(key, _)| key.as_str()).collect();
    let (bound_texts, standalone_texts): (Vec<_>, Vec<_>) =
        texts.into_iter().partition(|(_, text)| {
            text.container_id
                .as_deref()
                .is_some_and(|container| keys.contains(container))
        });
    elements.extend(standalone_texts);
    let mut labels: HashMap<String, String> = bound_texts
        .into_iter()
        .filter_map(|(_, text)| Some((text.container_id?, text.original_text.or(text.text)?)))
        .collect();
    let (arrows, elements): (Vec<_>, Vec<_>) = elements
        .into_iter()
        .partition(|(_, element)| element.kind == "arrow");
    let nodes: Vec<Node> = elements
        .into_iter()
        .map(|(key, element)| Node {
            label: labels
                .remove(&key)
                .or(element.original_text)
                .or(element.text)
                .or(element.name)
                .filter(|label| !label.is_empty()),
            key,
            kind: element.kind,
        })
        .collect();
    let nodes_by_key: HashMap<&str, &Node> =
        nodes.iter().map(|node| (node.key.as_str(), node)).collect();
    let bound_node = |binding: Option<Binding>| {
        let key = binding?.element_id?;
        nodes_by_key.get(key.as_str()).map(|&node| node.clone())
    };
    let edges = arrows
        .into_iter()
        .map(|(key, arrow)| Edge {
            label: labels.remove(&key),
            from: bound_node(arrow.start_binding),
            to: bound_node(arrow.end_binding),
            key,
            kind: arrow.kind,
        })
        .collect();
    Ok((Diagram { nodes, edges }, warnings))
}

fn keyed_elements(entries: Vec<Entry>) -> (Vec<(String, Element)>, Vec<String>) {
    let mut first_index: HashMap<String, usize> = HashMap::new();
    let mut elements = Vec::new();
    let mut warnings = Vec::new();
    for (index, entry) in entries.into_iter().enumerate() {
        if entry.is_deleted {
            continue;
        }
        let Some(key) = entry.id else {
            warnings.push(format!("elements[{index}] skipped: no id"));
            continue;
        };
        if let Some(first) = first_index.get(&key) {
            warnings.push(format!(
                "elements[{index}] skipped: id #{key} is used by elements[{first}]"
            ));
            continue;
        }
        first_index.insert(key.clone(), index);
        elements.push((key, entry.element));
    }
    (elements, warnings)
}
