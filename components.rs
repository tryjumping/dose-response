pub struct Position {x: int, y: int}
pub struct Health(int);

pub struct GameObject {
    position: Option<Position>,
    health: Option<Health>,
}
