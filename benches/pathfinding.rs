#[macro_use]
extern crate bencher;

use dose_response::{
    player::{Mind, PlayerInfo},
    point::Point,
    random::Random,
    state::Challenge,
    world::World,
    WORLD_SIZE,
};

use bencher::Bencher;

fn setup() -> World {
    let seed = 42;
    let mut rng = Random::from_seed(seed as u64);
    let player_info = PlayerInfo {
        pos: Point::new(0, 0),
        mind: Mind::default(),
        max_ap: 1,
        will: 3,
    };
    let mut world = World::default();
    let challenge = Challenge::default();
    world.initialise(&mut rng, seed, WORLD_SIZE.x, 32, player_info, challenge);
    world
}

// TODO: actually, maybe we don't care about nearest dose because it's not used for monsters
// and instead, not check irresistible in World::tile_contents for monsters.
// It's a thing we'd still like to speed up, but maybe it's not such a big issue with game speed if it's just player doing it

fn a(bench: &mut Bencher) {
    let world = setup();
    bench.iter(|| world.nearest_dose(Point::new(0, 0), 20))
}

fn b(bench: &mut Bencher) {
    let world = setup();
    bench.iter(|| world.nearest_dose(Point::new(0, 0), 40))
}

benchmark_group!(benches, a, b);
benchmark_main!(benches);
