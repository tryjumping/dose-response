use engine::{self, Draw, Mouse, Settings};
use game::{self, RunningState};
use keys::{Key, KeyCode};
use point::Point;
use rect::Rectangle;
use state::State;

use std::mem;
use std::time::Duration;

use serde::Serialize;
use rmps::Serializer;


extern {
    fn draw(nums: *const u8, len: usize);
    pub fn random() -> f32;
    fn wrapped_text_height_in_tiles(text_ptr: *const u8, text_len: usize, max_width_in_tiles: i32) -> i32;
}


fn key_code_from_backend(js_keycode: u32) -> Option<KeyCode> {
    use keys::KeyCode::*;
    // NOTE: these values correspond to:
    // https://git.gnome.org/browse/gtk+/plain/gdk/gdkkeysyms.h
    match js_keycode {
        // Numbers in ASCII
        48 => Some(D0),
        49 => Some(D1),
        50 => Some(D2),
        51 => Some(D3),
        52 => Some(D4),
        53 => Some(D5),
        54 => Some(D6),
        55 => Some(D7),
        56 => Some(D8),
        57 => Some(D9),

        // Uppercase letters in ASCII
        65 => Some(A),
        66 => Some(B),
        67 => Some(C),
        68 => Some(D),
        69 => Some(E),
        70 => Some(F),
        71 => Some(G),
        72 => Some(H),
        73 => Some(I),
        74 => Some(J),
        75 => Some(K),
        76 => Some(L),
        77 => Some(M),
        78 => Some(N),
        79 => Some(O),
        80 => Some(P),
        81 => Some(Q),
        82 => Some(R),
        83 => Some(S),
        84 => Some(T),
        85 => Some(U),
        86 => Some(V),
        87 => Some(W),
        88 => Some(X),
        89 => Some(Y),
        90 => Some(Z),

        // Lowercase letters in ASCII
        97 => Some(A),
        98 => Some(B),
        99 => Some(C),
        100 => Some(D),
        101 => Some(E),
        102 => Some(F),
        103 => Some(G),
        104 => Some(H),
        105 => Some(I),
        106 => Some(J),
        107 => Some(K),
        108 => Some(L),
        109 => Some(M),
        110 => Some(N),
        111 => Some(O),
        112 => Some(P),
        113 => Some(Q),
        114 => Some(R),
        115 => Some(S),
        116 => Some(T),
        117 => Some(U),
        118 => Some(V),
        119 => Some(W),
        120 => Some(X),
        121 => Some(Y),
        122 => Some(Z),

        0xFFB0 => Some(NumPad0),
        0xFFB1 => Some(NumPad1),
        0xFFB2 => Some(NumPad2),
        0xFFB3 => Some(NumPad3),
        0xFFB4 => Some(NumPad4),
        0xFFB5 => Some(NumPad5),
        0xFFB6 => Some(NumPad6),
        0xFFB7 => Some(NumPad7),
        0xFFB8 => Some(NumPad8),
        0xFFB9 => Some(NumPad9),

        0xFFBE => Some(F1),
        0xFFBF => Some(F2),
        0xFFC0 => Some(F3),
        0xFFC1 => Some(F4),
        0xFFC2 => Some(F5),
        0xFFC3 => Some(F6),
        0xFFC4 => Some(F7),
        0xFFC5 => Some(F8),
        0xFFC6 => Some(F9),
        0xFFC7 => Some(F10),
        0xFFC8 => Some(F11),
        0xFFC9 => Some(F12),

        0xFF51 => Some(Left),
        0xFF53 => Some(Right),
        0xFF52 => Some(Up),
        0xFF54 => Some(Down),

        0xFF0D => Some(Enter),
        32 => Some(Space),
        0xFF1B => Some(Esc),

        0x03F => Some(QuestionMark),

        _ => None,
    }
}


#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn key_pressed(
    state_ptr: *mut State,
    external_code: u32,
    ctrl: bool, alt: bool, shift: bool
)
{
    let mut state: Box<State> = unsafe { Box::from_raw(state_ptr) };

    let code = key_code_from_backend(external_code);
    if let Some(code) = code {
        state.keys.push(Key { code, alt, ctrl, shift});
    }

    mem::forget(state);
}


#[no_mangle]
pub extern "C" fn initialise() -> *mut State {
    println!("Initialising {} for WebAssembly", ::GAME_TITLE);
    let state = {
        Box::new(State::new_game(
            ::WORLD_SIZE,
            ::DISPLAYED_MAP_SIZE,
            ::PANEL_WIDTH,
            ::DISPLAY_SIZE,
            false,  // exit-after
            None,  // replay file
            false,  // invincible
        ))
    };

    Box::into_raw(state)
}


#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn update(state_ptr: *mut State, dt_ms: u32) {
    let mut state: Box<State> = unsafe { Box::from_raw(state_ptr) };

    let dt = Duration::from_millis(dt_ms as u64);
    let display_size = Point::new(0, 0);
    let fps = 60;
    let keys: Vec<Key> = vec![];
    let mouse: Mouse = Default::default();
    let mut settings = Settings{ fullscreen: false };
    let mut drawcalls: Vec<Draw> = vec![];

    let result = game::update(
        &mut state,
        dt,
        display_size,
        fps,
        &keys,
        mouse,
        &mut settings,
        &mut drawcalls,
    );

    match result {
        RunningState::Running => {}
        RunningState::NewGame(new_state) => {
            *state = new_state;
        }
        RunningState::Stopped => {},
    }

    engine::sort_drawcalls(&mut drawcalls, 0..);

    let mut js_drawcalls = Vec::with_capacity(drawcalls.len() * 6);

    let mut buffer: Vec<u8> = vec![42; 100];

    for drawcall in &drawcalls {
        buffer.clear();
        drawcall.serialize(&mut Serializer::new(&mut buffer)).unwrap();
        js_drawcalls.extend(&buffer);
    }

    unsafe {
        draw(js_drawcalls.as_ptr(), js_drawcalls.len());
    }

    mem::forget(state);
}
