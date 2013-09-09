use std::uint::range;
use std::rand::{RngUtil, Weighted};

#[deriving(Rand)]
pub enum WorldItem {
    Empty,
    Tree,
    Dose,
    StrongDose,
    Monster,
}

pub fn forrest<T: RngUtil>(rng: &mut T, w: uint, h: uint) -> ~[(int, int, WorldItem)] {
    let opts = [
        Weighted{weight: 600, item: Empty},
        Weighted{weight: 390, item: Tree},
        Weighted{weight: 7,  item: Dose},
        Weighted{weight: 3,  item: StrongDose},
    ];
    let mut result: ~[(int, int, WorldItem)] = ~[];
    for range(0, w) |x| {
        for range(0, h) |y| {
            result.push((x as int, y as int,
                       rng.choose_weighted(opts)));
        }
    }
    result
}
