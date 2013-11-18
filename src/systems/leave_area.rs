use components::*;
use world_gen;
use world;
use super::super::Resources;

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    ensure_components!(ecm, e, Destination);
    if e != res.player_id {return}
    let dest = ecm.get_destination(e);
    let (width, height) = res.world_size;
    let left_map_boundaries = (dest.x < 0 || dest.y < 0 ||
                               dest.x >= width ||
                               dest.y >= height);
    if left_map_boundaries {
        let player_entity = ecm.take_out(res.player_id);
        ecm.remove_all_entities();
        let player_id = ecm.add_entity(player_entity);
        res.player_id = player_id;
        // The player starts in the middle of the map with no pending
        // actions:
        ecm.set_position(player_id, Position{
                x: (width / 2) as int,
                y: (height / 2) as int,
            });
        ecm.remove_bump(player_id);
        ecm.remove_attack_target(player_id);
        ecm.remove_destination(player_id);
        let player_pos = ecm.get_position(player_id);
        world::populate_world(ecm,
                              res.world_size,
                              player_pos,
                              &mut res.rng,
                              world_gen::forrest);
        // TODO: We don't want the curret tick to continue after we've messed with
        // the game state. Signal the main loop to abort it early.
    }
}
