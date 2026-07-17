//! This module tests the performance of the GDScript formatter. Use this to quickly test the
//! performance impact of changes to the formatter locally.
//!
//! Run cargo run --bin benchmark --release to compile and run the benchmark.
//! You can use it in a shell script to compare performance between two git revisions.
//! To profile the CPU usage of the benchmark, run:
//!
//! ```sh
//! cargo build --profile profiling --bin benchmark
//! samply record target/profiling/benchmark --profile-long
//! ```
//!
//! For example, to compare between this commit and the previous one:
//!
//! ```sh
//! cargo run --bin benchmark --release > benchmark_results.txt
//! echo "On previous commit:\n" >> benchmark_results.txt
//! git checkout HEAD^
//! cargo run --bin benchmark --release >> benchmark_results.txt
//! git checkout -
//! ```
use gdscript_formatter::{FormatterConfiguration, RenderElement, format_gdscript_with_buffers};
use std::{
    env, fs,
    hint::black_box,
    time::{Duration, Instant},
};

const PROFILE_ITERATIONS: usize = 5_000;
const WARMUP_DURATION: Duration = Duration::from_millis(200);
const SAMPLE_DURATION: Duration = Duration::from_millis(200);
const SAMPLE_COUNT: usize = 10;

/// A struct that stores data and has functions to benchmark the GDScript
/// formatter and measure performance.
struct BenchmarkRunner {
    render_elements: Vec<RenderElement>,
    output: String,
}

struct BenchmarkMeasurement {
    median_seconds_per_iteration: f64,
    total_iterations: usize,
}

impl BenchmarkRunner {
    fn new() -> Self {
        Self {
            render_elements: Vec::new(),
            output: String::new(),
        }
    }

    fn format(&mut self, source: &str, config: &FormatterConfiguration) -> Result<(), String> {
        format_gdscript_with_buffers(
            black_box(source),
            black_box(config),
            &mut self.render_elements,
            &mut self.output,
        )?;
        black_box(&self.output);
        Ok(())
    }

    fn measure(
        &mut self,
        source: &str,
        config: &FormatterConfiguration,
    ) -> Result<BenchmarkMeasurement, String> {
        let warmup_start = Instant::now();
        while warmup_start.elapsed() < WARMUP_DURATION {
            self.format(source, config)?;
        }

        let mut seconds_per_iteration = Vec::with_capacity(SAMPLE_COUNT);
        let mut total_iterations = 0;
        let mut sample_index = 0;
        while sample_index < SAMPLE_COUNT {
            let sample_start = Instant::now();
            let mut sample_iterations = 0;
            while sample_start.elapsed() < SAMPLE_DURATION {
                self.format(source, config)?;
                sample_iterations += 1;
            }
            let sample_seconds = sample_start.elapsed().as_secs_f64();
            seconds_per_iteration.push(sample_seconds / sample_iterations as f64);
            total_iterations += sample_iterations;
            sample_index += 1;
        }

        let mut current_index = 1;
        while current_index < seconds_per_iteration.len() {
            let mut sorted_index = current_index;
            while sorted_index > 0
                && seconds_per_iteration[sorted_index] < seconds_per_iteration[sorted_index - 1]
            {
                seconds_per_iteration.swap(sorted_index, sorted_index - 1);
                sorted_index -= 1;
            }
            current_index += 1;
        }

        Ok(BenchmarkMeasurement {
            median_seconds_per_iteration: seconds_per_iteration[SAMPLE_COUNT / 2],
            total_iterations,
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let short_content = fs::read_to_string("benchmarks/gdscript_files/short.gd")?;
    let long_content = fs::read_to_string("benchmarks/gdscript_files/long.gd")?;
    let config = FormatterConfiguration::default();

    let mut arguments = env::args();
    arguments.next();
    if let Some(argument) = arguments.next() {
        if argument != "--profile-long" {
            return Err(format!("Unknown benchmark argument: {argument}").into());
        }
        let iterations = if let Some(value) = arguments.next() {
            value.parse::<usize>()?
        } else {
            PROFILE_ITERATIONS
        };
        if arguments.next().is_some() {
            return Err("Usage: benchmark --profile-long [iterations]".into());
        }

        let mut runner = BenchmarkRunner::new();
        for _ in 0..10 {
            runner.format(&long_content, &config)?;
        }
        println!("Profiling long GDScript file ({iterations} iterations)...");
        for _ in 0..iterations {
            runner.format(&long_content, &config)?;
        }
        return Ok(());
    }

    println!("Running GDScript Formatter Benchmark...");

    let safe_config = FormatterConfiguration {
        safe: true,
        ..config.clone()
    };
    let mut runner = BenchmarkRunner::new();
    println!("Benchmarking short file...");
    let short = runner.measure(&short_content, &config)?;
    println!("Benchmarking short file with safe mode...");
    let short_safe = runner.measure(&short_content, &safe_config)?;
    println!("Benchmarking long file...");
    let long = runner.measure(&long_content, &config)?;
    println!("Benchmarking long file with safe mode...");
    let long_safe = runner.measure(&long_content, &safe_config)?;

    let short_slowdown =
        (short_safe.median_seconds_per_iteration / short.median_seconds_per_iteration - 1.0)
            * 100.0;
    let long_slowdown =
        (long_safe.median_seconds_per_iteration / long.median_seconds_per_iteration - 1.0) * 100.0;

    println!("\nBenchmark Results:");
    println!("=================");
    println!(
        "Short file ({} iterations): {:.3}ms median per iteration",
        short.total_iterations,
        short.median_seconds_per_iteration * 1000.0
    );
    println!(
        "Long file ({} iterations):   {:.3}ms median per iteration",
        long.total_iterations,
        long.median_seconds_per_iteration * 1000.0
    );
    println!(
        "Short file with safe mode ({} iterations): {:.3}ms median per iteration, {:.1}% slower",
        short_safe.total_iterations,
        short_safe.median_seconds_per_iteration * 1000.0,
        short_slowdown
    );
    println!(
        "Long file with safe mode ({} iterations):   {:.3}ms median per iteration, {:.1}% slower",
        long_safe.total_iterations,
        long_safe.median_seconds_per_iteration * 1000.0,
        long_slowdown
    );

    Ok(())
}
