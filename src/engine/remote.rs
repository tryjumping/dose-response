use crate::{
    color::Color,
    engine::{Display, Mouse, Settings, TextMetrics, UpdateFn},
    game::RunningState,
    keys::Key,
    point::Point,
    state::State,
};

use std::{error::Error, thread, time::Duration};

use serde_json;
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
            let key_data = self
                .socket
                .recv_bytes(0)
                .map(|bytes| String::from_utf8(bytes))??;
            let key = serde_json::from_str(&key_data)?;
            Ok(Some(key))
        }
    }

    fn send_display(&self, display: &RemoteDisplay) -> Result<(), Box<Error>> {
        let message = serde_json::to_string(display)?;
        self.socket.send(message.as_bytes(), 0)?;

        Ok(())
    }
}

pub struct Metrics {
    tile_width_px: i32,
    text_width_px: i32,
}

impl TextMetrics for Metrics {
    fn tile_width_px(&self) -> i32 {
        self.tile_width_px
    }

    fn text_width_px(&self) -> i32 {
        self.text_width_px
    }
}

#[derive(Serialize, Deserialize)]
struct RemoteDisplay {
    pub width: i32,
    pub height: i32,
    pub cells: Vec<char>,
}

impl RemoteDisplay {
    fn new(width: i32, height: i32) -> Self {
        RemoteDisplay {
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

pub fn main_loop(
    display_size: Point,
    _default_background: Color,
    _window_title: &str,
    mut state: Box<State>,
    update: UpdateFn,
) {
    let ipc = match ZeroMQ::new("ipc:///tmp/dose-response.ipc") {
        Ok(ipc) => ipc,
        Err(err) => panic!("Could not create a ZeroMQ socket: {:?}", err),
    };

    let tilesize = super::TILESIZE;
    let mouse = Mouse::new();
    let mut settings = Settings { fullscreen: false };
    let mut keys = vec![];
    let mut display = Display::new(
        display_size,
        Point::from_i32(display_size.y / 2),
        tilesize as i32,
    );
    let mut remote_display = RemoteDisplay::new(display_size.x, display_size.y);

    loop {
        keys.clear();
        remote_display.clear();

        match ipc.try_read_key() {
            Ok(Some(key)) => {
                keys.push(key);
            }
            Ok(None) => {}
            Err(err) => panic!("Error reading a key {:?}", err),
        };

        match update(
            &mut state,
            Duration::from_millis(16),
            display_size,
            60,
            &keys,
            mouse,
            &mut settings,
            &Metrics {
                tile_width_px: tilesize as i32,
            },
            &mut display,
        ) {
            RunningState::Running => {
                for (pos, cell) in display.cells() {
                    remote_display.set(pos, cell.glyph);
                }
                // NOTE: if the client is sleeping, this will fail but
                // we don't mind. We ran the update, that's all we
                // wanted to do.
                let _ = ipc.send_display(&remote_display);
            }
            RunningState::NewGame(_new_state) => unimplemented!(),
            RunningState::Stopped => {
                // TODO: send a QUIT message here
                break;
            }
        };

        thread::sleep(Duration::from_millis(16));
    }
}
