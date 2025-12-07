mod ast;
mod lexer;
mod renderer;

use std::{env, fs, time::Instant};

use lexer::Parser;

use crate::renderer::HtmlRenderer;

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
