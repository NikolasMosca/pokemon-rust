/// LCG deterministico: stesso seed → stesso risultato.
/// Parametri classici di Numerical Recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rng(u64);

impl Rng {
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Avanza il seed e restituisce un valore in [0, max).
    pub fn next(&mut self, max: u64) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 % max
    }

    /// Restituisce true con probabilità 1/chance (es. chance=16 → 6.25%).
    pub fn roll(&mut self, chance: u64) -> bool {
        self.next(chance) == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stesso_seed_stesso_risultato() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        assert_eq!(a.next(100), b.next(100));
    }

    #[test]
    fn seed_diversi_risultati_diversi() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        assert_ne!(a.next(u64::MAX), b.next(u64::MAX));
    }

    #[test]
    fn next_sempre_in_range() {
        let mut rng = Rng::new(999);
        for _ in 0..1000 {
            assert!(rng.next(16) < 16);
        }
    }

    #[test]
    fn roll_1_su_1_sempre_true() {
        let mut rng = Rng::new(0);
        assert!(rng.roll(1));
    }
}
