use components::*;
use engine::{Display, Color};
use extra::deque::Deque;


pub enum Command {
    N, E, S, W, NE, NW, SE, SW,
}

pub fn input_system(entity: &mut GameObject, commands: &mut Deque<Command>) {
    // TODO: replace this with a check for InputComponent
    if entity.health.is_none() { return }
    if entity.position.is_none() { return }
    if commands.is_empty() { return }

    let pos = entity.position.get();
    let newpos = match commands.pop_front() {
        N => Position{y: pos.y-1, .. pos},
        S => Position{y: pos.y+1, .. pos},
        W => Position{x: pos.x-1, .. pos},
        E => Position{x: pos.x+1, .. pos},

        NW => Position{x: pos.x-1, y: pos.y-1},
        NE => Position{x: pos.x+1, y: pos.y-1},
        SW => Position{x: pos.x-1, y: pos.y+1},
        SE => Position{x: pos.x+1, y: pos.y+1},
    };
    entity.position = Some(newpos);
}

pub fn tile_system(entity: &GameObject, display: &mut Display) {
    if entity.position.is_none() { return }
    if entity.tile.is_none() { return }

    let Position{x, y} = entity.position.get();
    let Tile{level, glyph, color} = entity.tile.get();
    display.draw_char(level, x as uint, y as uint, glyph, color, Color(20, 20, 20));
}

pub fn health_system(entity: &mut GameObject) {
    if entity.health.is_none() { return }
    let health = *entity.health.get();
    entity.health = Some(Health(health - 1));
}
