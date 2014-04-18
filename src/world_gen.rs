use rand::Rng;
use rand::distributions::{Weighted, WeightedChoice, IndependentSample};

#[deriving(Clone, Rand, Eq)]
pub enum WorldItem {
    Empty,
    Tree,

    Dose,
    StrongDose,
    Food,

    Anxiety,
    Depression,
    Hunger,
    Voices,
    Shadows,
}

pub fn forrest<T: Rng>(rng: &mut T, w: int, h: int) -> Vec<(int, int, WorldItem)> {
    let monster_count = 5;
    let monster_weight = 30 / monster_count;
    let opts = WeightedChoice::new(vec! [
        Weighted{weight: 595, item: Empty},
        Weighted{weight: 390, item: Tree},
        Weighted{weight: 7,  item: Dose},
        Weighted{weight: 3,  item: StrongDose},
        Weighted{weight: 5,  item: Food},
        Weighted{weight: monster_weight,  item: Anxiety},
        Weighted{weight: monster_weight,  item: Depression},
        Weighted{weight: monster_weight,  item: Hunger},
        Weighted{weight: monster_weight,  item: Voices},
        Weighted{weight: monster_weight,  item: Shadows},
    ]);
    let mut result: Vec<(int, int, WorldItem)> = Vec::new();
    for x in range(0, w) {
        for y in range(0, h) {
            result.push((x, y, opts.ind_sample(rng)));
        }
    }
    result
}
