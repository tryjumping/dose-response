bitflags! {
    /// Flag to indicate features that block pathfinding/walking.
    pub struct Blocker: u32 {
        const WALL    = 0b00000001;
        const MONSTER = 0b00000010;
        const PLAYER  = 0b00000100;
    }
}
