use clap::Parser;
use rand::prelude::SliceRandom;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::time::Instant;

const GREEN: &str = "\x1B[32m";
const RESET: &str = "\x1B[0m";

fn generate_random_fasta(size: usize) -> String {
    let mut rng = rand::thread_rng();
    format!(
        ">seq0\n{}",
        (0..size)
            .map(|_| {
                *[
                    "a", "c", "g", "t", "u", "r", "y", "s", "w", "k", "m", "b", "d", "h", "v", "*",
                    "-",
                ]
                .choose(&mut rng)
                .unwrap()
            })
            .collect::<String>()
    )
}

fn run_command(cmd_beginning: &str, config: &BenchConfig) -> f64 {
    let start = Instant::now();

    let command = format!(
        "{} -f {} -m {} -M {} -g {} -x {}",
        cmd_beginning,
        config.input_file,
        config.min_len,
        config.max_len,
        config.max_gap,
        config.mismatches
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if stderr.contains("panic") {
        println!("Stderr: {}", stderr);
    }

    if stdout.contains("Error") {
        println!("Error: {}", stdout);
        // std::process::exit(0);
    }

    start.elapsed().as_secs_f64()
}

fn test_equality() {
    let expected = fs::read_to_string("IUPACpal.out")
        .expect("Failed to read file")
        .trim()
        .to_string();
    let received = fs::read_to_string("IUPACpalrs.out")
        .expect("Failed to read file")
        .trim()
        .to_string();

    let expected = expected.replace("Palindromes:\n", "");
    let received = received.replace("Palindromes:\n", "");
    let expected_lines: Vec<&str> = expected.split("\n\n").skip(1).collect();
    let received_lines: Vec<&str> = received.split("\n\n").skip(1).collect();

    // Sort then to compare when trying alternative sorting strategies in find_palindromes
    //
    // fn block_sort(block: &str) -> (usize, usize, usize, usize) {
    //     if block.trim().is_empty() {
    //         return (1_000_000_000, 0, 0, 0);
    //     }
    //     let lines: Vec<&str> = block.split('\n').collect();
    //     let mut iter1 = lines[0].split_whitespace();
    //     // dbg!(&iter1.clone().collect::<Vec<&str>>().join("-"));
    //     let a = iter1.next().unwrap();
    //     let _ = iter1.next(); // Skip the second element
    //     let c = iter1.next().unwrap();

    //     let mut iter2 = lines[2].split_whitespace();
    //     let d = iter2.next().unwrap();
    //     let _ = iter2.next(); // Skip the second element
    //     let f = iter2.next().unwrap();
    //     // println!("{} {} {} {}", &a, &c, &d, &f);
    //     (
    //         a.parse().unwrap(),
    //         c.parse().unwrap(),
    //         d.parse().unwrap(),
    //         f.parse().unwrap(),
    //     )
    // }

    // expected_lines.sort_by(|a, b| block_sort(a).cmp(&block_sort(b)));
    // received_lines.sort_by(|a, b| block_sort(a).cmp(&block_sort(b)));

    assert_eq!(expected_lines.len(), received_lines.len(),);

    for (el, rl) in expected_lines.iter().zip(received_lines.iter()) {
        assert_eq!(el, rl, "Received line:\n{}\nbut expected:\n{}", rl, el);
    }

    println!(
        "{}OK{}: Compared {} Palindromes",
        GREEN,
        RESET,
        expected_lines.len() - 1
    );
}

fn average(timings: &Vec<f64>) -> f64 {
    let total: f64 = timings.iter().sum();
    total / timings.len() as f64
}

#[derive(Parser)]
struct BenchConfig {
    /// Input filename (FASTA).
    #[arg(short = 'f', default_value_t = String::from("rand.fasta"))]
    input_file: String,

    /// Input sequence name.
    #[arg(short, default_value_t = String::from("seq0"))]
    seq_name: String,

    /// Minimum length.
    #[arg(short, default_value_t = 10)]
    min_len: i32,

    /// Maximum length.
    #[arg(short = 'M', default_value_t = 100)]
    max_len: i32,

    /// Maximum permissible gap.
    #[arg(short = 'g', default_value_t = 100)]
    max_gap: i32,

    /// Maximum permissible mismatches.
    #[arg(short = 'x', default_value_t = 0)]
    mismatches: i32,

    /// Output filename.
    #[arg(short, default_value_t = String::from("IUPACpalrs.out"))]
    output_file: String,

    /// Output format (classic, csv or custom_csv).
    #[arg(short = 'F', default_value_t = String::from("classic"))]
    output_format: String,

    /// Size of the generated fasta
    #[arg(long, default_value_t = 1000)]
    size_fasta: usize,

    /// Number of tests to perform
    #[arg(long, default_value_t = 20)]
    n_tests: usize,
}

fn main() {
    let start = Instant::now();

    let config = BenchConfig::parse();

    let mut rust_timings = Vec::new();
    let mut cpp_timings = Vec::new();

    for _ in 0..config.n_tests {
        let fasta = generate_random_fasta(config.size_fasta);
        let mut file = File::create(config.input_file.clone()).expect("Failed to create file");
        file.write_all(fasta.as_bytes())
            .expect("Failed to write to file");

        let cpp_timing = run_command("IUPACpal/IUPACpal", &config);
        let rust_timing = run_command("target/release/main", &config);

        cpp_timings.push(cpp_timing);
        rust_timings.push(rust_timing);

        test_equality();
    }

    println!(
        "Results for {} random tests of size {}",
        config.n_tests, config.size_fasta
    );

    println!("cpp  average: {}", average(&cpp_timings));
    println!("rust average: {}", average(&rust_timings));

    println!("\nAll tests finished in {}", start.elapsed().as_secs_f64());
}
