use std::{
    env,
    error::Error,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use image::{Rgba, RgbaImage};
use rusttype::{point, FontCollection, PositionedGlyph, Scale};

fn copy_output_artifacts_internal(filename: &str) -> Result<(), Box<Error>> {
    // NOTE: this is a hack to save the font file next to the produced build binary
    let target_triple = env::var("TARGET")?;
    let host_triple = env::var("HOST")?;
    let out_dir = env::var("OUT_DIR")?;
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR")?;

    let mut src = PathBuf::new();
    src.push(out_dir);
    src.push(filename);

    let mut dst = PathBuf::new();
    dst.push(cargo_manifest_dir);
    dst.push("target");
    if target_triple != host_triple {
        dst.push(target_triple)
    }
    dst.push(env::var("PROFILE")?);
    dst.push(filename);

    ::std::fs::copy(src, dst)?;
    Ok(())
}

fn copy_output_artifacts_to_target(filename: &str) {
    println!("Attempting to copy {}", filename);
    if let Err(e) = copy_output_artifacts_internal(filename) {
        println!("Warning: could not copy output artifacts to the target directory.");
        println!("{:?}", e);
    }
}

fn webgl_from_desktop(desktop_shader: &str, replacements: &[(&str, &str)]) -> String {
    let mut tmp: String = desktop_shader.into();
    for (pattern, replacement) in replacements {
        tmp = tmp.replace(pattern, replacement);
    }

    tmp
}

fn generate_webgl_shaders(
    out_dir: &Path,
    vertex_src: &str,
    fragment_src: &str,
) -> Result<(PathBuf, PathBuf), Box<Error>> {
    let vertex_replacements = &[
        ("#version 150 core\n", ""),
        ("in vec2", "attribute vec2"),
        ("in vec3", "attribute vec3"),
        ("in vec4", "attribute vec4"),
        ("out vec2", "varying vec2"),
        ("out vec3", "varying vec3"),
        ("out vec4", "varying vec4"),
    ];

    let fragment_replacements = &[
        ("out vec4 out_color;", ""),
        ("#version 150 core", "precision mediump float;"),
        ("in vec2", "varying vec2"),
        ("in vec3", "varying vec3"),
        ("in vec4", "varying vec4"),
        ("out vec2", "varying vec2"),
        ("out vec3", "varying vec3"),
        ("out vec4", "varying vec4"),
        ("out_color", "gl_FragColor"),
        ("texture(", "texture2D("),
    ];

    let shader = webgl_from_desktop(vertex_src, vertex_replacements);
    let vs_path = out_dir.join("webgl_vertex_shader.glsl");
    let mut file = File::create(&vs_path)?;
    file.write_all(shader.as_bytes())?;
    file.sync_all()?;

    let shader = webgl_from_desktop(fragment_src, fragment_replacements);
    let fs_path = out_dir.join("webgl_fragment_shader.glsl");
    let mut file = File::create(&fs_path)?;
    file.write_all(shader.as_bytes())?;
    file.sync_all()?;

    Ok((vs_path, fs_path))
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

fn current_git_commit() -> Option<String> {
    Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
}

fn main() {
    let git_hash = env::var_os("APPVEYOR_REPO_COMMIT")
        .or(env::var_os("TRAVIS_COMMIT"))
        .and_then(|s| s.into_string().ok())
        .or_else(current_git_commit)
        .unwrap_or_default();
    println!("cargo:rustc-env=DR_GIT_HASH={}", git_hash);
    println!(
        "cargo:rustc-env=DR_TARGET_TRIPLE={}",
        env::var("TARGET").unwrap_or_default()
    );

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let _ = save_out_dir(&cargo_manifest_dir, out_dir);

    // let font_data = include_bytes!("../Arial Unicode.ttf");
    let font_data = include_bytes!("fonts/mononoki-Regular.ttf");
    let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap();

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

    let h_metrics = lookup_table
        .iter()
        .map(|&(_index, chr)| font.glyph(chr).scaled(scale).h_metrics().advance_width);

    let glyphs: Vec<PositionedGlyph> = lookup_table
        .iter()
        .map(|&(index, chr)| {
            font.glyph(chr)
                .scaled(scale)
                .positioned(point(height * index as f32, v_metrics.ascent))
        })
        .collect();

    let texture_width = pixel_height * glyphs.iter().count();
    let texture_height = pixel_height;

    println!(
        "texture width: {}, texture height: {}",
        texture_width, texture_height
    );

    let mut lookup_table_contents = String::new();

    lookup_table_contents.push_str(&format!("pub const TILESIZE: u32 = {};\n", height as u32));
    lookup_table_contents.push_str(&format!(
        "pub const TEXTURE_WIDTH: u32 = {};\n",
        texture_width as u32
    ));
    lookup_table_contents.push_str(&format!(
        "pub const TEXTURE_HEIGHT: u32 = {};\n",
        texture_height as u32
    ));

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

    fontmap.save(out_dir.join("font.png")).unwrap();

    let vertex_src = include_str!("src/shader_150.glslv");
    let fragment_src = include_str!("src/shader_150.glslf");
    generate_webgl_shaders(out_dir, vertex_src, fragment_src).unwrap();

    // We want these artifacts in the target dir right next to the
    // binaries, not just in the hard-to-find out-dir
    copy_output_artifacts_to_target("font.png");
    copy_output_artifacts_to_target("webgl_vertex_shader.glsl");
    copy_output_artifacts_to_target("webgl_fragment_shader.glsl");
}
