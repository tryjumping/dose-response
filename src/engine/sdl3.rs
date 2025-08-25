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

use sdl3::{event::Event, keyboard::Keycode, render::Canvas, video::Window};

use egui::{ClippedPrimitive, Context};

use rodio::OutputStream;

struct Game {
    cycle: u8,
    tick: u32,
    event_pump: sdl3::EventPump,
}

impl Game {
    fn update_and_render(&mut self, dt: Duration, canvas: &mut Canvas<Window>) -> bool {
        println!("Game update, {dt:?}");
        self.tick += 1;

        self.cycle = self.cycle.wrapping_add(1);
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return false,
                _ => {}
            }
        }

        let i = self.cycle;
        canvas.set_draw_color(sdl3::pixels::Color::RGB(i, 64, 255 - i));
        canvas.clear();
        canvas.present();

        true
    }
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

    // NOTE: you should be able to use opengl here just like with SDL2 I think! This function still exists:
    // https://docs.rs/sdl3/latest/sdl3/video/struct.Window.html#method.gl_create_context

    // So look at the sdl2.rs code and see what that does. And then try to replicate that here.

    // https://docs.rs/sdl3/latest/sdl3/video/gl_attr/index.html

    // NOTe: but I think we should start with integrating the game update loop first. So we have something to point at the rendering pipeline

    let window = video_subsystem
        .window(window_title, 800, 600)
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

    let mut game = Game {
        cycle: 0,
        tick: 0,
        event_pump,
    };

    let target_dt_nanoseconds = 1_000_000_000 / (formula::FPS as u32);
    let target_dt = Duration::new(0, target_dt_nanoseconds);
    dbg!(target_dt);

    // NOTE: this sets the boundary (variance) between actual `dt` and
    // "fixed `dt`" based on the target FPS.
    //
    // 1ms variance is probably totally fine, actually, but if we need
    // more precision, we can just supply a smaller number here.
    let inc = Duration::new(0, 1_000_000); // 1ms
    dbg!(inc);

    let start_time = Instant::now();
    let mut current_time = start_time;

    let mut running = true;
    while running {
        let elapsed_time = Instant::now().duration_since(start_time);
        let update_ready = (elapsed_time + inc) >= (game.tick * target_dt);

        if update_ready {
            let now = Instant::now();
            let dt = now.duration_since(current_time);
            current_time = now;

            running = game.update_and_render(dt, &mut canvas);

            let frame_dt = Instant::now().duration_since(current_time);

            // Make sure we advance at least by 1ms. Otherwise
            // we risk triggering update multiple times in a row if it
            // takes less than `inc` time.
            let ms = Duration::from_millis(1);
            debug_assert!(
                inc <= ms,
                "The frame catch-up increment must be smaller than 1ms"
            );
            if frame_dt < ms {
                std::thread::sleep(ms);
            }

            log::info!(
                "Total frame duration: {:?}",
                Instant::now().duration_since(current_time)
            );

            // Expectation: dt ~ target_dt
            log::info!(
                "Expected time based on fixed_dt: {:?}, actual elapsed time: {:?}",
                target_dt * game.tick,
                Instant::now().duration_since(start_time)
            );
        } else {
            // Catch up to the next scheduled game update (based on
            // `target_dt`) one increment at a time:
            std::thread::sleep(inc);

            // NOTE: I found this to be much more precise, responsive
            // (and more precisely controllable) than calculating for
            // how long to sleep until the next frame starts.
        };
    }

    Ok(())
}
