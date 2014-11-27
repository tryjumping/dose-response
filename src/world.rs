use item::Item;
use level::Level;
use monster::Monster;
use generators::GeneratedWorld;


pub fn populate_world(level: &mut Level,
                      monsters: &mut Vec<Monster>,
                      generated_world: GeneratedWorld) {
    // // TODO: move this random generation stuff to generators?
    // let near_player = |x, y| point::tile_distance(player_pos, (x, y)) < 6;
    // let pos_offset = &[-4, -3, -2, -1, 1, 2, 3, 4];
    // let initial_dose_pos = (player_pos.0 + *rng.choose(pos_offset).unwrap(),
    //                         player_pos.1 + *rng.choose(pos_offset).unwrap());
    // let mut initial_foods_pos = Vec::<(int, int)>::new();
    // for _ in range(0, rng.gen_range::<uint>(1, 4)) {
    //     let pos = (player_pos.0 + *rng.choose(pos_offset).unwrap(),
    //                player_pos.1 + *rng.choose(pos_offset).unwrap());
    //     initial_foods_pos.push(pos);
    // };
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
