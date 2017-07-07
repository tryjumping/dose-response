use formula;
use game::Action;
use monster::Monster;
use player::Mind;
use point::Point;
use rand::Rng;
use ranged_int::InclusiveRange;
use rect::Rectangle;
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
    CheckingOut(Point),
}


#[derive(Copy, Clone, PartialEq, Debug)]
/// Values the AI can update about itself before performing the action
/// it decided to make.
pub struct Update {
    pub ai_state: AIState,
    pub max_ap: i32,
}

/// Values related to the Player the AI routines might want to
/// investigate.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PlayerInfo {
    pub pos: Point,
    pub mind: Mind,
    pub max_ap: i32,
}


pub fn lone_attacker_act<R: Rng>(
    actor: &Monster,
    player_info: PlayerInfo,
    world: &mut World,
    rng: &mut R,
) -> (Update, Action) {
    let distance = actor.position.tile_distance(player_info.pos);
    let ai_state = if distance <= formula::CHASING_DISTANCE {
        AIState::Chasing
    } else {
        AIState::Idle
    };

    let update = Update { ai_state, max_ap: actor.max_ap };

    let action = match ai_state {
        AIState::Chasing => chasing_action(actor, player_info.pos),
        AIState::Idle => {
            let destination = idle_destination(actor, world, rng, player_info.pos);
            Action::Move(destination)
        }
        AIState::CheckingOut(destination) => Action::Move(destination),
    };
    (update, action)
}


pub fn pack_attacker_act<R: Rng>(
    actor: &Monster,
    player_info: PlayerInfo,
    world: &mut World,
    rng: &mut R,
) -> (Update, Action) {
    let player_distance = actor.position.tile_distance(player_info.pos);
    let ai_state = if player_distance <= formula::CHASING_DISTANCE {
        AIState::Chasing
    } else if actor.ai_state == AIState::Chasing {
        AIState::Idle
    } else {
        actor.ai_state
    };

    let update = Update { ai_state, max_ap: actor.max_ap };

    let action = match ai_state {
        AIState::Chasing => {
            let howling_area =
                Rectangle::center(actor.position, Point::from_i32(formula::HOWLING_DISTANCE));
            let howlees = world.monsters(howling_area)
                .filter(|m| {
                    m.behavior == Behavior::PackAttacker && m.position != actor.position
                })
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
    };
    (update, action)
}


pub fn friendly_act<R: Rng>(
    actor: &Monster,
    player_info: PlayerInfo,
    world: &mut World,
    rng: &mut R,
) -> (Update, Action) {
    let destination = idle_destination(actor, world, rng, player_info.pos);
    let update = Update {
        ai_state: actor.ai_state,
        max_ap: if player_info.mind.is_high() {
            formula::ESTRANGED_NPC_MAX_AP
        } else {
            player_info.max_ap
        }
    };

    let action = Action::Move(destination);
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
            )
            .unwrap_or_else(|| {
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
