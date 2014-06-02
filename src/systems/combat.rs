use ecm::{ComponentManager, ECM, Entity};
use engine::Color;
use components::{Anxiety, AnxietyKillCounter, AI, AttackTarget, AttackType,
                 Attributes, AttributeModifier,
                 AcceptsUserInput, Count, Corpse, Destination,
                 Kill, Monster, Panic, Panicking, Position, Solid, Stun,
                 Stunned, Tile, Turn,  Sec};
use entity_util;


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
                println!("Entity {} was killed by {}", target, attacker);
                entity_util::kill(ecm, target);
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
                println!("Entity {} was stunned by {}", target, attacker);
                // An attacker with stun disappears after delivering the blow
                entity_util::fade_out(ecm, attacker, Color{r: 0, g: 0, b: 0}, Sec(0.4));
                entity_util::kill(ecm, attacker);
                let stunned = if ecm.has::<Stunned>(target) {
                    let prev = ecm.get::<Stunned>(target);
                    Stunned{duration: prev.duration + duration, .. prev}
                } else {
                    Stunned{turn: *self.current_turn(), duration: duration}
                };
                ecm.set(target, stunned);
            }
            Panic{duration} => {
                println!("Entity {} panics because of {}", target, attacker);
                // An attacker with stun disappears after delivering the blow
                entity_util::fade_out(ecm, attacker, Color{r: 0, g: 0, b: 0}, Sec(0.4));
                entity_util::kill(ecm, attacker);
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
