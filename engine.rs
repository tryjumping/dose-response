pub enum MainLoopState {
    Running,
    Exit,
}

pub fn main_loop<T>(width: uint, height: uint,
                    initial_state: &fn(uint, uint) -> ~T,
                    update: &fn(&mut T) -> MainLoopState) {
    let mut game_state = initial_state(width, height);
    loop {
        match update(game_state) {
            Running => loop,
            Exit => break,
        }
    }
}
