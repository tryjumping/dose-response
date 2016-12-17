use point::Point;
use level::Level;

pub struct Path;

impl Path {
    pub fn find(_from: Point, _to: Point, _level: &Level) -> Path {
        unimplemented!()
    }

    pub fn find_test(_from: Point, _to: Point, _walkability_map: &Vec<bool>) -> Path {
        unimplemented!()
    }

    /// The number of steps to necessary to reach the destination. If
    /// no path was found, it is `0`.
    pub fn len(&self) -> usize {
        unimplemented!()
    }
}

impl Iterator for Path {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
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
                result.start == Point { x: x as i32, y: y as i32 };
            }
            if piece == Destination {
                assert_eq!(Point{x: -1, y: -1}, result.start);
                result.destination == Point { x: x as i32, y: y as i32 };
            }
            x += 1;
        }
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
