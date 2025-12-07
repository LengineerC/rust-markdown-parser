mod ast;
mod lexer;
mod renderer;

use std::{env, error::Error, fs, time::Instant};

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
// total cost: 383.3889ms
// ================================
// loop 0 cost: 75.4211ms
// loop 1 cost: 77.7203ms
// loop 2 cost: 80.9819ms
// loop 3 cost: 74.4916ms
// loop 4 cost: 74.774ms
// ================================
// Average cost: 76.67778ms
// 
// release
// total cost: 123.2115ms
// ================================
// loop 0 cost: 24.461ms
// loop 1 cost: 26.0559ms
// loop 2 cost: 23.8037ms
// loop 3 cost: 24.2078ms
// loop 4 cost: 24.6831ms
// ================================
// Average cost: 24.6423ms
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
    let md_path = cwd.join("test.md");

    let md_bytes = fs::read(md_path).unwrap();
    let md_string = String::from_utf8(md_bytes).unwrap();

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

fn main() -> Result<(), Box<dyn Error>> {
    let cwd = env::current_dir()?;
    let md_path = cwd.join("test.md");
    let html_path = cwd.join("test.html");

    let md_bytes = fs::read(md_path)?;
    let md_string = String::from_utf8(md_bytes)?;

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

    fs::write(html_path, full_html)?;

    performace_test();

    Ok(())
}
