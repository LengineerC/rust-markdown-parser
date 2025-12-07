mod ast;
mod lexer;
mod renderer;

use std::{env, fs, time::Instant};

use lexer::Parser;

use crate::renderer::HtmlRenderer;

// marked.js
// ================================
// loop 0 cost: 159.6985ms
// loop 1 cost: 80.4278ms
// loop 2 cost: 66.8025ms
// loop 3 cost: 65.4549ms
// loop 4 cost: 58.3078ms
// ================================
// Average cost: 86.1383ms
//
// dev
// total cost: 313.9141ms
// ================================
// loop 0 cost: 63.2594ms
// loop 1 cost: 60.8249ms
// loop 2 cost: 69.72ms
// loop 3 cost: 61.4909ms
// loop 4 cost: 58.6189ms
// ================================
// Average cost: 62.78282ms
//
// release
// total cost: 95.2053ms
// ================================
// loop 0 cost: 18.7684ms
// loop 1 cost: 18.3859ms
// loop 2 cost: 18.5061ms
// loop 3 cost: 18.7554ms
// loop 4 cost: 20.7895ms
// ================================
// Average cost: 19.04106ms
//
// pulldown-mark (dev)
// total cost: 148.8262ms
// ================================
// loop 0 cost: 31.8808ms
// loop 1 cost: 28.6303ms
// loop 2 cost: 28.8125ms
// loop 3 cost: 28.7767ms
// loop 4 cost: 30.7259ms
// ================================
// Average cost: 29.76524ms
fn performace_test() {
    let loop_time = 5;
    let mut costs: Vec<u128> = Vec::new();

    let cwd = env::current_dir().unwrap();
    let md_path = cwd.join("performance.md");

    let md_bytes = fs::read(md_path).unwrap();
    let mut md_string = String::from_utf8(md_bytes).unwrap();
    
    md_string = Parser::preprocess(&md_string);

    let mut parser = Parser::new(&md_string);

    for _ in 0..loop_time {
        let start_time = Instant::now();

        HtmlRenderer::render(&parser.parse());

        let duration = start_time.elapsed();
        costs.push(duration.as_nanos());
    }

    let total_cost: u128 = costs.iter().sum();
    println!("total cost: {}ms", total_cost as f64 / 1_000_000.0);
    println!("================================");
    costs
        .iter()
        .enumerate()
        .for_each(|(i, c)| println!("loop {} cost: {:?}ms", i, *c as f64 / 1_000_000.0));
    println!("================================");
    println!(
        "Average cost: {:?}ms",
        total_cost as f64 / loop_time as f64 / 1_000_000.0
    );
}

fn output_test() {
    let cwd = env::current_dir().unwrap();
    let md_path = cwd.join("test.md");
    let html_path = cwd.join("test.html");

    let md_bytes = fs::read(md_path).unwrap();
    let mut md_string = String::from_utf8(md_bytes).unwrap();
    md_string = Parser::preprocess(&md_string);

    let mut p = Parser::new(&md_string);
    let ast = p.parse();

    println!("{:#?}", ast);

    let html = HtmlRenderer::render(&ast);

    let full_html = format!(
        r#"<!DOCTYPE html>
    <html>
    <head><title>Rust Markdown Compiler</title></head>
    <body>

    {}

    </body>
    </html>"#,
        html
    );

    fs::write(html_path, full_html).unwrap();
}

fn main() {
    output_test();
    performace_test();
}
