use components::*;

mod components;


pub fn debug_system(entity: &GameObject) {
}

pub fn tile_system(entity: &GameObject) {
    if entity.position.is_none() { return }
    let pos = entity.position.get();
}

pub fn health_system(entity: &mut GameObject) {
    if entity.health.is_none() { return }
    let health = *entity.health.get();
    entity.health = Some(Health(health - 1));
}
