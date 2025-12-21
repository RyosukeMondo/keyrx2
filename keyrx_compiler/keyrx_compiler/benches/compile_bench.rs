use criterion::{black_box, criterion_group, criterion_main, Criterion};
use keyrx_compiler::parser::Parser;
use std::path::Path;

fn bench_parse_simple(c: &mut Criterion) {
    let mut parser = Parser::new();
    let script = r#"
        device_start("keyboard");
        map("A", "VK_B");
        device_end();
    "#;
    c.bench_function("parse_simple", |b| b.iter(|| {
        parser.parse_string(black_box(script), black_box(Path::new("bench.rhai"))).unwrap()
    }));
}

fn bench_parse_complex(c: &mut Criterion) {
    let mut parser = Parser::new();
    let mut script = String::from("device_start(\"keyboard\");\n");
    for i in 0..100 {
        script.push_str(&format!("map(\"A\", \"VK_B\"); // {}\n", i));
    }
    script.push_str("device_end();");
    
    c.bench_function("parse_complex_100_mappings", |b| b.iter(|| {
        parser.parse_string(black_box(&script), black_box(Path::new("bench.rhai"))).unwrap()
    }));
}

criterion_group!(benches, bench_parse_simple, bench_parse_complex);
criterion_main!(benches);
