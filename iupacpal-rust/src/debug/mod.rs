use std::fmt::Display;

// type T = u8;

pub fn print_array<T: Display>(title: &str, seq: &[T], n: usize, print_indices: bool) {
    // pub fn print_array(title: &str, seq: &[u8], n: usize, print_indices: bool) {
    if print_indices {
        print!("{:>width$}", "", width = title.len() + 2);
        for i in 0..n {
            print!("{:>width$}", i, width = seq[i].to_string().len() + 2);
        }
        println!();
    }

    println!("{}:", title);
    for i in 0..n {
        print!("{}  ", seq[i]);
    }
    println!("\n");
}
