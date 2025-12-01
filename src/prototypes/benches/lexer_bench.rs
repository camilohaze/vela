/// Lexer Benchmarks
///
/// JIRA: VELA-565 (Sprint 4)
/// TASK: TASK-000Y - Crear framework de benchmarking
///
/// Mide performance del lexer prototype:
/// - Throughput (tokens/sec)
/// - Latency por token
/// - Memory allocations

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use vela_prototypes::Lexer;

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

fn bench_lexer_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_simple");
    group.throughput(Throughput::Bytes(SIMPLE_PROGRAM.len() as u64));

    group.bench_function("tokenize_simple", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(SIMPLE_PROGRAM));
            let tokens = lexer.tokenize();
            black_box(tokens);
        });
    });

    group.finish();
}

fn bench_lexer_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_medium");
    group.throughput(Throughput::Bytes(MEDIUM_PROGRAM.len() as u64));

    group.bench_function("tokenize_medium", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(MEDIUM_PROGRAM));
            let tokens = lexer.tokenize();
            black_box(tokens);
        });
    });

    group.finish();
}

fn bench_lexer_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_large");
    group.throughput(Throughput::Bytes(LARGE_PROGRAM.len() as u64));

    group.bench_function("tokenize_large", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(LARGE_PROGRAM));
            let tokens = lexer.tokenize();
            black_box(tokens);
        });
    });

    group.finish();
}

fn bench_lexer_token_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_token_types");

    // Benchmark specific token types
    group.bench_function("keywords", |b| {
        let source = "let fn if else return true false let fn if else return";
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source));
            let tokens = lexer.tokenize();
            black_box(tokens);
        });
    });

    group.bench_function("operators", |b| {
        let source = "+ - * / == != < > = + - * / == != < >";
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source));
            let tokens = lexer.tokenize();
            black_box(tokens);
        });
    });

    group.bench_function("numbers", |b| {
        let source = "42 123 999 0 1 2 3 4 5 6 7 8 9 10 100 1000";
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source));
            let tokens = lexer.tokenize();
            black_box(tokens);
        });
    });

    group.bench_function("strings", |b| {
        let source = r#""hello" "world" "foo" "bar" "test" "example""#;
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source));
            let tokens = lexer.tokenize();
            black_box(tokens);
        });
    });

    group.bench_function("identifiers", |b| {
        let source = "foo bar baz qux test example variable name identifier";
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source));
            let tokens = lexer.tokenize();
            black_box(tokens);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_lexer_simple,
    bench_lexer_medium,
    bench_lexer_large,
    bench_lexer_token_types
);
criterion_main!(benches);
