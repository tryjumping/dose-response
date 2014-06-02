use components::{AI, AcceptsUserInput, Corpse, Count, Destination,
                 InventoryItem, Solid, Tile, Turn};
use components::{ColorAnimation, ColorAnimationState, FadingOut, Sec, Repetitions, Forward};
use ecm::{ComponentManager, ECM, Entity};
use engine::Color;
use point;


pub fn set_color_animation_loop(ecm: &mut ECM, e: Entity,
                                from: Color, to: Color,
                                repetitions: Repetitions, duration: Sec) {
    ecm.set(e, ColorAnimation{
        from: from,
        to: to,
        repetitions: repetitions,
        transition_duration: duration,
        current: ColorAnimationState{
            color: from,
            fade_direction: Forward,
            elapsed_time: Sec(0.0),
        },
    });
}

pub fn fade_out(ecm: &mut ECM, e: Entity, color_to_fade_out: Color, duration: Sec) {
    assert!(ecm.has::<Tile>(e), "Can't fade out an entity without a tile.");
    let tile = ecm.get::<Tile>(e);
    ecm.set(e, FadingOut);
    ecm.set(e, ColorAnimation{
        from: tile.color,
        to: color_to_fade_out,
        repetitions: Count(1),
        transition_duration: duration,
        current: ColorAnimationState{
            color: tile.color,
            fade_direction: Forward,
            elapsed_time: Sec(0.0),
        }
    });
}

pub fn kill(ecm: &mut ECM, e: Entity) {
    if !ecm.has_entity(e) {return}
    // TODO: we assume that an entity without a turn is already dead. Add a
    // `Dead` component (or something similar) instead.
    // TODO: also, this is a bug: killing should be idempotent
    if !ecm.has::<Turn>(e) {return}
    ecm.remove::<AI>(e);
    ecm.remove::<AcceptsUserInput>(e);
    ecm.remove::<Turn>(e);
    ecm.remove::<Destination>(e);
    let solid_corpse = ecm.has::<Corpse>(e) && ecm.get::<Corpse>(e).solid;
    if !solid_corpse {
        ecm.remove::<Solid>(e);
    }
    // Replace the entity's Tile with the tile of a corpse.
    if ecm.has::<Corpse>(e) && ecm.has::<Tile>(e) {
        let corpse = ecm.get::<Corpse>(e);
        fade_out(ecm, e, corpse.color, Sec(1.0));
        let tile = ecm.get::<Tile>(e);
        ecm.set(e, Tile{glyph: corpse.glyph,
                        color: corpse.color,
                        .. tile});
    } else if ecm.has::<FadingOut>(e) {
        // TODO: we probably shouldn't remove the fading-out entities here.
        // Makes no sense. Just remove their tiles after the fadeout.
    } else {
        ecm.remove::<Tile>(e);
    }
}

pub fn explosion<T: point::Point>(ecm: &mut ECM, center: T, radius: int) {
    for (x, y) in point::points_within_radius(center, radius) {
        for e in ecm.entities_on_pos((x, y)) {
            if ecm.has_entity(e) && ecm.has::<AI>(e) {
                kill(ecm, e);
            }
        }
    }
}

pub fn get_first_owned_food(ecm: &ECM, owner: Entity) -> Option<Entity> {
    // TODO: sloooooooow. Add some caching like with Position?
    for e in ecm.iter() {
        if ecm.has::<InventoryItem>(e) {
            let item = ecm.get::<InventoryItem>(e);
            if item.owner == owner {
                return Some(e);
            }
        }
    }
    None
}
