//! Constant-product pool with fee-aware arbitrage.

pub struct Pool {
    pub x: f64, // risky asset
    pub y: f64, // numeraire
    pub gamma: f64,
}

impl Pool {
    pub fn new(x: f64, y: f64, fee: f64) -> Self {
        Pool { x, y, gamma: 1.0 - fee }
    }

    pub fn price(&self) -> f64 {
        self.y / self.x
    }

    /// Trade the pool to the no-arbitrage boundary for external price `s`.
    ///
    /// The fee stays in the pool, so the post-trade reserves are (x', y') with
    /// x' = k / (y + g*dy), y' = y + dy, and the arb stops when y'/x' = s*g.
    /// That gives g*dy^2 + (1+g)*y*dy + y^2 - k*s*g = 0 (mirror image for sells).
    pub fn arbitrage(&mut self, s: f64) {
        let k = self.x * self.y;
        let g = self.gamma;
        if s > self.price() / g {
            // asset is cheap in the pool: arb pays dy, receives dx
            let dy = solve(g, self.y, k * s * g);
            self.x = k / (self.y + g * dy);
            self.y += dy;
        } else if s < self.price() * g {
            // asset is expensive in the pool: arb pays dx, receives dy
            let dx = solve(g, self.x, k * g / s);
            self.y = k / (self.x + g * dx);
            self.x += dx;
        }
    }

    pub fn value(&self, s: f64) -> f64 {
        self.x * s + self.y
    }
}

/// Positive root of g*d^2 + (1+g)*r*d + r^2 - target = 0.
fn solve(g: f64, r: f64, target: f64) -> f64 {
    let b = (1.0 + g) * r;
    let c = r * r - target;
    (-b + (b * b - 4.0 * g * c).sqrt()) / (2.0 * g)
}

/// xorshift64* PRNG
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.max(1))
    }

    pub fn next_f64(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let v = self.0.wrapping_mul(0x2545F4914F6CDD1D);
        (v >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Standard normal via Box-Muller.
    pub fn normal(&mut self) -> f64 {
        let u1 = self.next_f64().max(f64::MIN_POSITIVE);
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

/// Simulate one path, returning (lp / hodl with fee, lp / hodl without fee).
pub fn run_path(rng: &mut Rng, vol: f64, fee: f64, steps: usize, dt: f64) -> (f64, f64) {
    let s0 = 1000.0;
    let mut s = s0;
    let mut pool = Pool::new(1.0, s0, fee);
    let mut bare = Pool::new(1.0, s0, 0.0);
    for _ in 0..steps {
        s *= (-0.5 * vol * vol * dt + vol * dt.sqrt() * rng.normal()).exp();
        pool.arbitrage(s);
        bare.arbitrage(s);
    }
    let hodl = 1.0 * s + s0;
    (pool.value(s) / hodl, bare.value(s) / hodl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_fee_matches_il_formula() {
        for r in [0.25, 0.5, 2.0, 4.0] {
            let mut p = Pool::new(1.0, 100.0, 0.0);
            p.arbitrage(100.0 * r);
            let lp_over_hodl = p.value(100.0 * r) / (100.0 * r + 100.0);
            let il = 2.0 * f64::sqrt(r) / (1.0 + r);
            assert!((lp_over_hodl - il).abs() < 1e-12, "r={r}");
        }
    }

    #[test]
    fn arbitrage_lands_on_band_edge() {
        let mut p = Pool::new(10.0, 10_000.0, 0.003);
        p.arbitrage(1200.0);
        assert!((p.price() / p.gamma - 1200.0).abs() < 1e-6);
        p.arbitrage(900.0);
        assert!((p.price() * p.gamma - 900.0).abs() < 1e-6);
    }

    #[test]
    fn small_move_inside_band_is_ignored() {
        let mut p = Pool::new(10.0, 10_000.0, 0.003);
        p.arbitrage(1001.0);
        assert_eq!(p.price(), 1000.0);
    }

    #[test]
    fn fees_grow_k() {
        let mut p = Pool::new(10.0, 10_000.0, 0.003);
        let k0 = p.x * p.y;
        p.arbitrage(1500.0);
        p.arbitrage(800.0);
        assert!(p.x * p.y > k0);
    }

    #[test]
    fn rng_is_deterministic_and_in_range() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            let v = a.next_f64();
            assert!((0.0..1.0).contains(&v));
            assert_eq!(v, b.next_f64());
        }
    }
}
