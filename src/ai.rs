use crate::formula;
use crate::game::Action;
use crate::monster::Monster;
use crate::player::PlayerInfo;
use crate::point::Point;
use crate::ranged_int::InclusiveRange;
use crate::rect::Rectangle;
use crate::world::World;
use rand::Rng;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Behavior {
    LoneAttacker,
    PackAttacker,
    Friendly,
    Immobile,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum AIState {
    Idle,
    Chasing,
    CheckingOut(Point),
    NoOp,
}

#[derive(Copy, Clone, PartialEq, Debug)]
/// Values the AI can update about itself before performing the action
/// it decided to make.
pub struct Update {
    pub ai_state: AIState,
    pub max_ap: i32,
}

pub fn lone_attacker_act<R: Rng>(
    actor: &Monster,
    player_info: PlayerInfo,
    world: &mut World,
    rng: &mut R,
) -> (Update, Action) {
    if actor.ai_state == AIState::NoOp {
        return noop_action(actor);
    }
    let distance = actor.position.tile_distance(player_info.pos);
    let ai_state = if distance <= formula::CHASING_DISTANCE {
        AIState::Chasing
    } else {
        AIState::Idle
    };

    let update = Update {
        ai_state,
        max_ap: actor.ap.max(),
    };

    let action = match ai_state {
        AIState::Chasing => chasing_action(actor, player_info.pos),
        AIState::Idle => {
            let destination = idle_destination(actor, world, rng, player_info.pos);
            Action::Move(destination)
        }
        AIState::CheckingOut(destination) => Action::Move(destination),
        AIState::NoOp => unreachable!(),
    };
    (update, action)
}

pub fn pack_attacker_act<R: Rng>(
    actor: &Monster,
    player_info: PlayerInfo,
    world: &mut World,
    rng: &mut R,
) -> (Update, Action) {
    if actor.ai_state == AIState::NoOp {
        return noop_action(actor);
    }
    let player_distance = actor.position.tile_distance(player_info.pos);
    let ai_state = if player_distance <= formula::CHASING_DISTANCE {
        AIState::Chasing
    } else if actor.ai_state == AIState::Chasing {
        AIState::Idle
    } else {
        actor.ai_state
    };

    let update = Update {
        ai_state,
        max_ap: actor.ap.max(),
    };

    let action = match ai_state {
        AIState::Chasing => {
            let howling_area =
                Rectangle::center(actor.position, Point::from_i32(formula::HOWLING_DISTANCE));
            let howlees = world
                .monsters(howling_area)
                .filter(|m| m.behavior == Behavior::PackAttacker && m.position != actor.position)
                .map(|m| m.position)
                .collect::<Vec<_>>();

            for pos in howlees {
                if let Some(monster) = world.monster_on_pos(pos) {
                    monster.ai_state = AIState::CheckingOut(player_info.pos);
                }
            }

            chasing_action(actor, player_info.pos)
        }

        AIState::Idle => {
            let destination = idle_destination(actor, world, rng, player_info.pos);
            Action::Move(destination)
        }
        AIState::CheckingOut(destination) => Action::Move(destination),
        AIState::NoOp => unreachable!(),
    };
    (update, action)
}

pub fn friendly_act<R: Rng>(
    actor: &Monster,
    player_info: PlayerInfo,
    world: &mut World,
    rng: &mut R,
) -> (Update, Action) {
    if actor.ai_state == AIState::NoOp {
        return noop_action(actor);
    }
    let player_is_nearby =
        player_info.pos.distance(actor.position) <= formula::FRIENDLY_NPC_FREEZE_RADIUS;

    let destination = if actor.accompanying_player {
        // Pick a position near the player
        world
            .random_position_in_range(
                rng,
                player_info.pos,
                InclusiveRange(1, 3),
                10,
                actor.blockers,
                player_info.pos,
            ).unwrap_or(player_info.pos)
    } else if player_is_nearby && !player_info.mind.is_high() {
        // If the NPC is approachable and nearby, make it stop
        // wandering about so it's easier to actually approach by the
        // player.
        actor.position
    } else {
        idle_destination(actor, world, rng, player_info.pos)
    };

    let update = Update {
        ai_state: actor.ai_state,
        max_ap: if player_info.mind.is_high() {
            formula::ESTRANGED_NPC_MAX_AP
        } else {
            player_info.max_ap
        },
    };

    let action = Action::Move(destination);
    (update, action)
}

pub fn noop_act<R: Rng>(
    actor: &Monster,
    _player_info: PlayerInfo,
    _world: &mut World,
    _rng: &mut R,
) -> (Update, Action) {
    noop_action(actor)
}

pub fn noop_action(actor: &Monster) -> (Update, Action) {
    let update = Update {
        ai_state: actor.ai_state,
        max_ap: actor.ap.max(),
    };
    let action = Action::Move(actor.position);
    (update, action)
}

fn idle_destination<R: Rng>(
    actor: &Monster,
    world: &World,
    rng: &mut R,
    player_position: Point,
) -> Point {
    if actor.path.is_empty() {
        // Move randomly about
        world
            .random_position_in_range(
                rng,
                actor.position,
                InclusiveRange(2, 8),
                10,
                actor.blockers,
                player_position,
            ).unwrap_or_else(|| {
                world.random_neighbour_position(
                    rng,
                    actor.position,
                    actor.blockers,
                    player_position,
                )
            })
    } else {
        // We already have a path, just set the same destination:
        *actor.path.last().unwrap()
    }
}

fn chasing_action(actor: &Monster, target_position: Point) -> Action {
    if actor.position.tile_distance(target_position) == 1 {
        Action::Attack(target_position, actor.attack_damage())
    } else {
        Action::Move(target_position)
    }
}
