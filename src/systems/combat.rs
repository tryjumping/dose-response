use std::time::Duration;
use emhyr::{Components, Entity};
use engine::Color;
use components::{Anxiety, AnxietyKillCounter, AttackTarget, AttackType,
                 Attributes, AttributeModifier, ModifyAttributes,
                 Kill, Monster, Panic, Panicking, Position, Stun,
                 Stunned, Turn};
use entity_util;


define_system! {
    name: CombatSystem;
    components(AttackTarget, AttackType, Turn);
    resources(player: Entity, current_turn: int);
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, attacker: Entity) {
        let free_aps = cs.get::<Turn>(attacker).ap;
        let AttackTarget(target) = cs.get::<AttackTarget>(attacker);
        cs.unset::<AttackTarget>(attacker);
        let attack_successful = free_aps > 0;
        if !attack_successful {return}
        // attacker spends an AP
        let turn = cs.get::<Turn>(attacker);
        cs.set(turn.spend_ap(1), attacker);
        match cs.get::<AttackType>(attacker) {
            Kill => {
                println!("{} was killed by {}", target, attacker);
                entity_util::kill(cs, target);
                // TODO: This is a hack. The player should fade out, the other
                // monsters just disappear. Need to make this better without
                // special-casing the player.
                if target != *self.player() {
                    cs.unset::<Position>(target);
                }
                let target_is_anxiety = cs.has::<Monster>(target) &&
                    cs.get::<Monster>(target).kind == Anxiety;
                if target_is_anxiety && cs.has::<AnxietyKillCounter>(attacker) {
                    let counter = cs.get::<AnxietyKillCounter>(attacker);
                    cs.set(AnxietyKillCounter{
                        count: counter.count + 1,
                        .. counter
                    }, attacker);
                }
            }
            Stun{duration} => {
                println!("{} was stunned by {}", target, attacker);
                // An attacker with stun disappears after delivering the blow
                entity_util::fade_out(cs, attacker, Color{r: 0, g: 0, b: 0}, Duration::milliseconds(400));
                entity_util::kill(cs, attacker);
                let stunned = if cs.has::<Stunned>(target) {
                    let prev = cs.get::<Stunned>(target);
                    Stunned{duration: prev.duration + duration, .. prev}
                } else {
                    Stunned{turn: *self.current_turn(), duration: duration}
                };
                cs.set(stunned, target);
            }
            Panic{duration} => {
                println!("{} panics because of {}", target, attacker);
                // An attacker with stun disappears after delivering the blow
                entity_util::fade_out(cs, attacker, Color{r: 0, g: 0, b: 0}, Duration::milliseconds(400));
                entity_util::kill(cs, attacker);
                let panicking = if cs.has::<Panicking>(target) {
                    let prev = cs.get::<Panicking>(target);
                    Panicking{duration: prev.duration + duration, .. prev}
                } else {
                    Panicking{turn: *self.current_turn(), duration: duration}
                };
                cs.set(panicking, target);
            }
            ModifyAttributes => {
                if !cs.has::<AttributeModifier>(attacker) {
                    fail!("The attacker must have attribute_modifier");
                }
                let modifier = cs.get::<AttributeModifier>(attacker);
                if cs.has::<Attributes>(target) {
                    let attrs = cs.get::<Attributes>(target);
                    cs.set(Attributes{
                        state_of_mind: attrs.state_of_mind + modifier.state_of_mind,
                        will: attrs.will + modifier.will}, target)
                }
            }
        }
    }
}
