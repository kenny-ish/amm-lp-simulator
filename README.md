# amm-lp-simulator

Monte Carlo simulation of a constant-product LP position compared with holding the two tokens.
Volatility, the pool fee and arbitrage against the pool all change the result, and this lets you
vary them.

```bash
cargo run --release -- --vol 0.8 --fee-bps 30 --days 90 --steps-per-day 24 --paths 2000
```

Each path works like this:

1. The external price follows geometric Brownian motion with zero drift (`--vol` is annualized).
2. After every step an arbitrageur trades the pool back to the edge of the no-arbitrage band. The pool
   price can differ from the external price by up to the fee before an arbitrage trade pays off.
3. The fee from each arbitrage trade stays in the pool, as in Uniswap v2.

The output is LP value divided by HODL value (mean and percentiles), how often the LP ended up ahead,
and the same numbers for a zero-fee pool, which is pure impermanent loss.

The arbitrage step has a closed form. With `g = 1 - fee` and the fee kept in the pool, buying until
the marginal price including the fee equals the external price `S` means solving
`(y + dy) * (y + g * dy) = k * S * g` for `dy`, which is a quadratic. Selling is the mirror case.

Random numbers come from a small xorshift64* generator with Box-Muller, seeded by `--seed`, so runs
are reproducible without any crates.

```bash
cargo test
```

One test checks that a zero-fee pool gives exactly the textbook impermanent loss formula
`2*sqrt(r)/(1+r) - 1`.
