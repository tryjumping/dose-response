use time::Duration;

use rand::Rng;

use game::Action;
use color::{self, Color};
use level::Walkability;
use graphics::Render;
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
    pub path: Vec<Point>,
    pub trail: Option<Point>,

    pub max_ap: i32,
    ap: i32,
}


#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord,
         Serialize, Deserialize)]
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
            path: vec![],
            trail: None,
        }
    }

    pub fn attack_damage(&self) -> Modifier {
        use player::Modifier::*;
        match self.kind {
            Anxiety => Attribute { will: -1, state_of_mind: 0 },
            Depression => Death,
            Hunger => Attribute { will: 0, state_of_mind: -20 },
            Shadows => Panic(4),
            Voices => Stun(4),
        }
    }

    pub fn act<R: Rng>(&self,
                       player_pos: Point,
                       world: &mut World,
                       rng: &mut R)
                       -> (AIState, Action) {
        if self.dead {
            panic!(format!("{:?} is dead, cannot run actions on it.", self));
        }
        let distance = self.position.tile_distance(player_pos);
        let ai_state = if distance <= 5 { Chasing } else { Idle };

        let action = match self.ai_state {
            Chasing => {
                if distance == 1 {
                    Action::Attack(player_pos, self.attack_damage())
                } else {
                    Action::Move(player_pos)
                }
            }
            Idle => {
                let destination = if self.path.is_empty() {
                    // Move randomly about
                    let mut destination =
                        world.random_neighbour_position(
                            rng,
                            self.position,
                            Walkability::BlockingMonsters);

                    for _ in 0..10 {
                        let x = rng.gen_range(-8, 9);
                        let y = rng.gen_range(-8, 9);
                        let candidate = self.position + (x, y);
                        if x.abs() > 2 && y.abs() > 2 &&
                           world.walkable(candidate,
                                          Walkability::WalkthroughMonsters) {
                            destination = candidate;
                            break;
                        }
                    }
                    destination
                } else {
                    // We already have a path, just set the same destination:
                    *self.path.last().unwrap()
                };
                Action::Move(destination)
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
            self.trail = None;
        }
    }

    pub fn alive(&self) -> bool {
        !self.dead
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
