use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::f32;

use point::Point;
use level::Level;

#[derive(Debug)]
pub struct Path {
    path: Vec<Point>,
}

impl Path {
    pub fn find(_from: Point, _to: Point, _level: &Level) -> Self {
        unimplemented!()
    }

    pub fn find_test(from: Point, to: Point, walkability_map: &Vec<bool>, map_width: usize) -> Self {
        let map_height = walkability_map.len() / map_width;
        assert_eq!(map_width * map_height, walkability_map.len());
        let neighbors = |current: Point| {
            assert!(current.x >= 0);
            assert!(current.y >= 0);
            assert!(current.x < map_width as i32);
            assert!(current.y < map_height as i32);
            let dp: [Point; 9] = [
                (-1, -1).into(),
                (-1,  0).into(),
                (-1,  1).into(),
                ( 0, -1).into(),
                ( 0,  0).into(),
                ( 0,  1).into(),
                ( 1, -1).into(),
                ( 1,  0).into(),
                ( 1,  1).into(),
            ];
            dp.clone()
                .iter()
                .map(|&d| current + d)
                .filter(|&point| {
                    point >= (0, 0) &&
                        point < (map_width as i32, map_height as i32) &&
                        walkability_map[point.y as usize * map_width + point.x as usize]
                })
                .collect::<Vec<_>>()
        };

        let cost = |current: Point, next: Point| -> f32 {
            assert!((current.x - next.x).abs() <= 1);
            assert!((current.y - next.y).abs() <= 1);
            1.0
        };

        let heuristic = |destination: Point, next: Point| -> f32 {
            ((destination.x - next.x).abs() + (destination.y - next.y).abs()) as f32
        };

        let mut frontier = BinaryHeap::new();
        frontier.push(State { position: from, cost: 0.0 });
        let mut came_from = HashMap::new();
        let mut cost_so_far = HashMap::new();

        came_from.insert(from, None);
        cost_so_far.insert(from, 0.0);

        while let Some(current) = frontier.pop() {
            if current.position == to {
                break
            }

            for &next in neighbors(current.position).iter() {
                let new_cost = cost_so_far[&current.position] + cost(current.position, next);
                let val = cost_so_far.entry(next).or_insert(f32::MAX);
                if new_cost < *val {
                    *val = new_cost;
                    let priority = new_cost + heuristic(to, next);
                    frontier.push(State { position: next, cost: priority });
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
                    Some(&None) => panic!(
                        "Every point except for the initial one (`from`) one should be some."),
                    None => {
                        path_buffer = vec![];
                        break
                    },
                }
            }
            path_buffer
        };

        assert_eq!(None, path.iter().find(|&&p| p == from));

        Path {
            path: path,
        }
    }

    /// The number of steps to necessary to reach the destination. If
    /// no path was found, it is `0`.
    pub fn len(&self) -> usize {
        self.path.len()
    }
}

impl Iterator for Path {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.path.pop()
    }
}

#[derive(Copy, Clone, PartialEq)]
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
        } else if other.cost == self.cost {
            Ordering::Equal
        } else {
            unreachable!()
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
    use point::Point;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    enum Piece {
        Start,
        Empty,
        Blocked,
        Destination,
    }

    #[derive(Debug)]
    struct Board {
        start: Point,
        destination: Point,
        board: Vec<Piece>,
        width: usize,
    }

    fn make_board(text: &str) -> Board {
        use self::Piece::*;
        let mut result = Board {
            start: Point{x: -1, y: -1},
            destination: Point{x: -1, y: -1},
            width: 0,
            board: vec![]
        };
        let mut x = 0;
        let mut y = 0;
        for c in text.chars() {
            if c == '\n' {
                if result.width == 0 {
                    result.width = x;
                } else {
                    assert_eq!(result.width, x);
                }
                x = 0;
                y += 1;
                continue
            }
            let piece = match c {
                '.' => Empty,
                '*' => Empty,
                's' => Start,
                'd' => Destination,
                'x' => Blocked,
                _   => unreachable!(),
            };
            result.board.push(piece);
            if piece == Start {
                assert_eq!(Point{x: -1, y: -1}, result.start);
                result.start = Point { x: x as i32, y: y as i32 };
            }
            if piece == Destination {
                assert_eq!(Point{x: -1, y: -1}, result.destination);
                result.destination = Point { x: x as i32, y: y as i32 };
            }
            x += 1;
        }
        assert!(result.start != Point { x: -1, y: -1});
        assert!(result.destination != Point { x: -1, y: -1});
        result
    }

    fn find_path(board: &Board) -> Path {
        let world = board.board.iter().map(|&piece| piece != Piece::Blocked).collect();
        Path::find_test(board.start, board.destination, &world, board.width)
    }

    #[test]
    fn test_straight_path() {
        let board = make_board("\
...........
.s******d..
...........
...........
");
        let path: Path = find_path(&board);
        assert_eq!(7, path.len());
        let expected = [(2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1), (8, 1)].iter()
            .cloned().map(Into::into).collect::<Vec<Point>>();
        assert_eq!(expected, path.collect::<Vec<_>>());
    }

    #[test]
    fn test_diagonal_path() {
        let board = make_board("\
s..........
.*.........
..*........
...d.......
");
        let path: Path = find_path(&board);
        assert_eq!(3, path.len());
        let expected = [(1, 1), (2, 2), (3, 3)].iter()
            .cloned().map(Into::into).collect::<Vec<Point>>();
        assert_eq!(expected, path.collect::<Vec<_>>());
    }

    #[test]
    fn test_no_path() {
        let board = make_board("\
....x......
.s..x...d..
....x......
....x......
");
        let path: Path = find_path(&board);
        assert_eq!(0, path.len());
    }

    #[test]
    fn test_line_obstacle() {
        let board = make_board("\
....x......
.s..x......
..*.x......
...*****d..
");
        let path: Path = find_path(&board);
        assert_eq!(7, path.len());
        let expected = [(2, 2), (3, 3), (4, 3), (5, 3), (6, 3), (7, 3), (8, 3)].iter()
            .cloned().map(Into::into).collect::<Vec<Point>>();
        assert_eq!(expected, path.collect::<Vec<_>>());
    }

    #[test]
    fn test_concave_obstacle() {
        let board = make_board("\
......x....
.s....xd...
..*...x*...
..*xxxx*...
...****....
");
        let path: Path = find_path(&board);
        assert_eq!(9, path.len());
        let expected = [(2, 2), (2, 3), (3, 4), (4, 4), (5, 4), (6, 4), (7, 3), (7, 2), (7, 1)].iter()
            .cloned().map(Into::into).collect::<Vec<Point>>();
        assert_eq!(expected, path.collect::<Vec<_>>());
    }
}
