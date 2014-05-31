use components::{AI, AcceptsUserInput, Corpse, Count, Destination, FadeColor, FadeOut,
                 InventoryItem, Solid, Tile, Turn};
use ecm::{ComponentManager, ECM, Entity};
use point;


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
        let tile = ecm.get::<Tile>(e);
        ecm.set(e, Tile{glyph: corpse.glyph,
                             color: corpse.color,
                             .. tile});
        ecm.set(e, FadeColor{
                from: tile.color,
                to: corpse.color,
                duration_s: 1f32,
                repetitions: Count(1),
            });
    } else if ecm.has::<FadeOut>(e) {
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
