use amm_lp_simulator::{run_path, Rng};

fn arg<T: std::str::FromStr>(args: &[String], name: &str, default: T) -> T {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn pct(sorted: &[f64], p: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let vol: f64 = arg(&args, "--vol", 0.8);
    let fee_bps: f64 = arg(&args, "--fee-bps", 30.0);
    let days: usize = arg(&args, "--days", 90);
    let spd: usize = arg(&args, "--steps-per-day", 24);
    let paths: usize = arg(&args, "--paths", 1000);
    let seed: u64 = arg(&args, "--seed", 7);

    let mut rng = Rng::new(seed);
    let dt = 1.0 / (365.0 * spd as f64);
    let mut with_fee = Vec::with_capacity(paths);
    let mut no_fee = Vec::with_capacity(paths);
    for _ in 0..paths {
        let (a, b) = run_path(&mut rng, vol, fee_bps / 1e4, days * spd, dt);
        with_fee.push(a);
        no_fee.push(b);
    }

    println!("vol {:.0}%  fee {fee_bps}bps  {days} days  {spd} steps/day  {paths} paths", vol * 100.0);
    for (label, v) in [("with fees", &mut with_fee), ("no fees  ", &mut no_fee)] {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mean = v.iter().sum::<f64>() / v.len() as f64;
        let wins = v.iter().filter(|&&r| r > 1.0).count() as f64 / v.len() as f64;
        println!(
            "{label}  LP/HODL mean {:+.2}%  p5 {:+.2}%  p50 {:+.2}%  p95 {:+.2}%  LP wins {:.0}%",
            (mean - 1.0) * 100.0,
            (pct(v, 0.05) - 1.0) * 100.0,
            (pct(v, 0.5) - 1.0) * 100.0,
            (pct(v, 0.95) - 1.0) * 100.0,
            wins * 100.0
        );
    }
}
