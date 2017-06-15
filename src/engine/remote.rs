

use color::Color;
use engine::{Draw, Settings, UpdateFn};
use keys::Key;
use point::Point;
use serde_json;
use std::error::Error;
use std::thread;
use time::Duration;

use zmq;


struct ZeroMQ {
    socket: zmq::Socket,
}

impl ZeroMQ {
    fn new(connection: &str) -> Result<Self, Box<Error>> {
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::REP)?;
        socket.bind(connection)?;

        Ok(ZeroMQ { socket: socket })
    }

    fn try_read_key(&self) -> Result<Option<Key>, Box<Error>> {
        let poll_status = self.socket.poll(zmq::POLLIN, 0)?;
        if poll_status == 0 {
            Ok(None)
        } else {
            let key_data = self.socket.recv_bytes(0).map(
                |bytes| String::from_utf8(bytes),
            )??;
            let key = serde_json::from_str(&key_data)?;
            Ok(Some(key))
        }
    }

    fn send_display(&self, display: &Display) -> Result<(), Box<Error>> {
        let message = serde_json::to_string(display)?;
        self.socket.send(message.as_bytes(), 0)?;

        Ok(())
    }
}


#[derive(Serialize, Deserialize)]
struct Display {
    pub width: i32,
    pub height: i32,
    pub cells: Vec<char>,
}

impl Display {
    fn new(width: i32, height: i32) -> Self {
        Display {
            width: width,
            height: height,
            cells: vec![' '; (width * height) as usize],
        }
    }

    fn set(&mut self, pos: Point, chr: char) {
        if pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height {
            self.cells[(pos.y * self.width + pos.x) as usize] = chr;
        }
    }

    fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = ' ';
        }
    }
}


pub fn main_loop<T>(
    display_size: Point,
    _default_background: Color,
    _window_title: &str,
    mut state: T,
    update: UpdateFn<T>,
) {
    let ipc = match ZeroMQ::new("ipc:///tmp/dose-response.ipc") {
        Ok(ipc) => ipc,
        Err(err) => panic!("Could not create a ZeroMQ socket: {:?}", err),
    };

    let settings = Settings { fullscreen: false };
    let mut keys = vec![];
    let mut drawcalls = Vec::with_capacity(4000);
    let mut display = Display::new(display_size.x, display_size.y);

    loop {
        keys.clear();
        drawcalls.clear();
        display.clear();

        match ipc.try_read_key() {
            Ok(Some(key)) => {
                keys.push(key);
            }
            Ok(None) => {}
            Err(err) => panic!("Error reading a key {:?}", err),
        };

        match update(
            state,
            Duration::milliseconds(16),
            display_size,
            60,
            &keys,
            settings,
            &mut drawcalls,
        ) {
            Some((_new_settings, new_state)) => {
                state = new_state;
                for drawcall in &drawcalls {
                    match drawcall {
                        &Draw::Char(pos, chr, _foreground_color) => {
                            display.set(pos, chr);
                        }
                        &Draw::Background(_pos, _background_color) => {}
                        &Draw::Text(_start_pos, ref _text, _color) => {}
                        &Draw::Rectangle(_top_left, _dimensions, _color) => {}
                        &Draw::Fade(_fade, _color) => {}
                    }
                }
                // NOTE: if the client is sleeping, this will fail but
                // we don't mind. We ran the update, that's all we
                // wanted to do.
                let _ = ipc.send_display(&display);
            }
            None => {
                // TODO: send a QUIT message here
                break;
            }
        };

        thread::sleep(Duration::milliseconds(16).to_std().unwrap());
    }
}
