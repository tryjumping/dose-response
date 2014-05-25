use ecm::{ComponentManager, ECM, Entity};
use engine::Color;
use components::{Anxiety, AnxietyKillCounter, AI, AttackTarget, AttackType,
                 AcceptsUserInput, Count, Corpse, Destination, FadeColor,
                 FadeOut, Kill, Monster, Panic, Position, Solid, Stun, Tile, Turn};


pub fn kill_entity(e: Entity,
                   ecm: &mut ECM) {
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

define_system! {
    name: CombatSystem;
    components(AttackTarget, AttackType, Turn);
    resources(ecm: ECM, player: Entity);
    fn process_entity(&mut self, dt_ms: uint, entity: Entity) {
        let mut ecm = &mut *self.ecm();
        let free_aps = ecm.get::<Turn>(entity).ap;
        let AttackTarget(target) = ecm.get::<AttackTarget>(entity);
        ecm.remove::<AttackTarget>(entity);
        let attack_successful = ecm.has_entity(target) && free_aps > 0;
        if !attack_successful {return}
        // attacker spends an AP
        let turn: Turn = ecm.get(entity);
        ecm.set(entity, turn.spend_ap(1));
        match ecm.get::<AttackType>(entity) {
            Kill => {
                println!("Entity {:?} was killed by {:?}", target, entity);
                kill_entity(target, ecm);
                // TODO: This is a hack. The player should fade out, the other
                // monsters just disappear. Need to make this better without
                // special-casing the player.
                if target != *self.player() {
                    ecm.remove::<Position>(target);
                }
                let target_is_anxiety = (ecm.has::<Monster>(target) &&
                                         ecm.get::<Monster>(target).kind == Anxiety);
                if target_is_anxiety && ecm.has::<AnxietyKillCounter>(entity) {
                    let counter = ecm.get::<AnxietyKillCounter>(entity);
                    ecm.set(entity, AnxietyKillCounter{
                        count: counter.count + 1,
                        .. counter
                    });
                }
            }
            Stun{duration} => {
                println!("TODO: stunned");
                // println!("Entity {} was stunned by {}", target.deref(), e.deref());
                // // An attacker with stun disappears after delivering the blow
                // ecm.set_fade_out(e, FadeOut{to: Color{r: 0, g: 0, b: 0}, duration_s: 0.4});
                // if ecm.has_tile(e) {
                //     let tile = ecm.get_tile(e);
                //     if tile.level > 0 {
                //         ecm.set(e, Tile{level: tile.level - 1, .. tile});
                //     }
                // }
                // kill_entity(e, ecm);
                // let stunned = if ecm.has_stunned(target) {
                //     let prev = ecm.get_stunned(target);
                //     Stunned{duration: prev.duration + duration, .. prev}
                // } else {
                //     Stunned{turn: res.turn, duration: duration}
                // };
                // ecm.set_stunned(target, stunned);
            }
            Panic{duration} => {
                println!("TODO: panic");
                println!("Entity {:?} panics because of {:?}", target, entity);
                // // An attacker with stun disappears after delivering the blow
                // ecm.set_fade_out(e, FadeOut{to: Color{r: 0, g: 0, b: 0}, duration_s: 0.4});
                // if ecm.has_tile(e) {
                //     let tile = ecm.get_tile(e);
                //     if tile.level > 0 {
                //         ecm.set(e, Tile{level: tile.level - 1, .. tile});
                //     }
                // }
                // kill_entity(e, ecm);
                // let panicking = if ecm.has_panicking(target) {
                //     let prev = ecm.get_panicking(target);
                //     Panicking{duration: prev.duration + duration, .. prev}
                // } else {
                //     Panicking{turn: res.turn, duration: duration}
                // };
                // ecm.set_panicking(target, panicking);
            }
            ModifyAttributes => {
                println!("TODO: modify attributes");
                // if !ecm.has_attribute_modifier(e) {
                //     fail!("The attacker must have attribute_modifier");
                // }
                // let modifier = ecm.get_attribute_modifier(e);
                // if ecm.has_attributes(target) {
                //     let attrs = ecm.get_attributes(target);
                //     ecm.set(target, Attributes{
                //         state_of_mind: attrs.state_of_mind + modifier.state_of_mind,
                //         will: attrs.will + modifier.will})
                // }
            }
        }
    }
}
