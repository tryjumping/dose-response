pub enum MainLoopState {
    Running,
    Exit,
}

pub fn main_loop<T>(initial_state: &fn() -> ~T,
                    update: &fn(&mut T) -> MainLoopState) {
    let mut game_state = initial_state();
    for 3.times {
        match update(game_state) {
            Running => (),
            Exit => break,
        }
    }
}
