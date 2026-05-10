#[cfg(feature = "database")]
use iscrawl::crawler_info;
#[cfg(feature = "database")]
use std::hint::black_box;
#[cfg(feature = "database")]
use std::time::{Duration, Instant};

#[cfg(feature = "database")]
const CRAWLER_UAS: &str = include_str!("../tests/fixtures/crawler_user_agents.txt");
#[cfg(feature = "database")]
const LOADKPI_CRAWLERS: &str = include_str!("../tests/fixtures/loadkpi_crawlers.txt");
#[cfg(feature = "database")]
const PGTS_CRAWLERS: &str = include_str!("../tests/fixtures/crawler_user_agents_pgts.txt");
#[cfg(feature = "database")]
const BROWSER_UAS: &str = include_str!("../tests/fixtures/browser_user_agents.txt");

#[cfg(feature = "database")]
const RUNS: usize = 10;
#[cfg(feature = "database")]
const PASSES: usize = 4;

#[cfg(feature = "database")]
fn main() {
    let crawler_count = CRAWLER_UAS.lines().count()
        + LOADKPI_CRAWLERS.lines().count()
        + PGTS_CRAWLERS.lines().count();
    let browser_count = BROWSER_UAS.lines().count();

    let mut corpus = Vec::with_capacity(crawler_count + browser_count);
    corpus.extend(CRAWLER_UAS.lines());
    corpus.extend(LOADKPI_CRAWLERS.lines());
    corpus.extend(PGTS_CRAWLERS.lines());
    corpus.extend(BROWSER_UAS.lines());

    let orders = random_orders(corpus.len());
    let summary = summarize(
        orders
            .iter()
            .map(|order| bench_database(&corpus, order))
            .collect(),
    );

    println!(
        "fixtures: {} total ({crawler_count} crawler, {browser_count} browser)",
        corpus.len()
    );
    println!("runs: {RUNS}, random order, passes/run: {PASSES}");
    println!(
        "database: {:>6.1} ns/call median, {:>6.1} best, \
         {:>6.1} mean, {:>7.2} M calls/s, {} matches/run",
        summary.median_ns,
        summary.best_ns,
        summary.mean_ns,
        calls_per_second(summary.median_ns),
        summary.match_count
    );
}

#[cfg(not(feature = "database"))]
fn main() {
    eprintln!("run with: cargo bench --features database --bench database");
}

#[cfg(feature = "database")]
fn bench_database(corpus: &[&str], order: &[usize]) -> Run {
    let mut match_count = 0usize;
    let start = Instant::now();

    for _ in 0..PASSES {
        for &index in order {
            let matched = crawler_info(black_box(corpus[index])).is_some();
            match_count += black_box(matched) as usize;
        }
    }

    Run {
        elapsed: start.elapsed(),
        calls: PASSES * corpus.len(),
        match_count,
    }
}

#[cfg(feature = "database")]
fn random_orders(len: usize) -> Vec<Vec<usize>> {
    (0..RUNS)
        .map(|run| {
            let mut order: Vec<_> = (0..len).collect();
            shuffle(&mut order, 0x9e37_79b9_7f4a_7c15 ^ run as u64);
            order
        })
        .collect()
}

#[cfg(feature = "database")]
fn shuffle(values: &mut [usize], mut state: u64) {
    for i in (1..values.len()).rev() {
        state = splitmix64(state);
        values.swap(i, state as usize % (i + 1));
    }
}

#[cfg(feature = "database")]
fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(feature = "database")]
struct Run {
    elapsed: Duration,
    calls: usize,
    match_count: usize,
}

#[cfg(feature = "database")]
struct Summary {
    median_ns: f64,
    best_ns: f64,
    mean_ns: f64,
    match_count: usize,
}

#[cfg(feature = "database")]
fn summarize(runs: Vec<Run>) -> Summary {
    let match_count = runs[0].match_count;
    assert!(
        runs.iter().all(|run| run.match_count == match_count),
        "benchmark runs produced inconsistent results"
    );

    let mut samples: Vec<_> = runs
        .iter()
        .map(|run| ns_per_call(run.elapsed, run.calls))
        .collect();
    samples.sort_by(f64::total_cmp);

    Summary {
        median_ns: samples[samples.len() / 2],
        best_ns: samples[0],
        mean_ns: samples.iter().sum::<f64>() / samples.len() as f64,
        match_count,
    }
}

#[cfg(feature = "database")]
fn ns_per_call(elapsed: Duration, calls: usize) -> f64 {
    elapsed.as_nanos() as f64 / calls as f64
}

#[cfg(feature = "database")]
fn calls_per_second(ns_per_call: f64) -> f64 {
    1_000.0 / ns_per_call
}
