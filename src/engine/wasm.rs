use crate::{
    engine::{self, Display, Drawcall, Mouse, TextMetrics},
    game::{self, RunningState},
    keys::{Key, KeyCode},
    point::Point,
    settings::Settings,
    state::State,
};

use std::{mem, time::Duration};

const VERTEX_CAPACITY: usize =
    mem::size_of::<f32>() * engine::VERTEX_COMPONENT_COUNT * engine::VERTEX_CAPACITY;

// NOTE: we're forcing a specific tilesize for the web. This is
// because the player can't actually resize the canvas from the web UI
// right now.
//
// TODO: the default tilesize in WASM should come from the settings
// rather than a const here.
const DEFAULT_TILESIZE: i32 = 24;

extern "C" {
    fn draw(nums: *const u8, len: usize, texture_width_px: i32, texture_height_px: i32);
    pub fn random() -> f32;
}

fn key_code_from_backend(js_keycode: u32) -> Option<KeyCode> {
    use crate::keys::KeyCode::*;
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

struct Metrics {
    tile_width_px: i32,
}

impl TextMetrics for Metrics {
    fn tile_width_px(&self) -> i32 {
        self.tile_width_px
    }
}

/// Struct holding most of our games' memory. Everything we could
/// preallocate is here.
pub struct Wasm {
    state: *mut State,
    drawcalls: *mut Vec<Drawcall>,
    vertices: *mut Vec<u8>,
    display: *mut Display,
}

#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn key_pressed(
    wasm_ptr: *mut Wasm,
    external_code: u32,
    ctrl: bool,
    alt: bool,
    shift: bool,
    logo: bool,
) {
    let wasm: Box<Wasm> = unsafe { Box::from_raw(wasm_ptr) };
    let mut state: Box<State> = unsafe { Box::from_raw(wasm.state) };

    let code = key_code_from_backend(external_code);
    if let Some(code) = code {
        state.keys.push(Key {
            code,
            alt,
            ctrl,
            shift,
            logo,
        });
    }

    mem::forget(state);
    mem::forget(wasm);
}

#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn initialise(display_size: i32, panel_width: i32) -> *mut Wasm {
    log::info!("Initialising {} for WebAssembly", crate::metadata::TITLE);
    let displayed_map_size = Point {
        x: display_size - panel_width,
        y: display_size,
    };
    let state = Box::new(State::new_game(
        crate::WORLD_SIZE,
        displayed_map_size,
        panel_width,
        false, // exit-after
        None,  // replay file
        false, // invincible
    ));
    let drawcalls = Box::new(Vec::with_capacity(crate::engine::DRAWCALL_CAPACITY));
    let vertices = Box::new(Vec::with_capacity(VERTEX_CAPACITY));
    let display = Box::new(crate::engine::Display::new(
        Point::from_i32(display_size),
        DEFAULT_TILESIZE as i32,
    ));

    let wasm = {
        Box::new(Wasm {
            state: Box::into_raw(state),
            drawcalls: Box::into_raw(drawcalls),
            vertices: Box::into_raw(vertices),
            display: Box::into_raw(display),
        })
    };

    Box::into_raw(wasm)
}

