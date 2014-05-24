use emhyr::{ComponentManager, ECM, Entity};
use engine::{Display, Color};
use components::{InventoryItem, Edible};
use systems::addiction_graphics::intoxication_state::*;


fn intoxication_to_str(state: int) -> &'static str {
    match IntoxicationState::from_int(state) {
        Exhausted => "Exhausted",
        DeliriumTremens => "Delirium tremens",
        Withdrawal => "Withdrawn",
        Sober => "Sober",
        High => "High",
        VeryHigh => "High as a kite",
        Overdosed => "Overdosed",
    }
}

fn food_count(ecm: &ECM, player: Entity) -> uint {
    ecm.iter().count(|e| ecm.has::<InventoryItem>(e) && ecm.has::<Edible>(e) && ecm.get::<InventoryItem>(e).owner == player)
}

pub fn system(ecm: &ECM,
              display: &mut Display) {
    fail!("TODO");
    // let (_width, height) = display.size();
    // let player = res.player;
    // ensure_components!(ecm, player, Attributes);
    // let attrs: Attributes = ecm.get(player);
    // let dead = match ecm.has::<Position>(player) {
    //     true => ~"",
    //     false => ~"dead ",
    // };
    // let intoxication_description = intoxication_to_str(attrs.state_of_mind);
    // let stunned = match ecm.has::<Stunned>(player) {
    //     true => format!("stunned({}) ", ecm.get::<Stunned>(player).remaining(res.turn)),
    //     false => ~"",
    // };
    // let panicking = match ecm.has::<Panicking>(player) {
    //     true => format!("panic({}) ", ecm.get::<Panicking>(player).remaining(res.turn)),
    //     false => ~"",
    // };
    // let effects = format!("{}{}{}", dead, stunned, panicking);
    // let status_bar = format!("{}  Will: {}  Food: {}  {}",
    //                          intoxication_description,
    //                          attrs.will,
    //                          food_count(ecm, player),
    //                          if effects.len() > 0 {"Effects: " + effects} else {~""});
    // display.write_text(status_bar,
    //                    0, height - 1,
    //                    Color::new(255, 255, 255), Color::new(0, 0, 0));
}
