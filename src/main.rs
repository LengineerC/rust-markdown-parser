mod ast;
mod lexer;
mod renderer;

use std::{env, error::Error, fs};

use lexer::Parser;

use crate::renderer::HtmlRenderer;

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

    Ok(())
}
