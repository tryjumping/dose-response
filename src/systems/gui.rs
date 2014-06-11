use ecm::{ComponentManager, ECM, Entity};
use engine::{Display, Color};
use components::{Attributes, Edible, InventoryItem, Position, Stunned, Panicking};
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
    ecm.iter().filter(|&e| ecm.has::<InventoryItem>(e) && ecm.has::<Edible>(e) && ecm.get::<InventoryItem>(e).owner == player).count()
}

define_system! {
    name: GUISystem;
    resources(ecm: ECM, display: Display, player: Entity, current_turn: int);
    fn process_all_entities(&mut self, dt_ms: uint, mut entities: &mut Iterator<Entity>) {
        let mut ecm = &mut *self.ecm();
        let mut display = &mut *self.display();
        let (_width, height) = display.size();
        let player = *self.player();
        let current_turn = *self.current_turn();
        if !ecm.has_entity(player) || !ecm.has::<Attributes>(player) {return}

        let attrs: Attributes = ecm.get(player);
        let intoxication_description = intoxication_to_str(attrs.state_of_mind);
        let mut status_bar = format!("{}  Will: {}  Food: {}",
                                 intoxication_description,
                                 attrs.will,
                                 food_count(ecm, player));

        let mut effects = String::new();
        if !ecm.has::<Position>(player) {
            effects.push_str("dead ");
        };
        if ecm.has::<Stunned>(player) {
            let remaining = ecm.get::<Stunned>(player).remaining(current_turn);
            effects.push_str(format!("stunned({})", remaining).as_slice());
        };
        if ecm.has::<Panicking>(player) {
            let remaining = ecm.get::<Panicking>(player).remaining(current_turn);
            effects.push_str(format!("panic({}) ", remaining).as_slice());
        };
        if effects.len() > 0 {
            status_bar.push_str("  Effects: ");
            status_bar.push_str(effects.as_slice());
        }
        display.write_text(status_bar.as_slice(),
                           0, height - 1,
                           Color::new(255, 255, 255), Color::new(0, 0, 0));
    }
}
