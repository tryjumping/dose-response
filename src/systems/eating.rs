use std::iter::range_inclusive;

use components::*;
use super::super::Resources;
use super::combat;

fn get_first_owned_food(ecm: &ComponentManager, owner: ID) -> Option<ID> {
    for e in ecm.iter() {
        if ecm.has_inventory_item(e) {
            let item = ecm.get_inventory_item(e);
            if item.owner == owner {
                return Some(e);
            }
        }
    }
    None
}

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              _res: &mut Resources) {
    // Only the player can eat for now:
    ensure_components!(ecm, e, AcceptsUserInput, Position, Turn);
    let free_aps = ecm.get_turn(e).ap;
    if free_aps <= 0 { return }
    match get_first_owned_food(ecm, e) {
        Some(food) => {
            println!("{:?} eats food {:?}", e, food);
            ecm.remove_inventory_item(food);
            // TODO: this is copypasted from the Interaction system. Deduplicate!
            if ecm.has_explosion_effect(food) {
                println!("Eating kills off nearby enemies");
                let pos = ecm.get_position(e);
                let radius = ecm.get_explosion_effect(food).radius;
                for x in range_inclusive(pos.x - radius, pos.x + radius) {
                    for y in range_inclusive(pos.y - radius, pos.y + radius) {
                        for monster in ecm.entities_on_pos(Position{x: x, y: y}) {
                            if ecm.has_entity(monster) && ecm.has_ai(monster) {
                                combat::kill_entity(monster, ecm);
                            }
                        }
                    }
                }
            } else {
                println!("The food doesn't have enemy-killing effect.");
            }
        }
        None => println!("{:?} doesn't have any food.", e),
    }
}
