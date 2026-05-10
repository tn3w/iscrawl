# iscrawl

[![Crates.io](https://img.shields.io/crates/v/iscrawl.svg)](https://crates.io/crates/iscrawl)
[![Docs.rs](https://docs.rs/iscrawl/badge.svg)](https://docs.rs/iscrawl)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Fast crawler/bot detection from User-Agent strings.

```
sub-140ns cold, 5ns warm
heuristic bool API
optional Crawlerdex info lookup
```

## Install

```toml
[dependencies]
iscrawl = "1.1"
```

Database metadata:

```toml
[dependencies]
iscrawl = { version = "1.1", features = ["database"] }
```

## Use

```rust
use iscrawl::is_crawler;

assert!(is_crawler("Googlebot/2.1 (+http://www.google.com/bot.html)"));
assert!(!is_crawler(
    "Mozilla/5.0 (X11; Linux x86_64; rv:115.0) Gecko/20100101 Firefox/115.0"
));
```

Default build has no deps. Use `crawler_info` with the `database` feature.

## Database

`crawler_info(user_agent)` returns `Option<&'static CrawlerInfo>` from the
bundled Crawlerdex DB. Matching is separate from `is_crawler`.

```rust
use iscrawl::crawler_info;

assert_eq!(
    crawler_info("Googlebot/2.1").unwrap().description,
    "Google's main web crawling bot for search indexing"
);
```

Update DB:

```bash
curl -fsSL https://github.com/tn3w/Crawlerdex/releases/latest/download/crawlers.min.json \
  -o crawlers.min.json
```

## Why fast

- `is_crawler`: stack buffer of 512 bytes, no heap.
- First-byte lookup table prunes 99% of needle scans.
- Single pass over the lowered input.
- Thread-local 256-slot direct-mapped cache keyed by pointer/length with edge-word guards.
- `crawler_info`: Aho-Corasick literals + chunked regex fallback.
- `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`.

Benchmarked on x86_64: cold **~140 ns/call**, warm cache hit **~5 ns/call**.

## How it decides

1. Empty input counts as crawler.
2. Input over 512 bytes is rejected (returns `false`).
3. If any crawler keyword (`bot`, `crawl`, `spider`, `scanner`, `+http`, `@`, `archive`, ...) appears: crawler.
4. If the UA does not start with `Mozilla/` or `Opera/` and has no known browser engine token (`gecko`, `webkit`, `chrome`, `firefox`, `msie`, `edge`, `opera`, ...): crawler.
5. If the UA starts with `Mozilla/` or `Opera/` but is missing both an engine token and the `(compatible;` marker: crawler.
6. Otherwise: browser.

`is_crawler` is heuristic. `crawler_info` is feature-gated + database-backed.

## Accuracy

Measured against bundled fixture corpora:

| corpus                       | size   | result             |
| ---------------------------- | ------ | ------------------ |
| crawler_user_agents.txt      | 2,149  | 95.4% detected     |
| loadkpi_crawlers.txt         | 3,696  | 94.7% detected     |
| crawler_user_agents_pgts.txt | 156    | 98.1% detected     |
| browser_user_agents.txt      | 19,897 | <1% false positive |

Run `cargo test --release` to verify on your machine.

## Bench

```bash
cargo bench --bench bench
cargo bench --features database --bench database
```

| run         | ns/call | M calls/s |
| ----------- | ------: | --------: |
| cold corpus |   137.1 |      7.30 |
| warm hits   |     4.5 |    222.26 |

Fixture corpus: 25,898 User-Agents.

## Develop

```bash
cargo build --release
cargo test --release
cargo test --release --features database
cargo test --doc
cargo doc --no-deps --open
```

## Format

Standard formatting across Rust + markdown/yaml.

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
npx --yes prettier --write --single-quote --print-width=100 --trailing-comma=es5 --end-of-line=lf "**/*.{md,yml}"
```

CI enforces `cargo fmt --check` and `cargo clippy -D warnings` on every push.

## Publish

Pushes to `main`/`master` trigger [`.github/workflows/publish.yml`](.github/workflows/publish.yml):

1. Reads `name` + `version` from `Cargo.toml`.
2. Skips if that version is already on crates.io.
3. Tests on Ubuntu, macOS, Windows × stable, beta.
4. Runs `cargo publish` using `CARGO_REGISTRY_TOKEN`.
5. Tags the commit `vX.Y.Z`.

Bump `version` in `Cargo.toml`, push to `main`, done.

## Funding

If this saved you time, [buy me a coffee](https://www.buymeacoffee.com/tn3w).

## License

Apache-2.0. See [LICENSE](LICENSE).
