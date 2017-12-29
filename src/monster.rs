use self::Kind::*;

use ai::{self, AIState, Behavior, PlayerInfo, Update};
use blocker::Blocker;
use color::{self, Color};
use game::Action;
use graphics::Render;
use player::Modifier;
use point::Point;

use rand::{Rand, Rng};
use std::time::Duration;
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
    pub blockers: Blocker,
    pub path: Vec<Point>,
    pub trail: Option<Point>,
    pub color: Color,
    pub companion_bonus: Option<CompanionBonus>,
    pub accompanying_player: bool,

    pub max_ap: i32,
    ap: i32,
}


#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind {
    Anxiety,
    Depression,
    Hunger,
    Shadows,
    Voices,
    Npc,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CompanionBonus {
    DoubleWillGrowth,
    HalveExhaustion,
    DoubleActionPoints,
}


// TODO: use a rand_derive or something.
// I tried it 2017-07-28 and it wasn't working then.
impl Rand for CompanionBonus {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        use self::CompanionBonus::*;
        match rng.gen_range(0, 3) {
            0 => DoubleWillGrowth,
            1 => HalveExhaustion,
            2 => DoubleActionPoints,
            _ => unreachable!(),
        }
    }
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

        // NOTE: NPCs can't walk into the player, monsters can
        let blockers = match kind {
            Npc => Blocker::PLAYER | Blocker::WALL | Blocker::MONSTER,
            _ => Blocker::WALL | Blocker::MONSTER,
        };

        let color = match kind {
            Depression => color::depression,
            Anxiety => color::anxiety,
            Hunger => color::hunger,
            Shadows => color::shadows,
            Voices => color::voices,
            Npc => color::npc_speed,
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
            blockers,
            path: vec![],
            trail: None,
            color,
            companion_bonus: None,
            accompanying_player: false,
        }
    }

    pub fn attack_damage(&self) -> Modifier {
        use player::Modifier::*;
        match self.kind {
            Anxiety => Attribute {
                will: -1,
                state_of_mind: 0,
            },
            Depression => Death,
            Hunger => Attribute {
                will: 0,
                state_of_mind: -20,
            },
            Shadows => Panic(4),
            Voices => Stun(4),
            Npc => Attribute {
                will: 0,
                state_of_mind: 0,
            },  // NOTE: no-op
        }
    }

    pub fn act<R: Rng>(
        &self,
        player_info: PlayerInfo,
        world: &mut World,
        rng: &mut R,
    ) -> (Update, Action) {
        if self.dead {
            panic!(format!("{:?} is dead, cannot run actions on it.", self));
        }
        match self.behavior {
            Behavior::LoneAttacker => ai::lone_attacker_act(self, player_info, world, rng),
            Behavior::PackAttacker => ai::pack_attacker_act(self, player_info, world, rng),
            Behavior::Friendly => ai::friendly_act(self, player_info, world, rng),
        }
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

    pub fn glyph(&self) -> char {
        match self.kind {
            Anxiety => 'a',
            Depression => 'D',
            Hunger => 'h',
            Shadows => 'S',
            Voices => 'v',
            Npc => '@',
        }
    }
}


impl Render for Monster {
    fn render(&self, _dt: Duration) -> (char, Color, Option<Color>) {
        (self.glyph(), self.color, None)
    }
}
