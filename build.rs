extern crate rusttype;
extern crate image;

use std::env;
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

    // TODO: think about centering each character?
    let text = "@.+x# RustType";
    let glyphs: Vec<PositionedGlyph> = font.glyphs_for(text.chars())
        .enumerate()
        .map(|(index, glyph)| glyph.scaled(scale)
             .positioned(point(height * index as f32, v_metrics.ascent)))
        .collect();

    let width = pixel_height * text.chars().count();

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
            g.draw(|x, y, v| {
                let x = x as i32 + bb.min.x;
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

    if let Err(e) = fontmap.save(out_dir.join("out.png")) {
        println!("Error while saving the font map: '{}'", e);
    }

}
