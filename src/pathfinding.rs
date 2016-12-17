use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::usize;

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

    pub fn find_test(from: Point, to: Point, walkability_map: &Vec<bool>) -> Self {
        let neighbors = |_current| -> Vec<Point> {
            unimplemented!()
        };

        let cost = |_current, _next| -> usize {
            unimplemented!()
        };

        let heuristic = |_destination, _next| -> usize {
            unimplemented!()
        };

        let mut frontier = BinaryHeap::new();
        frontier.push(State { position: from, cost: 0 });
        let mut path = vec![from];
        let mut cost_so_far = HashMap::new();

        cost_so_far.insert(from, 0);

        while let Some(current) = frontier.pop() {
            if current.position == to {
                break
            }

            for &next in neighbors(current).iter() {
                let new_cost = cost_so_far[&current.position] + cost(current, next);
                let val = cost_so_far.entry(next).or_insert(usize::MAX);
                if new_cost < *val {
                    *val = new_cost;
                    let priority = new_cost + heuristic(to, next);
                    frontier.push(State { position: next, cost: priority });
                    path.push(next);
                }
            }
        }

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

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: usize,
    position: Point,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
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
        Path::find_test(board.start, board.destination, &world)
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
        println!("{:?}", path);
        assert_eq!(7, path.len());
        //assert_eq!(board.path, path.collect::<Vec<_>>());
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
        //assert_eq!(board.path, path.collect::<Vec<_>>());
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
        //assert_eq!(board.path, path.collect::<Vec<_>>());
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
        //assert_eq!(board.path, path.collect::<Vec<_>>());
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
        //assert_eq!(board.path, path.collect::<Vec<_>>());
    }
}
