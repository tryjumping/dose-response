macro_rules! ensure_components(
    ($ecm:expr, $entity:expr, $($component:ident),+) => (
        if !$ecm.has_entity($entity) || $(!$ecm.has::<$component>($entity))||+ {return}
    )
)

// pub mod addiction;
// pub mod addiction_graphics;
// pub mod ai;
// pub mod bump;
pub mod color_fade;
// pub mod combat;
// pub mod dose;
// pub mod eating;
// pub mod effect_duration;
// pub mod exploration;
// pub mod fade_out;
// pub mod gui;
pub mod input;
// pub mod interaction;
// pub mod leave_area;
// pub mod movement;
// pub mod panic;
// pub mod player_dead;
// pub mod stun;
pub mod tile;
// pub mod turn;
// pub mod turn_tick_counter;
// pub mod will;
