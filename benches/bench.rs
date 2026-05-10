use iscrawl::is_crawler;
use std::hint::black_box;
use std::time::{Duration, Instant};

const CRAWLER_UAS: &str = include_str!("../tests/fixtures/crawler_user_agents.txt");
const LOADKPI_CRAWLERS: &str = include_str!("../tests/fixtures/loadkpi_crawlers.txt");
const PGTS_CRAWLERS: &str = include_str!("../tests/fixtures/crawler_user_agents_pgts.txt");
const BROWSER_UAS: &str = include_str!("../tests/fixtures/browser_user_agents.txt");

const RUNS: usize = 15;
const COLD_PASSES: usize = 16;
const HOT_REPEATS: usize = 64;

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

    let total = corpus.len();
    assert!(total > 256, "fixture corpus should overflow the cache");

    let orders = random_orders(total);
    let cold = summarize(
        orders
            .iter()
            .map(|order| bench_cold_passes(&corpus, order))
            .collect(),
    );
    let hot = summarize(
        orders
            .iter()
            .map(|order| bench_hot_hits(&corpus, order))
            .collect(),
    );

    println!("fixtures: {total} total ({crawler_count} crawler, {browser_count} browser)");
    println!(
        "runs: {RUNS}, random order, cold passes/run: {COLD_PASSES}, warm repeats/ua/run: {HOT_REPEATS}"
    );
    println!(
        "cold corpus: {:>6.1} ns/call median, {:>6.1} best, {:>6.1} mean, {:>7.2} M calls/s, {} true/run",
        cold.median_ns,
        cold.best_ns,
        cold.mean_ns,
        calls_per_second(cold.median_ns),
        cold.true_count
    );
    println!(
        "warm hits:   {:>6.1} ns/call median, {:>6.1} best, {:>6.1} mean, {:>7.2} M calls/s, {} true/run",
        hot.median_ns,
        hot.best_ns,
        hot.mean_ns,
        calls_per_second(hot.median_ns),
        hot.true_count
    );
}

fn bench_cold_passes(corpus: &[&str], order: &[usize]) -> Run {
    let mut true_count = 0usize;
    let start = Instant::now();

    for _ in 0..COLD_PASSES {
        for &index in order {
            true_count += black_box(is_crawler(black_box(corpus[index]))) as usize;
        }
    }

    Run {
        elapsed: start.elapsed(),
        calls: COLD_PASSES * corpus.len(),
        true_count,
    }
}

fn bench_hot_hits(corpus: &[&str], order: &[usize]) -> Run {
    let mut true_count = 0usize;
    let mut calls = 0usize;
    let start = Instant::now();

    for &index in order {
        let ua = corpus[index];
        black_box(is_crawler(black_box(ua)));
        for _ in 0..HOT_REPEATS {
            true_count += black_box(is_crawler(black_box(ua))) as usize;
        }
        calls += HOT_REPEATS;
    }

    Run {
        elapsed: start.elapsed(),
        calls,
        true_count,
    }
}

fn random_orders(len: usize) -> Vec<Vec<usize>> {
    (0..RUNS)
        .map(|run| {
            let mut order: Vec<_> = (0..len).collect();
            shuffle(&mut order, 0x9e37_79b9_7f4a_7c15 ^ run as u64);
            order
        })
        .collect()
}

fn shuffle(values: &mut [usize], mut state: u64) {
    for i in (1..values.len()).rev() {
        state = splitmix64(state);
        values.swap(i, state as usize % (i + 1));
    }
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

struct Run {
    elapsed: Duration,
    calls: usize,
    true_count: usize,
}

struct Summary {
    median_ns: f64,
    best_ns: f64,
    mean_ns: f64,
    true_count: usize,
}

fn summarize(runs: Vec<Run>) -> Summary {
    let true_count = runs[0].true_count;
    assert!(
        runs.iter().all(|run| run.true_count == true_count),
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
        true_count,
    }
}

fn ns_per_call(elapsed: Duration, calls: usize) -> f64 {
    elapsed.as_nanos() as f64 / calls as f64
}

fn calls_per_second(ns_per_call: f64) -> f64 {
    1_000.0 / ns_per_call
}
