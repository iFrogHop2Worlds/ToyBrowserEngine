use crate::css::Unit::Px;
use crate::css::Value::{Keyword, Length};
use crate::layout::BoxType::{AnonymousBlock, BlockNode, InlineNode};
use crate::style::{StyleNode, Display};
#[derive(Clone, Copy, Default, Debug)]
struct Dimensions {
    content: Rect,
    padding: EdgeSizes,
    border: EdgeSizes,
    margin: EdgeSizes
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub height: f32,
    pub width: f32
}

#[derive(Default, Clone, Copy, Debug)]
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
    fn get_style_node(&self) -> &'a StyleNode<'a> {
        match self.box_type {
            BlockNode(node) | InlineNode(node) => node,
            AnonymousBlock => panic!("Anonymous block box has no style node")
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

    fn layout(&mut self, containing_block: &mut Dimensions) {
        match self.box_type {
            BlockNode(_) => self.layout_block(containing_block),
            InlineNode(_) => {}, //todo
            AnonymousBlock => {} //todo
        }
    }
    /// This function performs a single traversal of the layout tree, doing width
    /// calculations on the way down and height calculations on the way back up.
    fn layout_block(&mut self, containing_block: &mut Dimensions) {
        // Child width can depend on parents width, so we need to calc
        // this box width before laying out its children
        self.calculate_block_width(containing_block.clone());

        // Determine where the box is located within its container
        self.calculate_block_position(containing_block.clone());

        // recursively lay out the children of this box
        self.layout_block_children();

        // Parent height can depend on the child height so calculate_hieght
        // must be called after the children are laid out
        self.calculate_block_height();
    }

    fn calculate_block_width(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();
        // width has an init value auto
        let auto = Keyword("auto".to_string());
        let mut width = style.value("width").unwrap_or(auto.clone());
        // margin, border and padding hav init val 0
        let zero = Length(0.0, Px);

        let mut margin_left = style.lookup("margin-left", "margin", &zero);
        let mut margin_right = style.lookup("margin-right", "margin", &zero);

        let border_left = style.lookup("border-left-width", "border-width", &zero);
        let border_right = style.lookup("border-right-width", "border_width", &zero);

        let padding_left = style.lookup("padding-left", "padding", &zero);
        let padding_right = style.lookup("padding-right", "padding", &zero);

        let total: f32 = sum([&margin_left, &margin_right, &border_left, &border_right,
            &padding_left, &padding_right, &width].iter().map(|v| v.to_px()));

        // if width is not auto and the total is wider than the container, treat auto margin as 0
        if width != auto && total > containing_block.content.width {
            if margin_left == auto {
                margin_left = Length(0.0, Px);
            }
            if margin_right == auto {
                margin_right = Length(0.0, Px);
            }
        }

        let underflow = containing_block.content.width - total; // the amount of space left in the container
        // https://www.w3.org/TR/CSS2/visudet.html#blockwidth - algorithm
        match(width == auto, margin_left == auto, margin_right == auto) {
            // if the values are overconstrained, calc margin_right.
            (false, false, false) => {
                margin_right = Length(margin_right.to_px() + underflow, Px);
            }
            // if exactly one size is auto its used value follows from the equality.
            (false, false, true) => { margin_left = Length(underflow, Px); }
            (false, true, false) => { margin_right = Length(underflow, Px); }
            // if width is auto then all other auto values become 0
            (true, _, _) => {
                if margin_left == auto { margin_left = Length(0.0, Px); }
                if margin_right == auto { margin_right = Length(0.0, Px); }

                if underflow >= 0.0 {
                    // Expand width to fill the underflow.
                    width = Length(underflow, Px);
                } else {
                    // Width can't be negative. Adjust the right margin instead.
                    width = Length(0.0, Px);
                    margin_right = Length(margin_right.to_px() + underflow, Px);
                }
            }
            // if marin-left and margin-right are both auto their used values are equal
            (false, true, true) => {
                margin_left = Length(underflow / 2.0, Px);
                margin_right = Length(underflow / 2.0, Px);
            }
        }
    }

    pub fn calculate_block_position(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();
        let d = &mut self.dimensions;

        // margin border and padding have init value 0
        let zero = Length(0.0, Px);

        // If margin to or margin bottom is auto the use value is zero
        d.margin.top = style.lookup("margin-top", "margin", &zero).to_px();
        d.margin.bottom = style.lookup("margin-bottom", "margin", &zero).to_px();

        d.border.top = style.lookup("border-top-width", "border-width", &zero).to_px();
        d.border.bottom = style.lookup("border-bottom-width", "border-width", &zero).to_px();

        d.padding.top = style.lookup("padding-top", "padding", &zero).to_px();
        d.padding.bottom = style.lookup("padding-bottom", "padding", &zero).to_px();

        d.content.x = containing_block.content.x +
            d.margin.left + d.border.left + d.padding.left;

        // Position the box below all the prev boxes in the container
        d.content.y = containing_block.content.height + containing_block.content.y +
                        d.margin.top + d.border.top + d.padding.top;
    }

    fn layout_block_children(&mut self) {
        let d = &mut self.dimensions;
        for child in &mut self.children {
            child.layout(d);
            // Track the height so each child is laid out below the previous content.
            d.content.height = d.content.height + child.dimensions.margin_box().height;
        }
    }

    fn calculate_block_height(&mut self) {
        if let Some(Length(h, Px)) = self.get_style_node().value("height") {
            self.dimensions.content.height = h;
        }
    }
}

impl Dimensions {
    // the area covered by the content area plus its padding.
    fn padding_box(self) -> Rect {
        self.content.expanded_by(self.padding)
    }
    // the area covered by the content area plus its padding and borders.
    fn border_box(self) -> Rect {
        self.padding_box().expanded_by(self.border)
    }
    // the area covered by the content area plus its padding, borders and margin
    fn margin_box(self) -> Rect {
        self.border_box().expanded_by(self.margin)
    }
}

impl Rect {
    fn expanded_by(self, edge: EdgeSizes) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
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

fn sum<I>(iter: I) -> f32 where I: Iterator<Item=f32> {
    iter.fold(0., |a, b| a + b)
}


