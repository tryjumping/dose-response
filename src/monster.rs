use self::Kind::*;

use crate::{
    ai::{self, AIState, Behavior, Update},
    blocker::Blocker,
    color::{self, Color},
    game::Action,
    player::{Modifier, PlayerInfo},
    point::Point,
    random::Random,
    ranged_int::{InclusiveRange, Ranged},
    world::World,
};

use std::fmt::{Display, Error, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Monster {
    pub kind: Kind,
    /// The *world position* of the monster
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

    pub ap: Ranged,
}

// TODO: we should make the various behaviours dependent on data
// assigned to the entity, rather then matching over the entity type
// every time.
#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind {
    Anxiety,
    Depression,
    Hunger,
    Shadows,
    Voices,
    Npc,
    Signpost,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CompanionBonus {
    DoubleWillGrowth,
    HalveExhaustion,
    ExtraActionPoint,
    Victory,
}

impl CompanionBonus {
    pub fn random(rng: &mut Random) -> CompanionBonus {
        use self::CompanionBonus::*;
        match rng.range_inclusive(0, 2) {
            0 => DoubleWillGrowth,
            1 => HalveExhaustion,
            2 => ExtraActionPoint,
            _ => unreachable!(),
        }
    }
}

impl Display for CompanionBonus {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use self::CompanionBonus::*;
        let s = match *self {
            DoubleWillGrowth => "Faster Will Gain",
            HalveExhaustion => "Slow Exhaustion",
            ExtraActionPoint => "Extra AP",
            Victory => "Victory",
        };
        f.write_str(s)
    }
}

impl Monster {
    pub fn new(kind: Kind, position: Point) -> Monster {
        let die_after_attack = match kind {
            Shadows | Voices => true,
            Anxiety | Depression | Hunger | Npc | Signpost => false,
        };

        let max_ap = match kind {
            Depression => 2,
            Anxiety | Hunger | Shadows | Voices | Npc => 1,
            Signpost => 0,
        };

        let behavior = match kind {
            Depression => Behavior::LoneAttacker,
            Anxiety => Behavior::LoneAttacker,
            Hunger => Behavior::PackAttacker,
            Shadows => Behavior::LoneAttacker,
            Voices => Behavior::LoneAttacker,
            Npc => Behavior::Friendly,
            Signpost => Behavior::Immobile,
        };

        let invincible = match kind {
            Npc | Signpost => true,
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
            Signpost => color::signpost,
        };

        Monster {
            kind,
            position,
            dead: false,
            die_after_attack,
            invincible,
            behavior,
            ai_state: AIState::Idle,
            ap: Ranged::new_min(InclusiveRange(0, max_ap)),
            blockers,
            path: vec![],
            trail: None,
            color,
            companion_bonus: None,
            accompanying_player: false,
        }
    }

    pub fn attack_damage(&self) -> Modifier {
        use crate::player::Modifier::*;
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
            Npc | Signpost => Attribute {
                will: 0,
                state_of_mind: 0,
            }, // NOTE: no-op
        }
    }

    pub fn act(
        &self,
        player_info: PlayerInfo,
        world: &mut World,
        rng: &mut Random,
    ) -> (Update, Action) {
        if self.dead {
            panic!("{:?} is dead, cannot run actions on it.", self);
        }
        match self.behavior {
            Behavior::LoneAttacker => ai::lone_attacker_act(self, player_info, world, rng),
            Behavior::PackAttacker => ai::pack_attacker_act(self, player_info, world, rng),
            Behavior::Friendly => ai::friendly_act(self, player_info, world, rng),
            Behavior::Immobile => ai::noop_act(self, player_info, world, rng),
        }
    }

    pub fn spend_ap(&mut self, count: i32) {
        let ap = self.ap.to_int();
        assert!(count <= ap);
        self.ap -= count;
    }

    pub fn has_ap(&self, count: i32) -> bool {
        !self.dead && self.ap.to_int() >= count
    }

    pub fn new_turn(&mut self) {
        if !self.dead {
            self.ap.set_to_max();
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
            Signpost => '!',
        }
    }

    pub fn name(&self) -> &str {
        match self.kind {
            Anxiety => "Anxiety",
            Depression => "Depression",
            Hunger => "Hunger",
            Shadows => "Shadows",
            Voices => "Voices",
            Npc => "NPC",
            Signpost => "signpost",
        }
    }
}

impl std::fmt::Display for Monster {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Monster {{")?;
        write!(f, "name: {}, position: {:?}", self.name(), self.position)?;
        write!(f, "}}")
    }
}
