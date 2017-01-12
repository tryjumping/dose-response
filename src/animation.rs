use color::Color;
use point::Point;

use time::Duration;


// TODO: prolly refactor to a struct?
// Fields: position, max radius, current radius, colour, elapsed time
pub type Explosion = Option<(Point, i32, i32, Color, Duration)>;
