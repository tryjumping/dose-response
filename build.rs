use std::{
    env,
    error::Error,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use image::{Rgba, RgbaImage};
use rusttype::{point, FontCollection, Scale};

#[cfg(windows)]
fn set_exe_icon() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.compile().unwrap();
}

#[cfg(not(windows))]
fn set_exe_icon() {
    // NOTE: do nothing. We're not on Windos so we're not going to set
    // the icon.
}

fn copy_output_artifacts_internal(filename: &str) -> Result<(), Box<dyn Error>> {
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
) -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
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

fn save_out_dir(cargo_manifest_dir: &str, out_dir: &Path) -> Result<(), Box<dyn Error>> {
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

    let font_data = include_bytes!("fonts/mononoki-Regular.ttf");
    let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap();

    // only succeeds if collection consists of one font
    let font = collection.into_font().unwrap();

    // NOTE: generate the constants
    let mut lookup_table_contents = String::new();

    let mut backends = vec![];
    for (key, _value) in std::env::vars_os() {
        if let Some(var) = key.to_str() {
            if var.starts_with("CARGO_FEATURE_") && var.ends_with("_BACKEND") {
                let mut words: Vec<&str> = var.split('_').collect();
                words.pop();
                backends.push(words[2..].join("_").to_lowercase().to_string());
            }
        }
    }

    lookup_table_contents.push_str(&format!(
        "pub const AVAILABLE_BACKENDS: [&'static str; {}] = {:?};\n",
        backends.len(),
        backends,
    ));

    let text_sizes = [28, 21, 16];
    let default_text_size = 21;
    assert!(text_sizes.contains(&default_text_size));

    lookup_table_contents.push_str(&format!(
        "pub const DEFAULT_TEXT_SIZE: i32 = {};\n",
        default_text_size
    ));

    lookup_table_contents.push_str(&format!(
        "pub const AVAILABLE_TEXT_SIZES: [i32; {}] = {:?};\n",
        text_sizes.len(),
        text_sizes,
    ));

    // NOTE: generate the glyph map texture
    {
        // TODO: render a separate glyphmap for the game tiles as opposed to generic text
        // NOTE: We can center them properly and not have to do the position fixup in the game
        let default_tile_size = 30;
        let tile_sizes = [40, 30, 20];
        assert!(tile_sizes.contains(&default_tile_size));

        // TODO: get this out of graphics somehow?
        // Or like, validate all the glyphs from Graphics are covered?
        let tile_chars = " #.@&aDhSviI+x%!".chars().collect::<Vec<_>>();

        // NOTE: recardless of what value we set here, always keep it power of two!
        let texture_width = 512;
        let texture_height = 256;

        lookup_table_contents.push_str(&format!(
            "pub const DEFAULT_TILE_SIZE: i32 = {};\n",
            default_tile_size
        ));

        lookup_table_contents.push_str(&format!(
            "pub const TILE_TEXTURE_WIDTH: u32 = {};\n",
            texture_width as u32
        ));

        lookup_table_contents.push_str(&format!(
            "pub const TILE_TEXTURE_HEIGHT: u32 = {};\n",
            texture_height as u32
        ));

        lookup_table_contents.push_str(&format!(
            "pub const AVAILABLE_TILE_SIZES: [i32; {}] = {:?};\n",
            tile_sizes.len(),
            tile_sizes,
        ));

        // Lookup table for the printable ASCII chars (32 to 125)
        let lookup_table = tile_chars
            .iter()
            .enumerate()
            .map(|(index, &chr)| (index, chr))
            .collect::<Vec<_>>();

        let mut glyphs = vec![];

        let tilemap_offset_x = 0;
        let mut tilemap_offset_y = 0;

        // NOTE: the packing can be made more efficient if we place the
        // biggest glyphs first.
        let all_sizes = {
            let mut sizes: Vec<_> = tile_sizes.into();
            sizes.extend(&text_sizes);
            sizes.sort_by(|a, b| b.cmp(a));
            sizes
        };

        for &size in &all_sizes {
            let height = size as f32;
            let scale = Scale::uniform(height);
            let v_metrics = font.v_metrics(scale);

            let tiles_per_texture_width = texture_width / size;

            let glyphs_iter = lookup_table
                .iter()
                .filter(|&(_index, chr)| tile_chars.contains(chr))
                .map(|&(index, chr)| {
                    let glyph = font.glyph(chr).scaled(scale);
                    let advance_width = glyph.h_metrics().advance_width;
                    let offset_x = (size as f32 - advance_width) / 2.0;
                    let column = index as i32 % tiles_per_texture_width;
                    let line = index as i32 / tiles_per_texture_width;
                    let tilepos_x = column * size + tilemap_offset_x;
                    let tilepos_y = line * size + tilemap_offset_y;
                    let positioned_glyph = glyph.positioned(point(
                        tilepos_x as f32 + offset_x,
                        tilepos_y as f32 + v_metrics.ascent,
                    ));
                    (size, positioned_glyph, chr, tilepos_x, tilepos_y)
                });
            glyphs.extend(glyphs_iter);

            let full_font_width_px = lookup_table.len() as i32 * size;
            let lines = (full_font_width_px as f32 / texture_width as f32).ceil() as i32;
            tilemap_offset_y += size * lines;

            if tilemap_offset_y >= texture_height {
                panic!(
                    "The tile texture size ({}x{}) is not sufficient. Current height: {}",
                    texture_width, texture_height, tilemap_offset_y
                );
            }
        }

        // NOTE: Generate the `texture_coords_from_char` query function
        lookup_table_contents.push_str(
            "pub fn glyph_coords_px_from_char(size: u32, chr: char) -> Option<(i32, i32)> {\n",
        );
        lookup_table_contents.push_str("match (size, chr) {\n");
        for &(font_size, ref _glyph, chr, tilepos_x, tilepos_y) in &glyphs {
            lookup_table_contents.push_str(&format!(
                "  ({:?}, {:?}) => Some(({}, {})),\n",
                font_size, chr, tilepos_x, tilepos_y
            ));
        }

        lookup_table_contents.push_str("_ => None,\n}\n}\n\n");

        // NOTE: Generate the tilemap
        let mut textmap = RgbaImage::new(texture_width as u32, texture_height as u32);

        for (_font_size, g, _chr, _column, _line) in glyphs {
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
                        textmap.put_pixel(x as u32, y as u32, pixel);
                    }
                })
            }
        }

        textmap.save(out_dir.join("glyph.png")).unwrap();
    }

    let mut lt_file = File::create(out_dir.join("glyph_lookup_table.rs")).unwrap();
    lt_file.write_all(lookup_table_contents.as_bytes()).unwrap();

    let vertex_src = include_str!("src/shader_150.glslv");
    let fragment_src = include_str!("src/shader_150.glslf");
    generate_webgl_shaders(out_dir, vertex_src, fragment_src).unwrap();

    // We want these artifacts in the target dir right next to the
    // binaries, not just in the hard-to-find out-dir
    copy_output_artifacts_to_target("glyph.png");
    copy_output_artifacts_to_target("webgl_vertex_shader.glsl");
    copy_output_artifacts_to_target("webgl_fragment_shader.glsl");

    set_exe_icon();
}
