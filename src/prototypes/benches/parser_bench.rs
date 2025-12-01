/// Parser Benchmarks
///
/// JIRA: VELA-565 (Sprint 4)
/// TASK: TASK-000Y - Crear framework de benchmarking
///
/// Mide performance del parser prototype:
/// - Parse time
/// - AST node allocations
/// - Memory usage

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use vela_prototypes::parse_source;

// Sample Vela programs for benchmarking
const SIMPLE_PROGRAM: &str = r#"
let x = 42;
let y = x + 10;
"#;

const MEDIUM_PROGRAM: &str = r#"
fn fibonacci(n) {
    if n < 2 {
        return n;
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}

let result = fibonacci(10);
"#;

const LARGE_PROGRAM: &str = r#"
fn quicksort(arr) {
    if arr.length <= 1 {
        return arr;
    }
    
    let pivot = arr[0];
    let less = [];
    let greater = [];
    
    let i = 1;
    if i < arr.length {
        if arr[i] < pivot {
            less.push(arr[i]);
        } else {
            greater.push(arr[i]);
        }
        i = i + 1;
    }
    
    return quicksort(less) + [pivot] + quicksort(greater);
}

fn mergesort(arr) {
    if arr.length <= 1 {
        return arr;
    }
    
    let mid = arr.length / 2;
    let left = arr[0..mid];
    let right = arr[mid..arr.length];
    
    return merge(mergesort(left), mergesort(right));
}

fn merge(left, right) {
    let result = [];
    let i = 0;
    let j = 0;
    
    if i < left.length {
        if j < right.length {
            if left[i] < right[j] {
                result.push(left[i]);
                i = i + 1;
            } else {
                result.push(right[j]);
                j = j + 1;
            }
        } else {
            result.push(left[i]);
            i = i + 1;
        }
    }
    
    if j < right.length {
        result.push(right[j]);
        j = j + 1;
    }
    
    return result;
}
"#;

fn bench_parser_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_simple");
    group.throughput(Throughput::Bytes(SIMPLE_PROGRAM.len() as u64));

    group.bench_function("parse_simple", |b| {
        b.iter(|| {
            let program = parse_source(black_box(SIMPLE_PROGRAM)).unwrap();
            black_box(program);
        });
    });

    group.finish();
}

fn bench_parser_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_medium");
    group.throughput(Throughput::Bytes(MEDIUM_PROGRAM.len() as u64));

    group.bench_function("parse_medium", |b| {
        b.iter(|| {
            let program = parse_source(black_box(MEDIUM_PROGRAM)).unwrap();
            black_box(program);
        });
    });

    group.finish();
}

fn bench_parser_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_large");
    group.throughput(Throughput::Bytes(LARGE_PROGRAM.len() as u64));

    group.bench_function("parse_large", |b| {
        b.iter(|| {
            let program = parse_source(black_box(LARGE_PROGRAM)).unwrap();
            black_box(program);
        });
    });

    group.finish();
}

fn bench_parser_constructs(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_constructs");

    // Benchmark specific constructs
    group.bench_function("let_statements", |b| {
        let source = "let x = 1; let y = 2; let z = 3; let a = 4; let b = 5;";
        b.iter(|| {
            let program = parse_source(black_box(source)).unwrap();
            black_box(program);
        });
    });

    group.bench_function("function_declarations", |b| {
        let source = r#"
            fn foo() { return 1; }
            fn bar(x) { return x; }
            fn baz(x, y) { return x + y; }
        "#;
        b.iter(|| {
            let program = parse_source(black_box(source)).unwrap();
            black_box(program);
        });
    });

    group.bench_function("if_expressions", |b| {
        let source = r#"
            let x = if true { let y = 1; y; } else { let z = 2; z; };
            let a = if false { let b = 3; b; } else { let c = 4; c; };
        "#;
        b.iter(|| {
            let program = parse_source(black_box(source)).unwrap();
            black_box(program);
        });
    });

    group.bench_function("binary_expressions", |b| {
        let source = "let x = 1 + 2 * 3 - 4 / 5; let y = 10 == 5 + 5; let z = 20 < 30;";
        b.iter(|| {
            let program = parse_source(black_box(source)).unwrap();
            black_box(program);
        });
    });

    group.bench_function("function_calls", |b| {
        let source = "let x = foo(); let y = bar(1); let z = baz(1, 2);";
        b.iter(|| {
            let program = parse_source(black_box(source)).unwrap();
            black_box(program);
        });
    });

    group.finish();
}

fn bench_parser_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_full_pipeline");

    // Benchmark entire pipeline: lex + parse
    group.bench_function("lex_and_parse", |b| {
        b.iter(|| {
            let program = parse_source(black_box(MEDIUM_PROGRAM)).unwrap();
            black_box(program);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parser_simple,
    bench_parser_medium,
    bench_parser_large,
    bench_parser_constructs,
    bench_parser_full_pipeline
);
criterion_main!(benches);