#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn update(
    wasm_ptr: *mut Wasm,
    dt_ms: u32,
    canvas_width: i32,
    canvas_height: i32,
    mouse_pixel_x: i32,
    mouse_pixel_y: i32,
    mouse_left: bool,
    mouse_right: bool,
) {
    let mut wasm: Box<Wasm> = unsafe { Box::from_raw(wasm_ptr) };
    let mut state: Box<State> = unsafe { Box::from_raw(wasm.state) };
    let mut drawcalls: Box<Vec<Drawcall>> = unsafe { Box::from_raw(wasm.drawcalls) };
    let mut vertices: Box<Vec<u8>> = unsafe { Box::from_raw(wasm.vertices) };
    let mut display: Box<Display> = unsafe { Box::from_raw(wasm.display) };

    let dt = Duration::from_millis(dt_ms as u64);
    let display_size = display.size_without_padding();
    let new_display_size = Point::new(
        canvas_width / DEFAULT_TILESIZE,
        canvas_height / DEFAULT_TILESIZE,
    );
    if new_display_size != display_size {
        let new_display = Box::new(crate::engine::Display::new(
            new_display_size,
            DEFAULT_TILESIZE as i32,
        ));
        // TODO(shadower): This causes a panic during first load
        // (`RuntimeError: unreachable executed`). The browser seems
        // to handle it fine and everything after that seems to work
        // fine, but we should investigate.
        wasm.display = Box::into_raw(new_display);

        // NOTE(shadower): uncommenting this causes every `update` call to panic.
        // Why? Right now we just carry on and use the new Display size a frame
        // late, but we should be able to replace it here and now.
        // display = unsafe { Box::from_raw(wasm.display) };
    }
    let display_size = display.size_without_padding();
    let mouse_tile_x = mouse_pixel_x / DEFAULT_TILESIZE;
    let mouse_tile_y = mouse_pixel_y / DEFAULT_TILESIZE;

    let fps = 60;
    let keys: Vec<Key> = vec![];
    let mouse = Mouse {
        tile_pos: Point::new(mouse_tile_x, mouse_tile_y),
        screen_pos: Point::new(mouse_pixel_x, mouse_pixel_y),
        left_clicked: mouse_left,
        right_clicked: mouse_right,
        left_is_down: false,
        right_is_down: false,
    };
    let mut settings = Settings {
        fullscreen: false,
        backend: "web".into(),
        tile_size: DEFAULT_TILESIZE,
    };
    let metrics = Metrics {
        tile_width_px: DEFAULT_TILESIZE as i32,
    };

    let result = game::update(
        &mut state,
        dt,
        fps,
        &keys,
        mouse,
        &mut settings,
        &metrics,
        &mut crate::settings::NoOpStore,
        &mut display,
    );

    match result {
        RunningState::Running => {}
        RunningState::NewGame(new_state) => {
            *state = *new_state;
        }
        RunningState::Stopped => {}
    }

    let native_display_size_px = [
        display_size.x as f32 * DEFAULT_TILESIZE as f32,
        display_size.y as f32 * DEFAULT_TILESIZE as f32,
    ];

    drawcalls.clear();
    display.push_drawcalls(&mut drawcalls);

    vertices.clear();
    crate::engine::build_vertices(&drawcalls, &mut *vertices, native_display_size_px);

    if state.cheating {
        // // NOTE: render buffer size:
        // let drawcall = Draw::Text(
        //     display_size - (5, 5),
        //     format!("Buffer cap: {}", buffer.capacity()).into(),
        //     ::color::gui_text,
        //     engine::TextOptions::align_right(),
        // );
        // serialise_drawcall(&drawcall, &mut buffer, &mut js_drawcalls);

        // // NOTE: render js drawcall size
        // let drawcall = Draw::Text(
        //     display_size - (5, 4),
        //     format!("js_drawcall len: {}", js_drawcalls.len()).into(),
        //     ::color::gui_text,
        //     engine::TextOptions::align_right(),
        // );
        // serialise_drawcall(&drawcall, &mut buffer, &mut js_drawcalls);

        // // NOTE: render js drawcall size
        // let drawcall = Draw::Text(
        //     display_size - (5, 3),
        //     format!("drawcall len: {}", drawcalls.len()).into(),
        //     ::color::gui_text,
        //     engine::TextOptions::align_right(),
        // );
        // serialise_drawcall(&drawcall, &mut buffer, &mut js_drawcalls);

        // TODO: print out warning when we exceed the capacity to the
        // JS console.
    }

    unsafe {
        draw(
            vertices.as_ptr(),
            vertices.len(),
            crate::engine::TEXTURE_WIDTH as i32,
            crate::engine::TEXTURE_HEIGHT as i32,
        );
    }

    mem::forget(display);
    mem::forget(drawcalls);
    mem::forget(vertices);
    mem::forget(state);
    mem::forget(wasm);
}
