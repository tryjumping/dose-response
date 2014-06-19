use components::{ColorAnimation, FadingOut, Tile};
use ecm::{ComponentManager, ECM, Entity};


define_system! {
    name: FadeOutSystem;
    components(FadingOut, Tile);
    resources(ecm: ECM);
    fn process_entity(&mut self, _dt_ms: uint, entity: Entity) {
        let ecm = &mut *self.ecm();
        // the animation has ended, finish the fade out
        if !ecm.has::<ColorAnimation>(entity) {
            ecm.remove::<Tile>(entity);
            ecm.remove::<FadingOut>(entity);
        }
    }
}
