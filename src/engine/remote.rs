use time::Duration;

use color::Color;
use engine::{Draw, UpdateFn, Settings};
use keys::{Key, KeyCode};
use point::Point;


pub fn main_loop<T>(display_size: Point,
                    _default_background: Color,
                    _window_title: &str,
                    mut state: T,
                    update: UpdateFn<T>)
{
    let settings = Settings {
        fullscreen: false,
    };
    let mut drawcalls = Vec::with_capacity(4000);

    loop {
        unimplemented!();
        drawcalls.clear();
        match update(state,
                     Duration::milliseconds(16),
                     display_size,
                     60,
                     &[],
                     settings,
                     &mut drawcalls) {
            Some((_new_settings, new_state)) => {
                state = new_state;
            },
            None => break,
        };
    }
}
