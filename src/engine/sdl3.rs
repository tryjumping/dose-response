#![allow(unused_imports)]

use crate::{
    color::Color,
    engine::{
        self, Vertex,
        loop_state::{self, LoopState, ResizeWindowAction, UpdateResult},
        opengl::OpenGlApp,
    },
    formula, keys,
    point::Point,
    settings::{MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH, Store as SettingsStore},
    state::State,
};

use std::{
    error::Error,
    num::NonZeroU32,
    time::{Duration, Instant},
};

use sdl3::{event::Event, keyboard::Keycode};

use egui::{ClippedPrimitive, Context};

use rodio::OutputStream;

struct Game {
    cycle: u8,
    tick: u64,
    event_pump: sdl3::EventPump,
}

pub fn main_loop<S>(
    initial_default_background: Color,
    window_title: &str,
    settings_store: S,
    initial_state: Box<State>,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: SettingsStore + 'static,
{
    let egui_context = Context::default();

    // TODO: do we need this given SDL audio system?

    // NOTE: we need to store the stream to a variable here and then
    // match on a reference to it. Otherwise, it will be dropped and
    // the stream will close.
    let stream_result = OutputStream::try_default();
    let stream_handle = match &stream_result {
        Ok((_stream, stream_handle)) => Some(stream_handle),
        Err(error) => {
            log::error!("Cannot open the audio output stream: {:?}", error);
            None
        }
    };

    let loop_state = LoopState::initialise(
        settings_store.load(),
        initial_default_background,
        initial_state,
        egui_context,
        stream_handle,
    );

    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl3 demo", 800, 600)
        .position_centered()
        .build()?;

    // TODO: set window icon
    // {
    //     // requires "--features 'image'"
    //     use sdl3::surface::Surface;

    //     let window_icon = Surface::from_file("../../assets/icon_256x256.png")?;
    //     window.set_icon(window_icon);
    // }

    // TODO: set window min size
    // {
    //     // TODO: these are winit calls, translate to SDL!
    //     window.with_min_inner_size(LogicalSize::new(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT));
    //     window.with_inner_size(desired_size);
    // }

    // TODO: set up the OpenGL context
    let mut canvas = window.into_canvas();

    canvas.set_draw_color(sdl3::pixels::Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let event_pump = sdl_context.event_pump()?;
    let mut running = true;

    let mut game = Game {
        cycle: 0,
        tick: 0,
        event_pump,
    };

    // From: https://gafferongames.com/post/fix_your_timestep/
    // TODO: rename to `target_dt`?
    let dt = Duration::from_millis((1000.0 / formula::FPS) as u64);

    let mut current_time = Instant::now();
    let mut accumulator = Duration::ZERO;

    while running {
        let new_time = Instant::now();
        let frame_time = new_time - current_time;
        current_time = new_time;

        accumulator += frame_time;

        while running && accumulator >= dt {
            // TODO: I want a "real elapsed" DT here too that we ourselves calculate. Not just the const DT we specified above
            {
                game.tick += 1;
                println!("Game update");

                game.cycle = game.cycle.wrapping_add(1);
                for event in game.event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => running = false,
                        _ => {}
                    }
                }
                // The rest of the game loop goes here...
            }

            {
                // simulate render loop taking 10 milliseconds
                println!("Render");

                let i = game.cycle;
                canvas.set_draw_color(sdl3::pixels::Color::RGB(i, 64, 255 - i));
                canvas.clear();
                canvas.present();
            }

            accumulator -= dt;
        }

        // TODO: render here (but actually, we'll likely just render in the game loop unless there's some frame throttling here)
    }

    Ok(())
}
