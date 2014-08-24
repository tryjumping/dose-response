use std::time::Duration;
use components::{ColorAnimation, FadingOut, Tile};
use emhyr::{Components, Entity};


define_system! {
    name: FadeOutSystem;
    components(FadingOut, Tile);
    resources(player: Entity);
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, entity: Entity) {
        // the animation has ended, finish the fade out
        if !cs.has::<ColorAnimation>(entity) {
            cs.unset::<Tile>(entity);
            cs.unset::<FadingOut>(entity);
        }
    }
}
