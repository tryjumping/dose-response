use crate::{
    blocker,
    point::Point,
    world::{TileContents, World},
};

use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Path {
    path: Vec<Point>,
}

impl Path {
    pub fn find(
        from: Point,
        to: Point,
        world: &World,
        blockers: blocker::Blocker,
        player_position: Point,
        player_will: i32,
        // Are we interested in knowing about irresistible doses in the path?
        check_irresistible: bool,
        calculation_limit: i32,
        cost: &dyn Fn(Point, Point, TileContents) -> f32,
    ) -> Self {
        // TODO: if the cost was a struct/trait rather than a `fn`, we
        // could express the "are we interested in knowing about
        // irresistible doses?" there.
        if from == to {
            return Path { path: vec![] };
        }

        if !world.walkable(to, blockers, player_position) {
            return Path { path: vec![] };
        }

        if from.tile_distance(to) == 1 {
            return Path { path: vec![to] };
        }

        let neighbors = |current: Point| {
            assert!(world.within_bounds(current));
            let dp: [Point; 9] = [
                (-1, -1).into(),
                (-1, 0).into(),
                (-1, 1).into(),
                (0, -1).into(),
                (0, 0).into(),
                (0, 1).into(),
                (1, -1).into(),
                (1, 0).into(),
                (1, 1).into(),
            ];
            dp.iter()
                .map(|&d| current + d)
                .filter(|&point| {
                    world.within_bounds(point) && world.walkable(point, blockers, player_position)
                })
                .map(|point| {
                    (
                        point,
                        world.tile_contents(point, player_will, check_irresistible),
                    )
                })
                .collect::<Vec<_>>()
        };

        let mut frontier = BinaryHeap::new();
        frontier.push(State {
            position: from,
            cost: 0.0,
        });
        let mut came_from = HashMap::new();
        let mut cost_so_far = HashMap::new();

        came_from.insert(from, None);
        cost_so_far.insert(from, 0.0);

        let mut calculation_steps = 0;

        while let Some(current) = frontier.pop() {
            if current.position == to {
                break;
            }
            if calculation_steps >= calculation_limit {
                break;
            }
            calculation_steps += 1;
            let neigh = neighbors(current.position);
            for &(next, tile_contents) in &neigh {
                assert!((current.position.x - next.x).abs() <= 1);
                assert!((current.position.y - next.y).abs() <= 1);
                let new_cost =
                    cost_so_far[&current.position] + cost(current.position, next, tile_contents);
                let val = cost_so_far.entry(next).or_insert(f32::MAX);
                if new_cost < *val {
                    *val = new_cost;
                    let priority = new_cost + heuristic(to, next);
                    frontier.push(State {
                        position: next,
                        cost: priority,
                    });
                    came_from.insert(next, Some(current.position));
                }
            }
        }

        let path = {
            let mut current = to;
            let mut path_buffer = vec![current];
            while current != from {
                match came_from.get(&current) {
                    Some(&Some(new_current)) => {
                        current = new_current;
                        if current != from {
                            path_buffer.push(current);
                        }
                    }
                    Some(&None) => log::error!(
                        "Pathfinding: Every point except for the initial \
                         one (`from`) one should be some."
                    ),
                    None => {
                        path_buffer = vec![];
                        break;
                    }
                }
            }
            path_buffer
        };

        assert_eq!(None, path.iter().find(|&&p| p == from));
        Path { path }
    }

    /// The number of steps to necessary to reach the destination. If
    /// no path was found, it is `0`.
    pub fn len(&self) -> usize {
        self.path.len()
    }

    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }
}

impl Iterator for Path {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.path.pop()
    }
}

/// Calculate the pathfinding cost of moving to the next Point.
///
/// The higher the cost, the harder to move to the tile. The
/// `tile_contents` variable can help determine the underlying cost.
///
/// The destination is expected to be walkable (this function always
/// returns a finite cost).

pub fn direct_cost(_current: Point, _next: Point, tile_contents: TileContents) -> f32 {
    match tile_contents {
        TileContents::Monster => 1.0,
        TileContents::Item => 1.0,
        TileContents::Irresistible => 1.0,
        TileContents::Empty => 1.0,
    }
}

pub fn monster_cost(_current: Point, _next: Point, tile_contents: TileContents) -> f32 {
    match tile_contents {
        TileContents::Monster => 2.1,
        TileContents::Item => 1.0,
        TileContents::Irresistible => 1.0,
        TileContents::Empty => 1.0,
    }
}

pub fn player_cost(_current: Point, _next: Point, tile_contents: TileContents) -> f32 {
    match tile_contents {
        TileContents::Monster => 1.0,
        TileContents::Item => 1.0,
        TileContents::Irresistible => 4.0,
        TileContents::Empty => 1.0,
    }
}

