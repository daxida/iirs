use rand::prelude::SliceRandom;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::time::Instant;

const FILE_NAME: &str = "rand.fasta";
const MAX_GAP: usize = 100;
const MISMATCHES: usize = 2;

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

fn run(cmd_beginning: &str) -> f64 {
    let start = Instant::now();

    let command = format!(
        "{} -f {} -g {} -x {}",
        cmd_beginning, FILE_NAME, MAX_GAP, MISMATCHES
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // println!("{}", stdout);
    // println!("{}", stderr);

    if stderr.contains("panic") {
        println!("Stderr: {}", stderr);
    }

    if stdout.contains("Error") {
        println!("Error: {}", stdout);
        std::process::exit(0);
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

fn mean(timings: &Vec<f64>) -> f64 {
    let sz = timings.len() as f64; // Convert sz to f64
    let total: f64 = timings.iter().sum(); // Use i64 for total to avoid overflow

    total / sz // Perform floating-point division
}

fn main() {
    let start = Instant::now();

    // Do this with clap
    let mut size_fasta = 1_000;
    let mut n_tests = 20;

    let args: Vec<String> = env::args().collect();
    let mut args_iter = args.iter();

    while let Some(arg) = args_iter.next() {
        if arg == "--size" {
            if let Some(size_str) = args_iter.next() {
                if let Ok(size) = size_str.parse::<usize>() {
                    size_fasta = size;
                }
            }
        } else if arg == "--ntests" {
            if let Some(ntests_str) = args_iter.next() {
                if let Ok(ntests) = ntests_str.parse::<usize>() {
                    n_tests = ntests;
                }
            }
        }
    }

    // Compile Rust binary
    // Command::new("cargo")
    //     .arg("build")
    //     .arg("--release")
    //     .output()
    //     .expect("Failed to compile Rust binary");

    let mut rust_timings = Vec::new();
    let mut cpp_timings = Vec::new();

    for _ in 0..n_tests {
        let fasta = generate_random_fasta(size_fasta);
        let mut file = File::create(FILE_NAME).expect("Failed to create file");
        file.write_all(fasta.as_bytes())
            .expect("Failed to write to file");

        let cpp_timing = run("IUPACpal/IUPACpal");
        let rust_timing = run("target/release/main");

        cpp_timings.push(cpp_timing);
        rust_timings.push(rust_timing);

        test_equality();
    }

    println!(
        "Results for {} random tests of size {}",
        n_tests, size_fasta
    );

    println!("cpp  average: {}", mean(&cpp_timings));
    println!("rust average: {}", mean(&rust_timings));

    println!("\nAll tests finished in {}", start.elapsed().as_secs_f64());
}
