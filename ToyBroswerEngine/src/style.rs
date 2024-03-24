use std::collections::HashMap;
use crate::css::{Rule, StyleSheet};
use crate::css::{Selector, SimpleSelector, Specificity, Value};
use crate::css::Selector::Simple;
use crate::dom::{ElementData, Node};
use crate::dom::NodeType::{Element, Text};

/// todo
/// Cascading
/// Initial and/or computed values
/// Inheritance
/// The style attribute

// Map of css property names to values
type PropertyMap = HashMap<String, Value>;

// A node with associated style data
struct StyleNode<'a> {
    node: &'a Node,
    specified_values: PropertyMap,
    children: Vec<StyleNode<'a>>,
}

fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match *selector {
        Simple(ref simple_selector) => matches_simple_selector(elem, simple_selector)
    }
}

fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {
    // check type selector
    if selector.tage_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }
    // check ID Selector
    if selector.id.iter().any(|id| elem.id() != Some(id)){
        return false;
    }
    // check class selector
    let elem_classes = elem.classes();
    if selector.class.iter().any(|class| !elem_classes.contains(&**class)) {
        return false;
    }

    return true;
}

type MatchedRule<'a> = (Specificity, &'a Rule);
fn match_rule<'a>(elem: &ElementData, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    rule.selectors.iter()
        .find(|selector| matches(elem, *selector))
        .map(|selector| (selector.specificity(), rule))
}

fn matching_rules<'a>(elem: &ElementData, stylesheet: &'a StyleSheet) -> Vec<MatchedRule<'a>> {
    stylesheet.rules.iter().filter_map(|rule| match_rule(elem, rule)).collect()
}

fn specified_values(elem: &ElementData, style_sheet: &StyleSheet) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(elem, style_sheet);

    //go through rules from low to hi specificity.
    rules.sort_by(|&(a,_), &(b, _)| a.cmp(&b));
    for(_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    return values;
}

pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a StyleSheet) -> StyleNode<'a> {
    StyleNode {
        node: root,
        specified_values: match root.node_type {
            Element(ref elem) => specified_values(elem, stylesheet),
            Text(_) => HashMap::new()
        },
        children: root.children.iter().map(|child| style_tree(child, stylesheet)).collect()
    }
}