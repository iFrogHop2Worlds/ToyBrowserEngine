use crate::layout::BoxType::{AnonymousBlock, BlockNode, InlineNode};
use crate::style::{StyleNode, Display};
#[derive(Default)]
struct Dimensions {
    content: Rect,
    padding: EdgeSizes,
    border: EdgeSizes,
    margin: EdgeSizes
}

#[derive(Default)]
struct Rect {
    x: f32,
    y: f32,
    height: f32,
    width: f32
}

#[derive(Default)]
struct EdgeSizes {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32
}

struct LayoutBox<'a> {
    dimensions: Dimensions,
    box_type: BoxType<'a>,
    children: Vec<LayoutBox<'a>>
}

impl<'a> LayoutBox<'a> {
    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            box_type,
            dimensions: Default::default(), // sets to 0.0
            children: Vec::new(),
        }
    }

    fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            InlineNode(_) | AnonymousBlock => self,
            BlockNode(_) => {
                // If we've just generated an anonymous block box, keep using it.
                // Otherwise, create a new one.
                match self.children.last() {
                    Some(&LayoutBox { box_type: AnonymousBlock,..}) => {}
                    _ => self.children.push(LayoutBox::new(AnonymousBlock))
                }
                self.children.last_mut().unwrap()
            }
        }
    }
}

enum BoxType<'a> {
    BlockNode(&'a StyleNode<'a>),
    InlineNode(&'a StyleNode<'a>),
    AnonymousBlock,
}

fn build_layout_tree<'a>(style_node: &'a StyleNode<'a>) -> LayoutBox<'a> {
    //create root box
    let mut root = LayoutBox::new(match style_node.display() {
        Display::Block => BlockNode(style_node),
        Display::Inline => InlineNode(style_node),
        Display::None => panic!("Root node has display: none")
    });
    //create descendant boxes
    for child in &style_node.children {
        match child.display() {
            Display::Block => root.children.push(build_layout_tree(child)),
            Display::Inline => root.get_inline_container().children.push(build_layout_tree(child)),
            Display::None => {}
        }
    }

    return root;
}




