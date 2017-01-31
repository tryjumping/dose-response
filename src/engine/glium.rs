use std::path::Path;
use time::{Duration, PreciseTime};

use glium::{self, DisplayBuild, Surface};
use glium::draw_parameters::DrawParameters;
use glium::glutin::{Event, WindowBuilder};
use glium::glutin::ElementState as PressState;
use glium::glutin::VirtualKeyCode as GliumKey;
use image;
use image::GenericImage;

use color::Color;
use engine::{Draw, UpdateFn, Settings};
use keys::{Key, KeyCode};
use point::Point;



#[derive(Copy, Clone, Debug)]
struct Vertex {
    /// Position in the tile coordinates.
    ///
    /// Note that this doesn't have to be an integer, so you can
    /// implement smooth positioning by using a fractional value.
    tile_position: [f32; 2],

    /// Index into the texture map. [0, 0] is the top-left corner, the
    /// map's width and height depends on the number of textures in it.
    ///
    /// If a map has 16 textures in a 4x4 square, the top-left index
    /// is [0, 0] and the bottom-right is [3, 3].
    tilemap_index: [f32; 2],

    /// Colour of the glyph. The glyphs are greyscale, so this is how
    /// we set the final colour.
    color: [f32; 3],
}

implement_vertex!(Vertex, tile_position, tilemap_index, color);


pub fn main_loop<T>(display_size: Point,
                    default_background: Color,
                    window_title: &str,
                    font_path: &Path,
                    mut state: T,
                    update: UpdateFn<T>)
{
    let tilesize = 16;  // TODO: don't hardcode this value -- calculate it from the tilemap.
    let (screen_width, screen_height) = (display_size.x as u32 * tilesize as u32,
                                         display_size.y as u32 * tilesize as u32);

    // GL setup

    let display = WindowBuilder::new()
        .with_vsync()
        .with_title(window_title)
        .with_dimensions(screen_width, screen_height)
        .build_glium()
        .expect("dose response ERROR: Could not create the window.");

    let program = glium::Program::from_source(
        &display,
        include_str!("../shader_150.glslv"),
        include_str!("../shader_150.glslf"),
        None).unwrap();

    let texture = {
        let image = image::open(font_path).unwrap().to_rgba();
        let (w, h) = image.dimensions();
        assert_eq!(w % tilesize, 0);
        assert_eq!(h % tilesize, 0);
        let image = glium::texture::RawImage2d::from_raw_rgba(
            image.into_raw(), (w, h));
        glium::texture::SrgbTexture2d::new(&display, image).unwrap()
    };


    // Main loop

    let mut previous_frame_time = PreciseTime::now();

    loop {
        let now = PreciseTime::now();
        let dt = previous_frame_time.to(now);
        previous_frame_time = now;
        //self.world.update(dt, &self.key_events);


        // listing the events produced by the window and waiting to be received
        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,   // the window has been closed by the user
                Event::KeyboardInput(press_state, _, key_code) => {
                    if let Some(code) = key_code {
                        //self.key_events.push((code, press_state));
                    }
                }
                _ => ()
            }
        }

    }
}
