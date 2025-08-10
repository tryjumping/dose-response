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

    // NOTE: this gives us the more-or-less alternating 16/8/16/8ms cycles on MacOS
    // Just like we had with winit+glutin.
    // So that is something we want to handle. I don't want to run the update more than every 16ms.
    // And I think that's probably the case to handle: the render loop can run faster than the updates and we need to throttle them.

    // NOTE: on Geralt there seems to be no vsync so we're getting like 300 micro(!!) second updates. So we will have to enable the vsync thing there. Well, this makes for good test cases.

    //
    // NOTE: I was seeing the exact behavior with the "fixed timestep"  algo, so that's not the full story without some sort of modification either

    // Wonder if we could split the looop into three parts:
    // 1. poll events
    // 2. is it time to update? then update
    // 3. render (unconditionally)

    // And actually, if polling the events faster doesn't matter (because we're not updating and it since update/render aren't decoupled now, things like animations or mouse can't be smoother), just incorporate events into game update:
    // 1. is it time to update? then update
    // 2. render (unconditionally)

    // NOTE: this relies on vsync so we will have to put in some waiting to handle cases where vsync is off. I have a repro on Geralt for that right now.

    // From: https://gafferongames.com/post/fix_your_timestep/
    let target_dt = Duration::from_millis((1000.0 / formula::FPS) as u64);
    let mut total_elapsed_time = Duration::ZERO;
    let mut current_time = Instant::now();

    while running {
        let now = Instant::now();
        let dt = now - current_time;
        current_time = now;
        total_elapsed_time += dt;

        game.tick += 1;
        println!("Game update, {dt:?}");

        // TODO: print expected time (tick * target_dt) vs. actual elapsed time (sum(dt));
        // to see if we're getting any frame discrepancy

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

        println!("Render");

        let i = game.cycle;
        canvas.set_draw_color(sdl3::pixels::Color::RGB(i, 64, 255 - i));
        canvas.clear();
        canvas.present();

        let frame_dt = Instant::now().duration_since(current_time);

        dbg!(
            target_dt.as_secs_f64(),
            frame_dt.as_secs_f64(),
            (target_dt.as_secs_f64() - frame_dt.as_secs_f64()).abs()
        );

        // catch up with the target_dt if we ended early
        {
            let ms = 1.0 / 1000.0;

            if frame_dt < target_dt {
                let missing_time = target_dt - frame_dt;
                if missing_time.as_secs_f64() >= ms {
                    // wait
                    std::thread::sleep(missing_time);
                }
            }

            let frame_dt = Instant::now().duration_since(current_time);

            dbg!(
                frame_dt.as_secs_f64(),
                (target_dt.as_secs_f64() - frame_dt.as_secs_f64()).abs()
            );

            if (target_dt.as_secs_f64() - frame_dt.as_secs_f64().abs()) >= ms {
                log::warn!(
                    "Unexpected difference from the fixed frame: {}",
                    target_dt.as_secs_f64() - frame_dt.as_secs_f64()
                );
            }
        }

        // Expectation: dt ~ target_dt
        log::info!(
            "Expected time based on fixed_dt: {}s, actual elapsed time: {}s",
            (game.tick as f64) * target_dt.as_secs_f64(),
            total_elapsed_time.as_secs_f64()
        );

        // TODO Let the game run and see if we're getting significant
        // discrepancies (i.e. if the actual time is getting bigger
        // and bigger than the expectation based on fixed dt.)
        //
        // I think a next step beyond a total sleep time would be to
        // use an accumulator (with a sleep though! Maybe 1ms) to just
        // catch up on any stragglers that way. Pretty much what fix
        // your timestep does + the explicit wait.
        //
        // That way the discrepancy should never be more than 1ms
    }

    // // From: https://gafferongames.com/post/fix_your_timestep/
    // //let fixed_dt = Duration::from_millis((1000.0 / formula::FPS) as u64);
    // let fixed_dt = Duration::from_millis((1000.0 / 120.0) as u64);

    // let mut current_time = Instant::now();
    // let mut accumulator = Duration::ZERO;

    // let mut game_current_time = current_time;

    // // TODO actually maybe just start with a naive loop and then see?
    // // enable sleep for if we don't get vsync
    // // and maybe detect if we're running more than 60 FPS?

    // while running {
    //     let new_time = Instant::now();
    //     let frame_time = new_time - current_time;
    //     current_time = new_time;

    //     accumulator += frame_time;

    //     while running && accumulator >= fixed_dt {
    //         // TODO: I want a "real elapsed" DT here too that we ourselves calculate. Not just the const DT we specified above
    //         {
    //             let now = Instant::now();
    //             let dt = now - game_current_time;
    //             game_current_time = now;

    //             game.tick += 1;
    //             println!("Game update, {dt:?}ms");

    //             game.cycle = game.cycle.wrapping_add(1);
    //             for event in game.event_pump.poll_iter() {
    //                 match event {
    //                     Event::Quit { .. }
    //                     | Event::KeyDown {
    //                         keycode: Some(Keycode::Escape),
    //                         ..
    //                     } => running = false,
    //                     _ => {}
    //                 }
    //             }
    //             // The rest of the game loop goes here...
    //         }

    //         {
    //             // simulate render loop taking 10 milliseconds
    //             println!("Render");

    //             let i = game.cycle;
    //             canvas.set_draw_color(sdl3::pixels::Color::RGB(i, 64, 255 - i));
    //             canvas.clear();
    //             canvas.present();
    //         }

    //         // NOTE: this is problematic as it stands because it calculates the `dt` is fixed and irrespective of the actual time it took. I think at minimum we'd have to wait (if we're faster) to make sure we took at least dt times

    //         // I think I need to actually calculate and test out the usecases: what if the elapsed time is shorter than dt? what if it's longer? what about rendering?
    //         // NOTE: what motivated us to do this in the first place?
    //         // first, super inconsistent dts on macos when waiting for render loops (16, 3, 16, 3, 16 ms IIRC)
    //         // second, it'd be nicer for replays to have constant update framerate no matter what the display framerate did (e.g. always do 60fps update even on 120/144hZ displays)
    //         // later on, would set us up for possibly smoothing animations out too but that's a distant thing concern

    //         // SO first we really need to calculate proper elapsed dt and show it

    //         accumulator -= fixed_dt;
    //     }
    // }

    Ok(())
}
