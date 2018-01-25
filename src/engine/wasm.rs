use color::Color;
use engine::{self, Draw, Mouse, TextMetrics, Settings};
use game::{self, RunningState};
use keys::{Key, KeyCode};
use point::Point;
use state::State;

use std::mem;
use std::time::Duration;

use serde::Serialize;
use rmps::Serializer;


const BUFFER_CAPACITY: usize = 400;
const JS_DRAWCALL_CAPACITY: usize = 80_000;


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


struct Metrics;

impl TextMetrics for Metrics {
    fn get_text_height(&self, text_drawcall: &Draw) -> i32 {
        match text_drawcall {
            &Draw::Text(_pos, ref text, _color, options) => {
                if options.wrap && options.width > 0 {
                    #[allow(unsafe_code)]
                    unsafe {
                        wrapped_text_height_in_tiles(text.as_ptr(), text.len(), options.width)
                    }
                } else {
                    1
                }
            }
            _ => {
                panic!("The argument to `TextMetrics::get_text_height` must be `Draw::Text`!");
            },
        }
    }
}


/// Struct holding most of our games' memory. Everything we could
/// preallocate is here.
pub struct Wasm {
    state: *mut State,
    buffer: *mut Vec<u8>,
    drawcalls: *mut Vec<Draw>,
    js_drawcalls: *mut Vec<u8>,
    background_map: *mut Vec<Color>,
}


#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn key_pressed(
    wasm_ptr: *mut Wasm,
    external_code: u32,
    ctrl: bool, alt: bool, shift: bool
)
{
    let wasm: Box<Wasm> = unsafe { Box::from_raw(wasm_ptr) };
    let mut state: Box<State> = unsafe { Box::from_raw(wasm.state) };

    let code = key_code_from_backend(external_code);
    if let Some(code) = code {
        state.keys.push(Key { code, alt, ctrl, shift});
    }

    mem::forget(state);
    mem::forget(wasm);
}


#[no_mangle]
pub extern "C" fn initialise() -> *mut Wasm {
    println!("Initialising {} for WebAssembly", ::GAME_TITLE);
    let state = Box::new(State::new_game(
        ::WORLD_SIZE,
        ::DISPLAYED_MAP_SIZE,
        ::PANEL_WIDTH,
        ::DISPLAY_SIZE,
        false,  // exit-after
        None,  // replay file
        false,  // invincible
    ));
    let buffer = Box::new(Vec::with_capacity(BUFFER_CAPACITY));
    let drawcalls = Box::new(Vec::with_capacity(engine::DRAWCALL_CAPACITY));
    let js_drawcalls = Box::new(Vec::with_capacity(JS_DRAWCALL_CAPACITY));
    let background_map = Box::new(vec![Color{r: 0, g: 0, b: 0}; (::DISPLAY_SIZE.x * ::DISPLAY_SIZE.y) as usize]);
    let wasm = {
        Box::new(Wasm {
            state: Box::into_raw(state),
            buffer: Box::into_raw(buffer),
            drawcalls: Box::into_raw(drawcalls),
            js_drawcalls: Box::into_raw(js_drawcalls),
            background_map: Box::into_raw(background_map),
        })
    };

    Box::into_raw(wasm)
}


fn serialise_drawcall(drawcall: &Draw, buffer: &mut Vec<u8>, js_drawcalls: &mut Vec<u8>) {
    buffer.clear();
    drawcall.serialize(&mut Serializer::new(&mut *buffer)).unwrap();
    js_drawcalls.extend(buffer.iter());
}


