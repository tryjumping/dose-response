use self::Kind::*;

use ai::{self, AIState, Behavior};
use color::{self, Color};
use game::Action;
use graphics::Render;
use player::Modifier;
use point::Point;

use rand::Rng;
use time::Duration;
use world::World;


#[derive(Clone, PartialEq, Debug)]
pub struct Monster {
    pub kind: Kind,
    pub position: Point,
    pub dead: bool,
    pub die_after_attack: bool,
    pub invincible: bool,
    pub behavior: Behavior,
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
    Npc,
}

impl Monster {
    pub fn new(kind: Kind, position: Point) -> Monster {
        let die_after_attack = match kind {
            Shadows | Voices => true,
            Anxiety | Depression | Hunger | Npc => false,
        };
        let max_ap = match kind {
            Depression => 2,
            Anxiety | Hunger | Shadows | Voices | Npc => 1,
        };
        let behavior = match kind {
            Depression => Behavior::LoneAttacker,
            Anxiety => Behavior::LoneAttacker,
            Hunger => Behavior::PackAttacker,
            Shadows => Behavior::LoneAttacker,
            Voices => Behavior::LoneAttacker,
            Npc => Behavior::Friendly,
        };
        let invincible = match kind {
            Npc => true,
            _ => false,
        };
        Monster {
            kind,
            position,
            dead: false,
            die_after_attack,
            invincible,
            behavior,
            ai_state: AIState::Idle,
            ap: 0,
            max_ap,
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
            Npc => Attribute { will: 0, state_of_mind: 0 },  // NOTE: no-op
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
        let (ai_state, action) = match self.behavior {
            Behavior::LoneAttacker => {
                ai::lone_attacker_act(self, player_pos, world, rng)
            }
            Behavior::PackAttacker => {
                ai::pack_attacker_act(self, player_pos, world, rng)
            }
            Behavior::Friendly => {
                ai::friendly_act(self, player_pos, world, rng)
            }
        };
        (ai_state, action)
    }

    pub fn spend_ap(&mut self, count: i32) {
        if !(count <= self.ap) {
            // println!("bad assert: {:?}", self);
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
            Npc => ('@', color::npc, None),
        }
    }
}
