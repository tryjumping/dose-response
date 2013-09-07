use components::*;

mod components;


pub fn debug_system(entity: &GameObject) {
    println(fmt!("DEBUG processing entity: %?", entity));
}

pub fn tile_system(entity: &GameObject) {
    if entity.position.is_none() { return }
    let pos = entity.position.get();
    println(fmt!("Tile system renders tile on pos: %?", pos));
}

pub fn health_system(entity: &mut GameObject) {
    if entity.health.is_none() { return }
    let health = *entity.health.get();
    entity.health = Some(Health(health - 1));
}