#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn update(wasm_ptr: *mut Wasm, dt_ms: u32) {
    let wasm: Box<Wasm> = unsafe { Box::from_raw(wasm_ptr) };
    let mut state: Box<State> = unsafe { Box::from_raw(wasm.state) };
    let mut buffer: Box<Vec<u8>> = unsafe { Box::from_raw(wasm.buffer) };
    let mut drawcalls: Box<Vec<Draw>> = unsafe { Box::from_raw(wasm.drawcalls) };
    let mut js_drawcalls: Box<Vec<u8>> = unsafe { Box::from_raw(wasm.js_drawcalls) };
    let mut background_map: Box<Vec<Color>> = unsafe { Box::from_raw(wasm.background_map) };

    drawcalls.clear();
    js_drawcalls.clear();

    let dt = Duration::from_millis(dt_ms as u64);
    let display_size = state.display_size;
    let fps = 60;
    let keys: Vec<Key> = vec![];
    let mouse: Mouse = Default::default();
    let mut settings = Settings{ fullscreen: false };

    let result = game::update(
        &mut state,
        dt,
        display_size,
        fps,
        &keys,
        mouse,
        &mut settings,
        &mut Metrics,
        &mut drawcalls,
    );

    match result {
        RunningState::Running => {}
        RunningState::NewGame(new_state) => {
            *state = new_state;
        }
        RunningState::Stopped => {},
    }

    engine::populate_background_map(&mut background_map, display_size, &drawcalls);

    // Send the background drawcalls first
    for (index, background_color) in background_map.iter().enumerate() {
        let pos = Point::new((index as i32) % display_size.x, (index as i32) / display_size.x);
        let drawcall = Draw::Background(pos, *background_color);
        serialise_drawcall(&drawcall, &mut buffer, &mut js_drawcalls);
    }

    let mut screen_fade = None;

    for drawcall in drawcalls.iter() {
        match drawcall {
            &Draw::Background(..) => {}
            &Draw::Fade(color, fade) => {
                screen_fade = Some(Draw::Fade(color, fade));
            }

            &Draw::Char(pos, _glyph, _color) => {
                if pos.x >= 0 && pos.y >= 0 && pos.x < display_size.x && pos.y < display_size.y {
                    // Clear the background
                    let bg_dc = Draw::Background(pos, background_map[(pos.y * display_size.x + pos.x) as usize]);
                    serialise_drawcall(&bg_dc, &mut buffer, &mut js_drawcalls);

                    // Send the glyph
                    serialise_drawcall(&drawcall, &mut buffer, &mut js_drawcalls);
                }
            }

            _ => {
                serialise_drawcall(&drawcall, &mut buffer, &mut js_drawcalls);
            }
        }
    }


    if state.cheating {
        // NOTE: render buffer size:
        let drawcall = Draw::Text(display_size - (5, 5),
                                  format!("Buffer cap: {}", buffer.capacity()).into(),
                                  ::color::gui_text,
                                  engine::TextOptions::align_right());
        serialise_drawcall(&drawcall, &mut buffer, &mut js_drawcalls);

        // NOTE: render js drawcall size
        let drawcall = Draw::Text(display_size - (5, 4),
                                  format!("js_drawcall len: {}", js_drawcalls.len()).into(),
                                  ::color::gui_text,
                                  engine::TextOptions::align_right());
        serialise_drawcall(&drawcall, &mut buffer, &mut js_drawcalls);

        // NOTE: render js drawcall size
        let drawcall = Draw::Text(display_size - (5, 3),
                                  format!("drawcall len: {}", drawcalls.len()).into(),
                                  ::color::gui_text,
                                  engine::TextOptions::align_right());
        serialise_drawcall(&drawcall, &mut buffer, &mut js_drawcalls);

        // TODO: print out warning when we exceed the capacity to the
        // JS console.
    }



    // Send the Fade drawcall last
    if let Some(drawcall) = screen_fade {
        serialise_drawcall(&drawcall, &mut buffer, &mut js_drawcalls);
    }


    // TODO


    unsafe {
        draw(js_drawcalls.as_ptr(), js_drawcalls.len());
    }

    mem::forget(background_map);
    mem::forget(js_drawcalls);
    mem::forget(drawcalls);
    mem::forget(buffer);
    mem::forget(state);
    mem::forget(wasm);
}
