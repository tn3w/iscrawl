# iscrawl

[![Crates.io](https://img.shields.io/crates/v/iscrawl.svg)](https://crates.io/crates/iscrawl)
[![Docs.rs](https://docs.rs/iscrawl/badge.svg)](https://docs.rs/iscrawl)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Fast crawler/bot detection from User-Agent strings. ~155 ns cold, ~5 ns warm.

## Install

```toml
[dependencies]
iscrawl = "1.2"
```

With Crawlerdex metadata lookup:

```toml
[dependencies]
iscrawl = { version = "1.2", features = ["database"] }
```

Default build has no dependencies.

## Use

```rust
use iscrawl::is_crawler;

assert!(is_crawler("Googlebot/2.1 (+http://www.google.com/bot.html)"));
assert!(!is_crawler(
    "Mozilla/5.0 (X11; Linux x86_64; rv:115.0) Gecko/20100101 Firefox/115.0"
));
```

## Heuristic

1. Empty input: crawler.
2. Over 512 bytes: browser (not classified).
3. Crawler keyword (`http`, `@`, `bot`, `crawl`, `spider`, `checker`, `feed`, `fetch`,
   `monitor`, `ptst`, `preview`): crawler.
4. No browser engine token (`gecko`, `webkit`, `msie`, `trident`, `opera`, `konqueror`,
   `links`, `icab`, `netfront`, `lynx`, `mosaic`, `netsurf`): crawler.
5. Otherwise: browser.

Matching is ASCII case-insensitive.

## Database

`crawler_info(user_agent)` returns `Option<&'static CrawlerInfo>` from the bundled
Crawlerdex database. Independent of `is_crawler`.

```rust
use iscrawl::crawler_info;

let info = crawler_info("Googlebot/2.1").unwrap();
assert!(info.tags.iter().any(|tag| tag == "search-engine"));
```

Update the database:

```bash
curl -fsSL https://github.com/tn3w/Crawlerdex/releases/latest/download/crawlers.min.json \
  -o crawlers.min.json
```

## Why fast

- Lowercase copy into a 512-byte stack buffer, no heap.
- One pass: a 64 KiB table of needle-start byte pairs skips almost every position.
- Thread-local 256-slot cache keyed by pointer and length, guarded by first/last 8 bytes.
- `crawler_info`: one Aho-Corasick DFA over literal patterns and the required literal
  (prefix or suffix) of each regex; a regex runs only when its literal appears.
  Costs ~13 MB and ~25 ms on first use.
- `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`.

## Accuracy

Bundled fixture corpora (`tests/fixtures`):

| corpus                       |   size | result             |
| ---------------------------- | -----: | ------------------ |
| crawler_user_agents.txt      |  2,149 | 96.2% detected     |
| loadkpi_crawlers.txt         |  3,696 | 95.7% detected     |
| crawler_user_agents_pgts.txt |    156 | 98.7% detected     |
| browser_user_agents.txt      | 19,897 | 0.12% false positive |

Misses are mostly headless browsers and monitors that spoof a full browser UA; use
`crawler_info` for those.

## Bench

```bash
cargo bench                      # heuristic
cargo bench --features database  # plus database lookup
```

| run         | ns/call |
| ----------- | ------: |
| cold corpus |     156 |
| warm hits   |       5 |
| database    |     310 |

25,898 fixture User-Agents, x86_64.

## Develop

```bash
cargo test --release --all-features
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

`rustfmt.toml` sets `max_width = 90`. CI enforces fmt and clippy.

## Publish

Pushes to `main`/`master` run [publish.yml](.github/workflows/publish.yml): skip if the
`Cargo.toml` version is already on crates.io, else test (Ubuntu/macOS/Windows × stable/beta),
`cargo publish`, tag `vX.Y.Z`. Bump `version` and push.

## Funding

[Buy me a coffee](https://www.buymeacoffee.com/tn3w).

## License

Apache-2.0. See [LICENSE](LICENSE).
