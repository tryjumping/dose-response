mod ecm;
mod tcod;

fn generate_world(w: uint, h: uint) -> ~[(uint, uint, char)] {
    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut result: ~[(uint, uint, char)] = ~[];
    for std::uint::range(0, w) |x| {
        for std::uint::range(0, h) |y| {
            result.push((x, y, chars[(x * y) % chars.char_len()] as char));
        }
    }
    return result;
}

fn draw(layers: &[tcod::TCOD_console_t], world: &~[(uint, uint, char)], width: uint, height: uint) {
    let con = layers[layers.len() - 1];
    for world.iter().advance |&(x, y, glyph)| {
        tcod::console_set_char_background(con, x, y,
                                          tcod::TCOD_color_t{r: 30, g: 30, b: 30},
                                          tcod::TCOD_BKGND_SET);
        tcod::console_put_char(con, x, y, glyph, tcod::TCOD_BKGND_DEFAULT);
    }
    tcod::console_print_ex(con, width - 1, height-1,
                           tcod::TCOD_BKGND_NONE, tcod::TCOD_RIGHT,
                           fmt!("FPS: %?", tcod::sys_get_fps()));
}

enum Component {
    PositionC{x: int, y: int},
    HealthC(int),
}

enum ComponentType {
    Position,
    Health,
}

impl ecm::ComponentType for ComponentType {
    fn to_index(&self) -> uint {
        *self as uint
    }
}

struct P {x: int, y: int}
struct H(int);

struct GameObject {
    position: Option<P>,
    health: Option<H>,
}


struct ECM {
    game_objects: ~[GameObject],
}

impl ECM {
    fn add(&mut self, o: GameObject) {
        self.game_objects.push(o);
    }

    fn get(&self, index: uint) -> GameObject {
        self.game_objects[index]
    }

    fn get_ref<'r>(&'r self, index: uint) -> &'r GameObject {
        &self.game_objects[index]
    }
}


fn main() {
    let mut ecm: ecm::EntityManager<Component> = ecm::EntityManager::new();
    let e = ecm.new_entity();
    let f = ecm.new_entity();
    ecm.set(e, Position, PositionC{x: 10, y: 20});
    ecm.set(e, Health, HealthC(100));
    println(fmt!("e: %?, f: %?", e, f));
    let p = ecm.get(e, Position);
    println(fmt!("e's position: %?", p));

    let mut entities: ~[GameObject] = ~[];
    let player = GameObject{
        position: Some(P{x: 10, y: 20}),
        health: Some(H(100))};
    entities.push(player);
    entities.push(GameObject{position: Some(P{x: 1, y: 1}), health: None});

    let &tile = &entities[1];
    println(fmt!("player: %?", player));
    println(fmt!("tile: %?", tile));

    let mut em = ECM{game_objects: ~[]};
    em.add(player);
    em.add(tile);
    println(fmt!("ecm player: %?", em.get(0)));
    let tile: &GameObject = em.get_ref(0);
    println(fmt!("ecm tile ref: %?", tile));
}
