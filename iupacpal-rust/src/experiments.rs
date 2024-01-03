// use proc_macro::TokenStream;
// use quote::quote;
// use syn::{parse_macro_input, ItemFn};

// #[proc_macro_attribute]
// pub fn timeit(_: TokenStream, input: TokenStream) -> TokenStream {
//     let input_fn = parse_macro_input!(input as ItemFn);

//     let fn_name = &input_fn.sig.ident;
//     let expanded = quote! {
//         #input_fn

//         fn #fn_name() {
//             let start_time = std::time::Instant::now();
//             let result = #fn_name();
//             let elapsed_time = start_time.elapsed();
//             println!("Function '{}' elapsed time: {:?}", stringify!(#fn_name), elapsed_time);
//             result
//         }
//     };

//     TokenStream::from(expanded)
// }

// mji test
// macro_rules! timed {
//     ($($tt:tt)+) => {
//         let timer = std::time::Instant::now();
//         $($tt)+
//         println!("took {:?}", timer.elapsed());
//     };
// }

// fn main() {
//     timed!{
//         println!("hello");
//         println!("world");
//     }
// }
