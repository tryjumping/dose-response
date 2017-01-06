use time::Duration;

use rand::Rng;

use super::Action;
use color::{self, Color};
use level::Walkability;
use graphics::Render;
use pathfinding::Path;
use player::Modifier;
use point::Point;
use world::World;

use self::Kind::*;
use self::AIState::*;


#[derive(Clone, PartialEq, Debug)]
pub struct Monster {
    pub kind: Kind,
    pub position: Point,
    pub dead: bool,
    pub die_after_attack: bool,
    pub ai_state: AIState,

    pub max_ap: i32,
    ap: i32,
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Kind {
    Anxiety,
    Depression,
    Hunger,
    Shadows,
    Voices,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AIState {
    Idle,
    Chasing,
}

impl Monster {
    pub fn new(kind: Kind, position: Point) -> Monster {
        let die_after_attack = match kind {
            Shadows | Voices => true,
            Anxiety | Depression | Hunger => false,
        };
        let max_ap = match kind {
            Depression => 2,
            Anxiety | Hunger | Shadows | Voices => 1,
        };
        Monster {
            kind: kind,
            position: position,
            dead: false,
            die_after_attack: die_after_attack,
            ai_state: Idle,
            ap: 0,
            max_ap: max_ap,
        }
    }

    pub fn attack_damage(&self) -> Modifier {
        use player::Modifier::*;
        match self.kind {
            Anxiety => Attribute{will: -1, state_of_mind: 0},
            Depression => Death,
            Hunger => Attribute{will: 0, state_of_mind: -20},
            Shadows => Panic(4),
            Voices => Stun(4),
        }
    }

    pub fn act<R: Rng>(&self, player_pos: Point, world: &mut World, rng: &mut R) -> (AIState, Action) {
        if self.dead {
            panic!(format!("{:?} is dead, cannot run actions on it.", self));
        }
        let distance = self.position.tile_distance(player_pos);
        let ai_state = if distance <= 5 {
            Chasing
        } else {
            Idle
        };

        let random_neighbouring_pos = world.random_neighbour_position(
            rng, self.position, Walkability::BlockingMonsters);
        let action = match self.ai_state {
            Chasing => {
                if distance == 1 {
                    Action::Attack(player_pos, self.attack_damage())
                } else {
                    let mut path = Path::find(
                        self.position, player_pos, world, Walkability::BlockingMonsters);
                    Action::Move(path.next().unwrap_or(random_neighbouring_pos))
                }
            }
            Idle => {
                // Move randomly about
                Action::Move(random_neighbouring_pos)
            }
        };
        (ai_state, action)
    }

    pub fn spend_ap(&mut self, count: i32) {
        if !(count <= self.ap) {
            //println!("bad assert: {:?}", self);
        }
        assert!(count <= self.ap);
        self.ap -= count;
    }

    pub fn has_ap(&self, count: i32) -> bool {
        !self.dead && self.ap >= count
    }

    pub fn new_turn(&mut self) {
        if !self.dead {
            self.ap = self.max_ap;
        }
    }
}


impl Render for Monster {
    fn render(&self, _dt: Duration) -> (char, Color, Option<Color>) {
        match self.kind {
            Anxiety => ('a', color::anxiety, None),
            Depression => ('D', color::depression, None),
            Hunger => ('h', color::hunger, None),
            Shadows => ('S', color::voices, None),
            Voices => ('v', color::shadows, None),
        }
    }
}
