use components::{ColorAnimation, FadingOut, Tile};
use components::{Count};
use ecm::{ComponentManager, ECM, Entity};


define_system! {
    name: FadeOutSystem;
    components(FadingOut, Tile);
    resources(ecm: ECM);
    fn process_entity(&mut self, dt_ms: uint, entity: Entity) {
        unimplemented!();
        // let mut ecm = self.ecm();
        // let fade_out = ecm.get::<FadeOut>(entity);
        // let tile = ecm.get::<Tile>(entity);
        // if !ecm.has::<FadingOut>(entity) {
        //     // replace any existing animation with our fade out
        //     ecm.remove::<ColorAnimation>(entity);
        //     ecm.set(entity, FadeColor{
        //         from: tile.color,
        //         to: fade_out.to,
        //         duration_s: fade_out.duration_s,
        //         repetitions: Count(1),
        //     });
        //     ecm.set(entity, FadingOut);
        // } else if !ecm.has::<ColorAnimation>(entity) {
        //     // the animation has ended, finish the fade out
        //     ecm.remove::<Tile>(entity);
        //     ecm.remove::<FadingOut>(entity);
        // }
    }
}
