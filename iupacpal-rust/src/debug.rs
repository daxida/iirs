use std::fmt::Display;

pub fn print_array<T: Display>(title: &str, seq: &[T], print_indices: bool) {
    if print_indices {
        print!("{:>width$}", "", width = title.len() + 2);
        for (i, ch) in seq.iter().enumerate() {
            print!("{:>width$}", i, width = ch.to_string().len() + 2);
        }
        println!();
    }

    println!("{}:", title);
    for ch in seq {
        print!("{}  ", ch);
    }
    println!("\n");
}
