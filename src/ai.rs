use rand::Rng;

use formula;
use game::Action;
use monster::Monster;
use level::Walkability;
use point::Point;
use ranged_int::InclusiveRange;
use rect::Rectangle;
use world::{Chunk, World};


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
    CheckingOut(Point),
}


pub fn lone_attacker_act<R: Rng>(actor: &Monster,
                                 player_position: Point,
                                 world: &mut World,
                                 rng: &mut R)
                                 -> (AIState, Action) {
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
                    .random_position_in_range(rng,
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
        AIState::CheckingOut(destination) => Action::Move(destination)
    };
    (ai_state, action)
}


pub fn pack_attacker_act<R: Rng>(actor: &Monster,
                                 player_position: Point,
                                 world: &mut World,
                                 rng: &mut R)
                                 -> (AIState, Action) {
    // if seeing player: howl, start chasing
    // if within howling range: start chasing



    let distance = actor.position.tile_distance(player_position);
    let ai_state = if distance <= formula::CHASING_DISTANCE {
        AIState::Chasing
    } else if actor.ai_state == AIState::Chasing {
        AIState::Idle
    } else {
        actor.ai_state
    };

    let action = match ai_state {
        AIState::Chasing => {
            let howling_area = Rectangle::center(actor.position,
                                                 (formula::HOWLING_DISTANCE,
                                                  formula::HOWLING_DISTANCE)
                                                 .into());
            let howlees = world
                .chunks(howling_area)
                .flat_map(Chunk::monsters)
                .filter(|m| m.alive() && howling_area.contains(m.position))
                .filter(|m| {
                    m.behavior == Behavior::PackAttacker &&
                        m.position != actor.position
                })
                .map(|m| m.position)
                .collect::<Vec<_>>();

            for pos in howlees {
                if let Some(monster) = world.monster_on_pos(pos) {
                    monster.ai_state = AIState::CheckingOut(player_position);
                }
            }

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
                    .random_position_in_range(rng,
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
        AIState::CheckingOut(destination) => {
            Action::Move(destination)
        }
    };
    (ai_state, action)
}


pub fn friendly_act<R: Rng>(_actor: &Monster,
                            _player_position: Point,
                            _world: &mut World,
                            _rng: &mut R)
                            -> (AIState, Action) {
    unreachable!()
}
