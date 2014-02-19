use std::iter::range_inclusive;

use components::*;
use super::super::Resources;
use super::combat;

pub fn get_first_owned_food(ecm: &ComponentManager, owner: ID) -> Option<ID> {
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
    ensure_components!(ecm, e, AcceptsUserInput, Position, Turn, UsingItem);
    println!("processing food system");
    let food = ecm.get_using_item(e).item;
    if !ecm.has_edible(food) {
        println!("food {:?} isn't edible", food);
        return;
    }
    assert!(ecm.has_inventory_item(food));
    let turn = ecm.get_turn(e);
    if turn.ap <= 0 {
        return;
    }
    println!("{:?} eats food {:?}", e, food);
    ecm.remove_inventory_item(food);
    ecm.remove_using_item(e);
    ecm.set_turn(e, turn.spend_ap(1));
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
