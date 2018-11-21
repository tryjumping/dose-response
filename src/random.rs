use rand::SeedableRng;
use rand_isaac::IsaacRng;

pub type Random = IsaacRng;

pub fn from_seed(seed: u64) -> Random {
    IsaacRng::seed_from_u64(seed)
}
