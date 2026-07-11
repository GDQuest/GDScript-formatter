//! This module tests the performance of the GDScript formatter. Use this to quickly test the
//! performance impact of changes to the formatter locally.
//!
//! Run cargo run --bin benchmark --release to compile and run the benchmark.
//! You can use it in a shell script to compare performance between two git revisions.
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
use std::{fs, time::Instant};

const ITERATIONS: u16 = 40;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let short_content = fs::read_to_string("benchmarks/gdscript_files/short.gd")?;
    let long_content = fs::read_to_string("benchmarks/gdscript_files/long.gd")?;
    let config = FormatterConfiguration::default();

    println!("Running GDScript Formatter Benchmark...");

    let mut render_elements: Vec<RenderElement> = Vec::new();
    let mut output = String::new();

    println!("Running short file warmup (10 iterations)");
    for _ in 0..10 {
        format_gdscript_with_buffers(&short_content, &config, &mut render_elements, &mut output)?;
    }

    println!("Benchmarking short file ({} iterations)", ITERATIONS);
    let mut start = Instant::now();
    for _ in 0..ITERATIONS {
        format_gdscript_with_buffers(&short_content, &config, &mut render_elements, &mut output)?;
    }
    let duration_short_file = start.elapsed();

    println!("Benchmarking long file ({} iterations)...", ITERATIONS);
    start = Instant::now();
    for _ in 0..ITERATIONS {
        format_gdscript_with_buffers(&long_content, &config, &mut render_elements, &mut output)?;
    }
    let long_time = start.elapsed();

    let safe_config = FormatterConfiguration {
        safe: true,
        ..config
    };

    println!(
        "Benchmarking short file with safe mode ({} iterations)...",
        ITERATIONS
    );
    start = Instant::now();
    for _ in 0..ITERATIONS {
        format_gdscript_with_buffers(
            &short_content,
            &safe_config,
            &mut render_elements,
            &mut output,
        )?;
    }
    let duration_short_file_safe = start.elapsed();

    println!(
        "Benchmarking long file with safe mode ({} iterations)...",
        ITERATIONS
    );
    start = Instant::now();
    for _ in 0..ITERATIONS {
        format_gdscript_with_buffers(
            &long_content,
            &safe_config,
            &mut render_elements,
            &mut output,
        )?;
    }
    let long_time_safe = start.elapsed();

    let average_time_short = duration_short_file.as_micros() as f64 / f64::from(ITERATIONS);
    let average_time_long = long_time.as_micros() as f64 / f64::from(ITERATIONS);
    let average_time_safe_short =
        duration_short_file_safe.as_micros() as f64 / f64::from(ITERATIONS);
    let average_time_safe_long = long_time_safe.as_micros() as f64 / f64::from(ITERATIONS);

    let short_slowdown =
        ((average_time_safe_short - average_time_short) / average_time_short) * 100.0;
    let long_slowdown = ((average_time_safe_long - average_time_long) / average_time_long) * 100.0;

    println!("\nBenchmark Results:");
    println!("=================");
    println!(
        "Short file ({} iterations): {:?} (avg: {:.2}ms per iteration)",
        ITERATIONS,
        duration_short_file,
        average_time_short / 1000.0
    );
    println!(
        "Long file ({} iterations):   {:?} (avg: {:.2}ms per iteration)",
        ITERATIONS,
        long_time,
        average_time_long / 1000.0
    );
    println!(
        "Short file with safe mode ({} iterations): {:?} (avg: {:.2}ms per iteration, {:.1}% slower)",
        ITERATIONS,
        duration_short_file_safe,
        average_time_safe_short / 1000.0,
        short_slowdown
    );
    println!(
        "Long file with safe mode ({} iterations):   {:?} (avg: {:.2}ms per iteration, {:.1}% slower)",
        ITERATIONS,
        long_time_safe,
        average_time_safe_long / 1000.0,
        long_slowdown
    );

    Ok(())
}
