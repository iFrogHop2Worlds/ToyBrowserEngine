use std::collections::{HashMap, HashSet};

pub type AtterMap = HashMap<String, String>;

#[derive(Debug)]
pub struct Node {
    pub children: Vec<Node>,
    pub node_type: NodeType,
}

#[derive(Debug)]
pub enum NodeType {
    Text(String),
    Element(ElementData),
}
#[derive(Debug)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: AtterMap,
}

pub fn text(data: String) -> Node {
    Node {children: Vec::new(), node_type: NodeType::Text(data)}
}

pub fn elem(name: String, attrs: AtterMap, children: Vec<Node>) -> Node {
    Node {
        children,
        node_type: NodeType::Element(ElementData {
            tag_name: name,
            attributes: attrs
        })
    }
}

impl ElementData {
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    pub fn classes(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            Some(classlist) => classlist.split(' ').collect(),
            None => HashSet::new()
        }
    }
}