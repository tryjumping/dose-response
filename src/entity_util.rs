use std::time::Duration;
use components::{AI, AcceptsUserInput, Corpse, Count, Destination,
                 InventoryItem, Solid, Tile, Turn};
use components::{ColorAnimation, ColorAnimationState, FadingOut, Repetitions, Forward};
use engine::Color;
use point;
use point::Point;
use emhyr::{Components, Entity};

pub use super::PositionCache;


pub fn set_color_animation_loop(cs: &mut Components, e: Entity,
                                from: Color, to: Color,
                                repetitions: Repetitions, duration: Duration) {
    cs.set(ColorAnimation{
        from: from,
        to: to,
        repetitions: repetitions,
        transition_duration: duration,
        current: ColorAnimationState{
            color: from,
            fade_direction: Forward,
            elapsed_time: Duration::milliseconds(0),
        },
    }, e);
}

pub fn fade_out(cs: &mut Components, e: Entity, color_to_fade_out: Color, duration: Duration) {
    assert!(cs.has::<Tile>(e), "Can't fade out an entity without a tile.");
    let tile = cs.get::<Tile>(e);
    cs.set(FadingOut, e);
    cs.set(ColorAnimation{
        from: tile.color,
        to: color_to_fade_out,
        repetitions: Count(1),
        transition_duration: duration,
        current: ColorAnimationState{
            color: tile.color,
            fade_direction: Forward,
            elapsed_time: Duration::milliseconds(0),
        }
    }, e);
}

pub fn kill(cs: &mut Components, e: Entity) {
    // TODO: we assume that an entity without a turn is already dead. Add a
    // `Dead` component (or something similar) instead.
    // TODO: also, this is a bug: killing should be idempotent
    if !cs.has::<Turn>(e) {return}
    cs.unset::<AI>(e);
    cs.unset::<AcceptsUserInput>(e);
    cs.unset::<Turn>(e);
    cs.unset::<Destination>(e);
    let solid_corpse = cs.has::<Corpse>(e) && cs.get::<Corpse>(e).solid;
    if !solid_corpse {
        cs.unset::<Solid>(e);
    }
    // Replace the entity's Tile with the tile of a corpse.
    if cs.has::<Corpse>(e) && cs.has::<Tile>(e) {
        let corpse = cs.get::<Corpse>(e);
        fade_out(cs, e, corpse.color, Duration::seconds(1));
        let tile = cs.get::<Tile>(e);
        cs.set(Tile{glyph: corpse.glyph,
                    color: corpse.color,
                    .. tile}, e);
    } else if cs.has::<FadingOut>(e) {
        // TODO: we probably shouldn't remove the fading-out entities here.
        // Makes no sense. Just remove their tiles after the fadeout.
    } else {
        cs.unset::<Tile>(e);
    }
}

pub fn explosion<T: point::Point>(cs: &mut Components, center: T, radius: int) {
    for (x, y) in point::points_within_radius(center, radius) {
        fail!("entities_on_pos");
        // for e in ecm.entities_on_pos((x, y)) {
        //     if ecm.has_entity(e) && ecm.has::<AI>(e) {
        //         kill(ecm, e);
        //     }
        // }
    }
}

pub fn get_first_owned_food(ecm: &Components, owner: Entity) -> Option<Entity> {
    fail!("TODO: GET FIRST OWNED FOOD NOT IMPLEMENTED");
    // TODO: sloooooooow. Add some caching like with Position?
    // for e in ecm.iter() {
    //     if ecm.has::<InventoryItem>(e) {
    //         let item = ecm.get::<InventoryItem>(e);
    //         if item.owner == owner {
    //             return Some(e);
    //         }
    //     }
    // }
    // None
}


pub fn is_solid<P: Point>(pos: P, cache: &PositionCache, cs: &Components) -> bool {
    cache.entities_on_pos(pos.coordinates()).any(|e| {
        cs.has::<Solid>(e)
    })
}

pub fn is_walkable<P: Point>(pos: P, cache: &PositionCache, cs: &Components,
                             map_size: (int, int)) -> bool {
    let (width, height) = map_size;
    let (x, y) = pos.coordinates();
    if x < 0 || y < 0 || x >= width || y >= height {
        return false;
    };
    !is_solid(pos, cache, cs)
}

pub fn is_wall<P: Point>(pos: P, cache: &PositionCache, cs: &Components) -> bool {
    fail!("entities on pos not implemented");
    // ecm.entities_on_pos(pos.coordinates()).any(|e| {
    //     ecm.has::<Background>(e) && ecm.has::<Solid>(e)
    // })
}
