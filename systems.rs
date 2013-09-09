use components::*;
use engine::{Display, Color};
use extra::deque::Deque;


pub enum Command {
    N, E, S, W, NE, NW, SE, SW,
}

pub fn input_system(entity: &mut GameObject, commands: &mut Deque<Command>) {
    if entity.accepts_user_input.is_none() { return }
    if entity.position.is_none() { return }
    if commands.is_empty() { return }

    let pos = entity.position.get();
    let dest = match commands.pop_front() {
        N => Destination{x: pos.x, y: pos.y-1},
        S => Destination{x: pos.x, y: pos.y+1},
        W => Destination{x: pos.x-1, y: pos.y},
        E => Destination{x: pos.x+1, y: pos.y},

        NW => Destination{x: pos.x-1, y: pos.y-1},
        NE => Destination{x: pos.x+1, y: pos.y-1},
        SW => Destination{x: pos.x-1, y: pos.y+1},
        SE => Destination{x: pos.x+1, y: pos.y+1},
    };
    entity.destination = Some(dest);
}

pub fn movement_system(entity: &mut GameObject) {
    if entity.position.is_none() { return }
    if entity.destination.is_none() { return }

    let Destination{x, y} = entity.destination.get();
    entity.position = Some(Position{x: x, y: y});
    entity.destination = None;
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
