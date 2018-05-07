extern crate image;
extern crate rusttype;

use image::{Rgba, RgbaImage};

use rusttype::{point, FontCollection, PositionedGlyph, Scale};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};


fn copy_output_artifacts(cargo_manifest_dir: &str, fontmap: &RgbaImage) -> Result<(), Box<Error>> {
    // NOTE: this is a hack to save the font file next to the produced build binary
    let target_triple = env::var("TARGET")?;
    let host_triple = env::var("HOST")?;
    let mut target_dir = PathBuf::new();
    target_dir.push(cargo_manifest_dir);
    target_dir.push("target");
    if target_triple != host_triple {
        target_dir.push(target_triple)
    }
    target_dir.push(env::var("PROFILE")?);
    target_dir.push("font.png");
    fontmap.save(target_dir)?;
    Ok(())
}

fn save_out_dir(cargo_manifest_dir: &str, out_dir: &Path) -> Result<(), Box<Error>> {
    // Store the OUT_DIR value to the `out-dir-path` file so it's
    // accessible to scripts that run after the build.
    let path = Path::new(&cargo_manifest_dir).join("out-dir-path");
    let mut file = File::create(path)?;
    writeln!(file, "{}", out_dir.display())?;
    file.sync_all()?;
    Ok(())
}


fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let _ = save_out_dir(&cargo_manifest_dir, out_dir);


    // let font_data = include_bytes!("../Arial Unicode.ttf");
    let font_data = include_bytes!("fonts/mononoki-Regular.ttf");
    let collection = FontCollection::from_bytes(font_data as &[u8]);

    // only succeeds if collection consists of one font
    let font = collection.into_font().unwrap();

    // Desired font pixel height
    let height: f32 = 21.0;
    let pixel_height = height.ceil() as usize;

    let scale = Scale::uniform(height);

    // The origin of a line of text is at the baseline (roughly where
    // non-descending letters sit). We don't want to clip the text, so
    // we shift it down with an offset when laying it out.
    // v_metrics.ascent is the distance between the baseline and the
    // highest edge of any glyph in the font. That's enough to
    // guarantee that there's no clipping.
    let v_metrics = font.v_metrics(scale);

    // NOTE: To lay out text, use the `font.layout` call below. It
    // should handle glyph positioning, kerning, etc.: let offset =
    // point(0.0, v_metrics.ascent); let glyphs: Vec<PositionedGlyph>
    // = font.layout("RustType", scale, offset).collect();

    // Lookup table for the printable ASCII chars (32 to 126)
    let lookup_table = (32u8..127)
        .enumerate()
        .map(|(index, ascii_code)| (index, ascii_code as char))
        .collect::<Vec<_>>();

    let h_metrics = lookup_table.iter().map(|&(_index, chr)| {
        font.glyph(chr)
            .unwrap()
            .scaled(scale)
            .h_metrics()
            .advance_width
    });

    let glyphs: Vec<PositionedGlyph> = lookup_table
        .iter()
        .map(|&(index, chr)| {
            font.glyph(chr)
                .unwrap()
                .scaled(scale)
                .positioned(point(height * index as f32, v_metrics.ascent))
        })
        .collect();

    let texture_width = pixel_height * glyphs.iter().count();
    let texture_height = pixel_height;

    println!("texture width: {}, texture height: {}", texture_width, texture_height);

    let mut lookup_table_contents = String::new();


    lookup_table_contents.push_str(&format!("pub const TILESIZE: u32 = {};\n", height as u32));
    lookup_table_contents.push_str(&format!("pub const TEXTURE_WIDTH: u32 = {};\n", texture_width as u32));
    lookup_table_contents.push_str(&format!("pub const TEXTURE_HEIGHT: u32 = {};\n", texture_height as u32));

    lookup_table_contents
        .push_str("fn texture_coords_from_char(chr: char) -> Option<(i32, i32)> {\n");
    lookup_table_contents.push_str("match chr {\n");
    for &(index, chr) in &lookup_table {
        lookup_table_contents.push_str(&format!("  {:?} => Some(({}, 0)),\n", chr, index));
    }
    lookup_table_contents.push_str("_ => None,\n}\n}\n\n");

    // NOTE: uncomment this if we need this. For now we always align lines to the tiles.
    //lookup_table_contents.push_str(&format!("pub const VERTICAL_ASCENT: i32 = {};\n\n", v_metrics.ascent as i32));

    lookup_table_contents.push_str("pub fn glyph_advance_width(chr: char) -> Option<i32> {\n");
    lookup_table_contents.push_str("match chr {\n");

    for (&(_index, chr), advance_width) in lookup_table.iter().zip(h_metrics) {
        lookup_table_contents.push_str(&format!(
            "    {:?} => Some({}),\n",
            chr, advance_width as i32
        ));
    }

    lookup_table_contents.push_str("_ => None,\n}\n\n");
    lookup_table_contents.push_str("}\n");

    let mut lt_file = File::create(out_dir.join("glyph_lookup_table.rs")).unwrap();
    lt_file.write_all(lookup_table_contents.as_bytes()).unwrap();

    let mut fontmap = RgbaImage::new(texture_width as u32, texture_height as u32);

    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;
                // There's still a possibility that the glyph clips
                // the boundaries of the bitmap
                if x >= 0 && x < texture_width as i32 && y >= 0 && y < texture_height as i32 {
                    let alpha = (v * 255.0) as u8;
                    let pixel = Rgba {
                        data: [255, 255, 255, alpha],
                    };
                    fontmap.put_pixel(x as u32, y as u32, pixel);
                }
            })
        }
    }

    if let Err(e) = fontmap.save(out_dir.join("font.png")) {
        println!("Error while saving the font map: '{}'", e);
    }

    if let Err(e) = copy_output_artifacts(&cargo_manifest_dir, &fontmap) {
        println!("Warning: could not copy output artifacts to the target directory.");
        println!("{:?}", e);
    }
}
