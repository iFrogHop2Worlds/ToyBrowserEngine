use std::fs::File;
use std::env;
use std::default::Default;
use std::path::Path;
use getopts::{Options, Matches};
use image::{ImageBuffer, Rgba};
use image::DynamicImage::ImageRgba8;

mod lib;
use lib::*;


fn main() {
    // Parse command-line options:
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("h", "html", "HTML document", "FILENAME");
    opts.optopt("c", "css", "CSS stylesheet", "FILENAME");
    opts.optopt("o", "output", "Output file", "FILENAME");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!("{}", f.to_string())
    };

    // Read input files:
    let read_source = |arg_filename: Option<String>, default_filename: &str| {
        let path = match arg_filename {
            Some(filename) => filename,
            None => default_filename.to_string(),
        };
        std::fs::read_to_string(Path::new(&path)).unwrap()
    };
    let html = read_source(matches.opt_str("h"), "examples/test.html");
    let css  = read_source(matches.opt_str("c"), "examples/test.css");
    println!("css -> {}", &css);
    // Since we don't have an actual window, hard-code the "viewport" size.
    let initial_containing_block = layout::Dimensions {
        content: layout::Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 },
        padding: Default::default(),
        border: Default::default(),
        margin: Default::default(),
    };

    // Parsing and rendering:
    let root_node = html::parse(html);
    let stylesheet = css::parse(css);
    let style_root = style::style_tree(&root_node, &stylesheet);
    let layout_root = layout::layout_tree(&style_root, initial_containing_block);
    let canvas = painting::paint(&layout_root, initial_containing_block.content);

    // Create the output file:
    let filename = matches.opt_str("o").unwrap_or_else(|| "output.png".to_string());
    let filePath = Path::new(&filename);

    // Save an image:
    let (w, h) = (canvas.width as u32, canvas.height as u32);
    let buffer: Vec<Rgba<u8>> = unsafe { std::mem::transmute(canvas.pixels) };
    let img = ImageBuffer::from_fn(w, h, |x, y| buffer[(y * w + x) as usize]);

    let result = ImageRgba8(img).save(filePath);
    match result {
        Ok(_) => println!("Saved output as {}", filename),
        Err(_) => println!("Error saving output as {}", filename)
    }

    // Debug output:
    // println!("{}", layout_root.dimensions);
    // println!("{}", display_list);
}
