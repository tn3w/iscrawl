use iscrawl::is_crawler;
use std::hint::black_box;
use std::time::{Duration, Instant};

const FIXTURES: [&str; 3] = [
    include_str!("../tests/fixtures/crawler_user_agents.txt"),
    include_str!("../tests/fixtures/loadkpi_crawlers.txt"),
    include_str!("../tests/fixtures/crawler_user_agents_pgts.txt"),
];
const BROWSER_FIXTURE: &str = include_str!("../tests/fixtures/browser_user_agents.txt");

const RUNS: usize = 15;
const COLD_PASSES: usize = 16;
const WARM_REPEATS: usize = 64;

struct Run {
    elapsed: Duration,
    calls: usize,
    positives: usize,
}

struct Summary {
    median: f64,
    best: f64,
    mean: f64,
    positives: usize,
}

fn main() {
    let crawler_count: usize =
        FIXTURES.iter().map(|fixture| fixture.lines().count()).sum();
    let corpus: Vec<&str> = FIXTURES
        .iter()
        .chain([&BROWSER_FIXTURE])
        .flat_map(|fixture| fixture.lines())
        .collect();
    assert!(corpus.len() > 256, "corpus must overflow the cache");

    println!(
        "fixtures: {} total ({crawler_count} crawler), {RUNS} runs, random order",
        corpus.len()
    );
    report("cold corpus", &corpus, |corpus, order| {
        measure(corpus, order, COLD_PASSES, |user_agent| {
            is_crawler(user_agent)
        })
    });
    report("warm hits", &corpus, warm_run);
    report_database(&corpus);
}

#[cfg(feature = "database")]
fn report_database(corpus: &[&str]) {
    const DATABASE_PASSES: usize = 4;

    report("database", corpus, |corpus, order| {
        let lookup = |user_agent: &str| iscrawl::crawler_info(user_agent).is_some();
        measure(corpus, order, DATABASE_PASSES, lookup)
    });
}

#[cfg(not(feature = "database"))]
fn report_database(_corpus: &[&str]) {
    println!("database: run with --features database");
}

fn report(name: &str, corpus: &[&str], run: impl Fn(&[&str], &[usize]) -> Run) {
    let runs: Vec<Run> = (0..RUNS)
        .map(|seed| run(corpus, &shuffled_order(corpus.len(), seed as u64)))
        .collect();
    let summary = summarize(runs);

    println!(
        "{name:<12} {:>7.1} ns/call median, {:>7.1} best, {:>7.1} mean, \
         {:>7.2} M calls/s, {} positives/run",
        summary.median,
        summary.best,
        summary.mean,
        1_000.0 / summary.median,
        summary.positives
    );
}

fn measure(
    corpus: &[&str],
    order: &[usize],
    passes: usize,
    check: impl Fn(&str) -> bool,
) -> Run {
    let mut positives = 0;
    let start = Instant::now();

    for _ in 0..passes {
        for &index in order {
            positives += black_box(check(black_box(corpus[index]))) as usize;
        }
    }

    Run {
        elapsed: start.elapsed(),
        calls: passes * corpus.len(),
        positives,
    }
}

fn warm_run(corpus: &[&str], order: &[usize]) -> Run {
    let mut positives = 0;
    let start = Instant::now();

    for &index in order {
        let user_agent = black_box(corpus[index]);
        is_crawler(user_agent);
        for _ in 0..WARM_REPEATS {
            positives += black_box(is_crawler(user_agent)) as usize;
        }
    }

    Run {
        elapsed: start.elapsed(),
        calls: order.len() * WARM_REPEATS,
        positives,
    }
}

fn shuffled_order(length: usize, seed: u64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..length).collect();
    let mut state = 0x9e37_79b9_7f4a_7c15 ^ seed;

    for index in (1..length).rev() {
        state = splitmix64(state);
        order.swap(index, state as usize % (index + 1));
    }
    order
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn summarize(runs: Vec<Run>) -> Summary {
    let positives = runs[0].positives;
    assert!(
        runs.iter().all(|run| run.positives == positives),
        "benchmark runs produced inconsistent results"
    );

    let mut samples: Vec<f64> = runs
        .iter()
        .map(|run| run.elapsed.as_nanos() as f64 / run.calls as f64)
        .collect();
    samples.sort_by(f64::total_cmp);

    Summary {
        median: samples[samples.len() / 2],
        best: samples[0],
        mean: samples.iter().sum::<f64>() / samples.len() as f64,
        positives,
    }
}
