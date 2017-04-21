use rand::Rng;

use formula;
use game::Action;
use monster::Monster;
use level::Walkability;
use point::Point;
use ranged_int::InclusiveRange;
use world::World;


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Behavior {
    LoneAttacker,
    PackAttacker,
    Friendly,
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AIState {
    Idle,
    Chasing,
}


pub fn lone_attacker_act<R: Rng>(actor: &Monster,
                                 player_position: Point,
                                 world: &mut World,
                                 rng: &mut R) -> (AIState, Action)
{
    let distance = actor.position.tile_distance(player_position);
    let ai_state = if distance <= formula::CHASING_DISTANCE {
        AIState::Chasing
    } else {
        AIState::Idle
    };

    let action = match ai_state {
        AIState::Chasing => {
            if distance == 1 {
                Action::Attack(player_position, actor.attack_damage())
            } else {
                Action::Move(player_position)
            }
        }
        AIState::Idle => {
            let destination = if actor.path.is_empty() {
                // Move randomly about
                world
                    .random_position_in_range(
                        rng,
                        actor.position,
                        InclusiveRange(2, 8),
                        10,
                        Walkability::WalkthroughMonsters)
                    .unwrap_or_else(|| {
                        world.random_neighbour_position(
                            rng,
                            actor.position,
                            Walkability::BlockingMonsters)
                    })
            } else {
                // We already have a path, just set the same destination:
                *actor.path.last().unwrap()
            };
            Action::Move(destination)
        }
    };
    (ai_state, action)
}
