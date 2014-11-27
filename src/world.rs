use item::Item;
use level::Level;
use monster::Monster;
use generators::GeneratedWorld;


pub fn populate_world(level: &mut Level,
                      monsters: &mut Vec<Monster>,
                      generated_world: GeneratedWorld) {
    let (map, generated_monsters, items) = generated_world;
    for &(pos, item) in map.iter() {
        level.set_tile(pos, item);
    }
    for &(pos, kind) in generated_monsters.iter() {
        assert!(level.walkable(pos));
        let monster = Monster::new(kind, pos);
        monsters.push(monster);
    }
    for &(pos, kind) in items.iter() {
        assert!(level.walkable(pos));
        let item = Item::new(kind);
        level.add_item(pos, item);
    }
}
