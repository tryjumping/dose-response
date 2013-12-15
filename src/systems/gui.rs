use engine::{Display, Color};
use components::*;
use super::super::Resources;
use systems::addiction_graphics::intoxication_state::*;


fn intoxication_to_str(state: int) -> ~str {
    match IntoxicationState::from_int(state) {
        Exhausted => ~"Exhausted",
        DeliriumTremens => ~"Delirium tremens",
        Withdrawal => ~"Withdrawn",
        Sober => ~"Sober",
        High => ~"High",
        VeryHigh => ~"High as a kite",
        Overdosed => ~"Overdosed",
    }
}

pub fn system(ecm: &ComponentManager,
              res: &mut Resources,
              display: &mut Display) {
    let (_width, height) = display.size();
    let player = res.player_id;
    ensure_components!(ecm, player, Attributes);
    let attrs = ecm.get_attributes(player);
    let dead = match ecm.has_position(player) {
        true => ~"",
        false => ~"dead ",
    };
    let intoxication_description = intoxication_to_str(attrs.state_of_mind);
    let stunned = match ecm.has_stunned(player) {
        true => format!("stunned({}) ", ecm.get_stunned(player).remaining(res.turn)),
        false => ~"",
    };
    let panicking = match ecm.has_panicking(player) {
        true => format!("panic({}) ", ecm.get_panicking(player).remaining(res.turn)),
        false => ~"",
    };
    let effects = format!("{}{}{}", dead, stunned, panicking);
    let status_bar = format!("{},  Will: {}, Effects: {}",
                             intoxication_description,
                             attrs.will,
                             effects);
    display.write_text(status_bar,
                       0, height - 1,
                       Color::new(255, 255, 255), Color::new(0, 0, 0));
}
