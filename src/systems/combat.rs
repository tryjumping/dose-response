use components::*;
use engine::Color;
use super::super::Resources;
use util::Deref;

pub fn kill_entity(e: ID,
                   ecm: &mut ComponentManager) {
    if !ecm.has_entity(e) {return}
    // TODO: we assume that an entity without a turn is already dead. Add a
    // `Dead` component (or something similar) instead.
    // TODO: also, this is a bug: killing should be idempotent
    if !ecm.has_turn(e) {return}
    ecm.remove_ai(e);
    ecm.remove_accepts_user_input(e);
    ecm.remove_turn(e);
    ecm.remove_destination(e);
    let solid_corpse = ecm.has_corpse(e) && ecm.get_corpse(e).solid;
    if !solid_corpse {
        ecm.remove_solid(e);
    }
    // Replace the entity's Tile with the tile of a corpse.
    if ecm.has_corpse(e) && ecm.has_tile(e) {
        let corpse = ecm.get_corpse(e);
        let tile = ecm.get_tile(e);
        ecm.set(e, Tile{glyph: corpse.glyph,
                             color: corpse.color,
                             .. tile});
        ecm.set(e, FadeColor{
                from: tile.color,
                to: corpse.color,
                duration_s: 1f32,
                repetitions: Count(1),
            });
    } else if ecm.has_fade_out(e) {
        // TODO: we probably shouldn't remove the fading-out entities here.
        // Makes no sense. Just remove their tiles after the fadeout.
    } else {
        ecm.remove_tile(e);
    }
}

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    ensure_components!(ecm, e, AttackTarget, AttackType, Turn);
    let free_aps = ecm.get_turn(e).ap;
    let target = ecm.get_attack_target(e).deref();
    ecm.remove_attack_target(e);
    let attack_successful = ecm.has_entity(target) && free_aps > 0;
    if !attack_successful {return}
    // attacker spends an AP
    let turn = ecm.get_turn(e);
    ecm.set(e, turn.spend_ap(1));
    match ecm.get_attack_type(e) {
        Kill => {
            println!("Entity {} was killed by {}", target.deref(), e.deref());
            kill_entity(target, ecm);
            // TODO: This is a hack. The player should fade out, the other
            // monsters just disappear. Need to make this better without
            // special-casing the player.
            if target != res.player_id {
                ecm.remove_position(target);
            }
            let target_is_anxiety = (ecm.has_monster(target) &&
                                     ecm.get_monster(target).kind == Anxiety);
            if target_is_anxiety && ecm.has_anxiety_kill_counter(e) {
                let counter = ecm.get_anxiety_kill_counter(e);
                ecm.set(e, AnxietyKillCounter{
                        count: counter.count + 1,
                        .. counter
                    });
            }
        }
        Stun{duration} => {
            println!("Entity {} was stunned by {}", target.deref(), e.deref());
            // An attacker with stun disappears after delivering the blow
            ecm.set_fade_out(e, FadeOut{to: Color{r: 0, g: 0, b: 0}, duration_s: 0.4});
            if ecm.has_tile(e) {
                let tile = ecm.get_tile(e);
                if tile.level > 0 {
                    ecm.set(e, Tile{level: tile.level - 1, .. tile});
                }
            }
            kill_entity(e, ecm);
            let stunned = if ecm.has_stunned(target) {
                let prev = ecm.get_stunned(target);
                Stunned{duration: prev.duration + duration, .. prev}
            } else {
                Stunned{turn: res.turn, duration: duration}
            };
            ecm.set_stunned(target, stunned);
        }
        Panic{duration} => {
            println!("Entity {} panics because of {}", target.deref(), e.deref());
            // An attacker with stun disappears after delivering the blow
            ecm.set_fade_out(e, FadeOut{to: Color{r: 0, g: 0, b: 0}, duration_s: 0.4});
            if ecm.has_tile(e) {
                let tile = ecm.get_tile(e);
                if tile.level > 0 {
                    ecm.set(e, Tile{level: tile.level - 1, .. tile});
                }
            }
            kill_entity(e, ecm);
            let panicking = if ecm.has_panicking(target) {
                let prev = ecm.get_panicking(target);
                Panicking{duration: prev.duration + duration, .. prev}
            } else {
                Panicking{turn: res.turn, duration: duration}
            };
            ecm.set_panicking(target, panicking);
        }
        ModifyAttributes => {
            if !ecm.has_attribute_modifier(e) {
                fail!("The attacker must have attribute_modifier");
            }
            let modifier = ecm.get_attribute_modifier(e);
            if ecm.has_attributes(target) {
                let attrs = ecm.get_attributes(target);
                ecm.set(target, Attributes{
                        state_of_mind: attrs.state_of_mind + modifier.state_of_mind,
                        will: attrs.will + modifier.will})
            }
        }
    }
}
