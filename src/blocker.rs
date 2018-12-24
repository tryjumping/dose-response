use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    #[derive(Serialize, Deserialize)]
    /// Flag to indicate features that block pathfinding/walking.
    pub struct Blocker: u32 {
        const WALL    = 0b0000_0001;
        const MONSTER = 0b0000_0010;
        const PLAYER  = 0b0000_0100;
    }
}
