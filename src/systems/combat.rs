use ecm::{ComponentManager, ECM, Entity};
use engine::Color;
use components::{Anxiety, AnxietyKillCounter, AI, AttackTarget, AttackType,
                 Attributes, AttributeModifier,
                 AcceptsUserInput, Count, Corpse, Destination, FadeColor,
                 FadeOut, Kill, Monster, Panic, Panicking, Position, Solid, Stun,
                 Stunned, Tile, Turn};


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
    resources(ecm: ECM, player: Entity, current_turn: int);
    fn process_entity(&mut self, dt_ms: uint, attacker: Entity) {
        let mut ecm = &mut *self.ecm();
        let free_aps = ecm.get::<Turn>(attacker).ap;
        let AttackTarget(target) = ecm.get::<AttackTarget>(attacker);
        ecm.remove::<AttackTarget>(attacker);
        let attack_successful = ecm.has_entity(target) && free_aps > 0;
        if !attack_successful {return}
        // attacker spends an AP
        let turn: Turn = ecm.get(attacker);
        ecm.set(attacker, turn.spend_ap(1));
        match ecm.get::<AttackType>(attacker) {
            Kill => {
                println!("Entity {:?} was killed by {:?}", target, attacker);
                kill_entity(target, ecm);
                // TODO: This is a hack. The player should fade out, the other
                // monsters just disappear. Need to make this better without
                // special-casing the player.
                if target != *self.player() {
                    ecm.remove::<Position>(target);
                }
                let target_is_anxiety = (ecm.has::<Monster>(target) &&
                                         ecm.get::<Monster>(target).kind == Anxiety);
                if target_is_anxiety && ecm.has::<AnxietyKillCounter>(attacker) {
                    let counter = ecm.get::<AnxietyKillCounter>(attacker);
                    ecm.set(attacker, AnxietyKillCounter{
                        count: counter.count + 1,
                        .. counter
                    });
                }
            }
            Stun{duration} => {
                println!("Entity {:?} was stunned by {:?}", target, attacker);
                // An attacker with stun disappears after delivering the blow
                ecm.set(attacker, FadeOut{to: Color{r: 0, g: 0, b: 0}, duration_s: 0.4});
                if ecm.has::<Tile>(attacker) {
                    // TODO: why are we decrementing the tile level here? Looks like
                    // a hack for a faulty display logic to me.
                    let tile = ecm.get::<Tile>(attacker);
                    if tile.level > 0 {
                        ecm.set(attacker, Tile{level: tile.level - 1, .. tile});
                    }
                }
                kill_entity(attacker, ecm);
                let stunned = if ecm.has::<Stunned>(target) {
                    let prev = ecm.get::<Stunned>(target);
                    Stunned{duration: prev.duration + duration, .. prev}
                } else {
                    Stunned{turn: *self.current_turn(), duration: duration}
                };
                ecm.set(target, stunned);
            }
            Panic{duration} => {
                println!("Entity {:?} panics because of {:?}", target, attacker);
                // An attacker with stun disappears after delivering the blow
                ecm.set(attacker, FadeOut{to: Color{r: 0, g: 0, b: 0}, duration_s: 0.4});
                if ecm.has::<Tile>(attacker) {
                    // TODO: why are we decrementing the tile level here? Looks like
                    // a hack for a faulty display logic to me.
                    let tile = ecm.get::<Tile>(attacker);
                    if tile.level > 0 {
                        ecm.set(attacker, Tile{level: tile.level - 1, .. tile});
                    }
                }
                kill_entity(attacker, ecm);
                let panicking = if ecm.has::<Panicking>(target) {
                    let prev = ecm.get::<Panicking>(target);
                    Panicking{duration: prev.duration + duration, .. prev}
                } else {
                    Panicking{turn: *self.current_turn(), duration: duration}
                };
                ecm.set(target, panicking);
            }
            ModifyAttributes => {
                if !ecm.has::<AttributeModifier>(attacker) {
                    fail!("The attacker must have attribute_modifier");
                }
                let modifier = ecm.get::<AttributeModifier>(attacker);
                if ecm.has::<Attributes>(target) {
                    let attrs = ecm.get::<Attributes>(target);
                    ecm.set(target, Attributes{
                        state_of_mind: attrs.state_of_mind + modifier.state_of_mind,
                        will: attrs.will + modifier.will})
                }
            }
        }
    }
}