fn heuristic(destination: Point, next: Point) -> f32 {
    ((destination.x - next.x).abs() + (destination.y - next.y).abs()) as f32
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct State {
    cost: f32,
    position: Point,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        assert!(self.cost.is_finite());
        assert!(other.cost.is_finite());
        if other.cost > self.cost {
            Ordering::Greater
        } else if other.cost < self.cost {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod test {
    use super::Path;
    use crate::{
        blocker::Blocker,
        player::{Mind, PlayerInfo},
        point::Point,
        world::World,
    };

    struct Board {
        start: Point,
        destination: Point,
        world: World,
    }

    fn make_board(text: &str) -> Board {
        use crate::{
            level::{
                Tile,
                TileKind::{Empty, Tree},
            },
            random::Random,
        };
        let mut start = Point { x: 0, y: 0 };
        let mut destination = Point { x: 0, y: 0 };
        let mut x = 0;
        let mut y = 0;

        let lines = text
            .split('\n')
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>();
        let height = lines.len();
        assert!(height > 0);
        let width = lines[0].len();
        assert!(width > 0);
        assert!(lines.iter().all(|line| line.chars().count() == width));

        let mut rng = Random::from_seed(0);
        let player_info = PlayerInfo {
            pos: Point::new(0, 0),
            mind: Mind::Sober(crate::ranged_int::Ranged::new_max(crate::formula::SOBER)),
            max_ap: 1,
            will: 3,
        };
        let challenge = Default::default();
        let mut world = World::default();
        world.initialise(&mut rng, 0, 64, 32, player_info, challenge);
        // clear out the world
        for x in 0..16 {
            for y in 0..16 {
                let pos = Point::new(x, y);
                world.remove_monster(pos);
                if let Some(cell) = world.cell_mut(pos) {
                    cell.tile.kind = Empty;
                }
                assert_eq!(world.monster_on_pos(pos), None);
                assert!(world.walkable(pos, Blocker::WALL, player_info.pos));
            }
        }

        for line in lines {
            for c in line.chars() {
                if c == 's' {
                    start = Point {
                        x: x as i32,
                        y: y as i32,
                    };
                }

                if c == 'd' {
                    destination = Point {
                        x: x as i32,
                        y: y as i32,
                    };
                }

                let tile_kind = match c {
                    '.' => Empty,
                    '*' => Empty,
                    's' => Empty,
                    'd' => Empty,
                    'x' => Tree,
                    _ => unreachable!(),
                };
                let pos = Point {
                    x: x as i32,
                    y: y as i32,
                };
                if let Some(cell) = world.cell_mut(pos) {
                    cell.tile = Tile::new(tile_kind);
                }

                x += 1;
            }
            y += 1;
            x = 0;
        }

        assert_ne!(start, Point { x: -1, y: -1 });
        assert_ne!(destination, Point { x: -1, y: -1 });

        Board {
            start,
            destination,
            world,
        }
    }

    fn find_path(board: &Board, blockers: Blocker, calculation_limit: i32) -> Path {
        let player_position = Point::new(0, 0);
        let will = 2;
        let check_irresistible = false;
        Path::find(
            board.start,
            board.destination,
            &board.world,
            blockers,
            player_position,
            will,
            check_irresistible,
            calculation_limit,
            &super::direct_cost,
        )
    }

    #[test]
    fn test_neighbor() {
        let board = make_board(
            "
...........
.sd........
...........
...........
",
        );
        let path = find_path(&board, Blocker::WALL, 50);
        assert_eq!(1, path.len());
        let expected = [(2, 1)]
            .iter()
            .copied()
            .map(Into::into)
            .collect::<Vec<Point>>();
        assert_eq!(expected, path.collect::<Vec<_>>());
    }

    #[test]
    fn test_start_and_destination_identical() {
        let mut board = make_board(
            "
...........
.s.........
...........
...........
",
        );
        board.destination = board.start;
        let path = find_path(&board, Blocker::WALL, 50);
        assert_eq!(0, path.len());
        let expected: Vec<Point> = vec![];
        assert_eq!(expected, path.collect::<Vec<_>>());
    }

    #[test]
    fn test_straight_path() {
        let board = make_board(
            "
...........
.s******d..
...........
...........
",
        );
        let path = find_path(&board, Blocker::WALL, 50);
        assert_eq!(7, path.len());
        let expected = [(2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1), (8, 1)]
            .iter()
            .copied()
            .map(Into::into)
            .collect::<Vec<Point>>();
        assert_eq!(expected, path.collect::<Vec<_>>());
    }

    #[test]
    fn test_diagonal_path() {
        let board = make_board(
            "
s..........
.*.........
..*........
...d.......
",
        );
        let path = find_path(&board, Blocker::WALL, 50);
        assert_eq!(3, path.len());
        let expected = [(1, 1), (2, 2), (3, 3)]
            .iter()
            .copied()
            .map(Into::into)
            .collect::<Vec<Point>>();
        assert_eq!(expected, path.collect::<Vec<_>>());
    }

    #[test]
    fn test_no_path() {
        let board = make_board(
            "
xxxxx......
xs..x...d..
x...x......
xxxxx......
",
        );
        let path = find_path(&board, Blocker::WALL, 50);
        assert_eq!(0, path.len());
    }

    #[test]
    fn test_line_obstacle() {
        let board = make_board(
            "
....x......
.s..x......
..*.x......
...*****d..
",
        );
        let path = find_path(&board, Blocker::WALL, 50);
        assert_eq!(7, path.len());
        let expected = [(2, 2), (3, 3), (4, 3), (5, 3), (6, 3), (7, 3), (8, 3)]
            .iter()
            .copied()
            .map(Into::into)
            .collect::<Vec<Point>>();
        assert_eq!(expected, path.collect::<Vec<_>>());
    }

    #[test]
    fn test_concave_obstacle() {
        let board = make_board(
            "
......x....
......x....
......x....
......x....
......x....
.s....xd...
..*...x*...
..*xxxx*...
...****....
",
        );
        let path = find_path(&board, Blocker::WALL, 50);
        assert_eq!(9, path.len());
        let expected = [
            (2, 6),
            (2, 7),
            (3, 8),
            (4, 8),
            (5, 8),
            (6, 8),
            (7, 7),
            (7, 6),
            (7, 5),
        ]
        .iter()
        .copied()
        .map(Into::into)
        .collect::<Vec<Point>>();
        assert_eq!(expected, path.collect::<Vec<_>>());
    }
}
