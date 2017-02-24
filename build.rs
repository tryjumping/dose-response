extern crate rusttype;
extern crate image;

use std::env;
use std::io::Write;
use std::fs::File;
use std::path::Path;

use rusttype::{FontCollection, Scale, point, PositionedGlyph};
use image::{Rgba, RgbaImage};

fn main() {

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    // let font_data = include_bytes!("../Arial Unicode.ttf");
    let font_data = include_bytes!("fonts/mononoki-Regular.ttf");
    let collection = FontCollection::from_bytes(font_data as &[u8]);
    let font = collection.into_font().unwrap(); // only succeeds if collection consists of one font


    // Desired font pixel height
    let height: f32 = 16.0;
    let pixel_height = height.ceil() as usize;

    let scale = Scale::uniform(height);

    // The origin of a line of text is at the baseline (roughly where non-descending letters sit).
    // We don't want to clip the text, so we shift it down with an offset when laying it out.
    // v_metrics.ascent is the distance between the baseline and the highest edge of any glyph in
    // the font. That's enough to guarantee that there's no clipping.
    let v_metrics = font.v_metrics(scale);

    // NOTE: To lay out text, use the `font.layout` call below. It
    // should handle glyph positioning, kerning, etc.:
    // let offset = point(0.0, v_metrics.ascent);
    // let glyphs: Vec<PositionedGlyph> = font.layout("RustType", scale, offset).collect();

    // Lookup table for the printable ASCII chars (32 to 126)
    let lookup_table = (32u8..127)
        .enumerate()
        .map(|(index, ascii_code)| (index, ascii_code as char))
        .collect::<Vec<_>>();

    let mut lookup_table_fn_definition = String::new();

    lookup_table_fn_definition.push_str(
        "fn texture_coords_from_char(chr: char) -> Option<(i32, i32)> {\n");
    lookup_table_fn_definition.push_str(
        "match chr {\n");
    for &(index, chr) in &lookup_table {
        lookup_table_fn_definition.push_str(
            &format!("  {:?} => Some(({}, 0)),\n", chr, index));
    }
    lookup_table_fn_definition.push_str("_ => None,\n}\n}\n");

    let mut lt_file = File::create(out_dir.join("glyph_lookup_table.rs")).unwrap();
    lt_file.write_all(lookup_table_fn_definition.as_bytes()).unwrap();


    // TODO: center the characters horizontaly
    let glyphs: Vec<PositionedGlyph> = lookup_table.iter()
        .map(|&(index, chr)| font.glyph(chr).unwrap().scaled(scale)
             .positioned(point(height * index as f32, v_metrics.ascent)))
        .collect();

    let width = pixel_height * glyphs.iter().count();

    // TODO: when rendering a layd out text:
    // Find the most visually pleasing width to display
    // let width = glyphs.iter().rev()
    //     .filter_map(|g| g.pixel_bounding_box()
    //                 .map(|b| b.min.x as f32 + g.unpositioned().h_metrics().advance_width))
    //     .next().unwrap_or(0.0).ceil() as usize;

    println!("width: {}, height: {}", width, pixel_height);

    let mut fontmap = RgbaImage::new(width as u32, pixel_height as u32);

    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            // Center the glyphs horizontally within their tile
            let x_offset = bb.width() / 2;
            g.draw(|x, y, v| {
                let x = x as i32 + bb.min.x + x_offset;
                let y = y as i32 + bb.min.y;
                // There's still a possibility that the glyph clips the boundaries of the bitmap
                if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                    let alpha = (v * 255.0) as u8;
                    let pixel = Rgba {
                        data: [255, 255, 255, alpha]
                    };
                    fontmap.put_pixel(x as u32, y as u32, pixel);
                }
            })
        }
    }

    if let Err(e) = fontmap.save(out_dir.join("font.png")) {
        println!("Error while saving the font map: '{}'", e);
    }

}
