use self::Kind::*;

use crate::{
    ai::{self, AIState, Behavior, Update},
    blocker::Blocker,
    color::Color,
    formula,
    game::Action,
    graphic::Graphic,
    palette::Palette,
    player::{Modifier, PlayerInfo},
    point::Point,
    random::Random,
    ranged_int::{InclusiveRange, Ranged},
    state::Challenge,
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
    pub npc_color_index: usize,
    pub die_after_attack: bool,
    pub invincible: bool,
    pub behavior: Behavior,
    pub ai_state: AIState,
    pub blockers: Blocker,
    pub path: Vec<Point>,
    pub trail: Option<Point>,
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

impl Kind {
    pub fn is_monster(&self) -> bool {
        match *self {
            Anxiety => true,
            Depression => true,
            Hunger => true,
            Shadows => true,
            Voices => true,
            Npc => false,
            Signpost => false,
        }
    }
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
    pub fn new(kind: Kind, position: Point, challenge: Challenge) -> Monster {
        let die_after_attack = match kind {
            Shadows | Voices => true,
            Anxiety | Depression | Hunger | Npc | Signpost => false,
        };

        let max_ap = match kind {
            Depression => formula::depression_max_ap(challenge),
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

        let invincible = matches!(kind, Npc | Signpost);

        // NOTE: NPCs can't walk into the player, monsters can
        let blockers = match kind {
            Npc => Blocker::PLAYER | Blocker::WALL | Blocker::MONSTER,
            // NOTE: letting monsters walk through monsters,
            // pathfinding will sort out the costs
            _ => Blocker::WALL,
        };

        Monster {
            kind,
            position,
            dead: false,
            npc_color_index: 0,
            die_after_attack,
            invincible,
            behavior,
            ai_state: AIState::Idle,
            ap: Ranged::new_min(InclusiveRange(0, max_ap)),
            blockers,
            path: vec![],
            trail: None,
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
            Shadows => Panic(3),
            Voices => Stun(3),
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

    pub fn graphic(&self) -> Graphic {
        match self.kind {
            Anxiety => Graphic::Anxiety,
            Depression => Graphic::Depression,
            Hunger => Graphic::Hunger,
            Shadows => Graphic::Shadows,
            Voices => Graphic::Voices,
            Npc => match self.companion_bonus {
                Some(CompanionBonus::DoubleWillGrowth) => Graphic::CharacterTribalStaffTrousers,
                Some(CompanionBonus::HalveExhaustion) => Graphic::CharacterTribalStaffBelly,
                Some(CompanionBonus::ExtraActionPoint) => Graphic::CharacterTribalMoon,
                Some(CompanionBonus::Victory) => Graphic::CharacterBelly,
                None => Graphic::CharacterBelly,
            },
            Signpost => Graphic::Signpost,
        }
    }

    pub fn color(&self, palette: &Palette) -> Color {
        match self.kind {
            Depression => palette.depression,
            Anxiety => palette.anxiety,
            Hunger => palette.hunger,
            Shadows => palette.shadows,
            Voices => palette.voices,
            // TODO: Add dim colours when the player is high? OR do we do that elsewhere?
            Npc => match self.companion_bonus {
                Some(CompanionBonus::DoubleWillGrowth) => palette.npc_will,
                Some(CompanionBonus::HalveExhaustion) => palette.npc_mind,
                Some(CompanionBonus::ExtraActionPoint) => palette.npc_speed,
                // TODO: add vnpc colours
                Some(CompanionBonus::Victory) => palette.player(self.npc_color_index),
                None => Color::default(),
            },
            Signpost => palette.signpost,
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
