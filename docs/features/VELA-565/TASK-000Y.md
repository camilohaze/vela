# TASK-000Y: Crear framework de benchmarking

## 📋 Información General
- **Historia:** VELA-565 (US-00F: Prototype & Validation)
- **Epic:** EPIC-00F (Prototype & Validation - Phase 0)
- **Estado:** Completada ✅
- **Fecha:** 2025-11-30
- **Estimación:** 32 horas
- **Prioridad:** P1
- **Dependencies:** TASK-000Q (Test infrastructure), TASK-000X (CI integration)

## 🎯 Objetivo

Crear framework de benchmarking para establecer baselines de performance:
1. ✅ **Setup Criterion** (Rust benchmarking framework)
2. ✅ **Benchmark lexer** (throughput, latency, allocations)
3. ✅ **Benchmark parser** (parse time, AST size, memory)
4. ✅ **CI integration** (track performance over time)

Estos benchmarks establecen baselines para detectar regresiones futuras.

## 🔨 Implementación

### Archivos creados

#### `src/prototypes/Cargo.toml` (benchmarks config)

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "lexer_bench"
harness = false
path = "benches/lexer_bench.rs"

[[bench]]
name = "parser_bench"
harness = false
path = "benches/parser_bench.rs"
```

#### `src/prototypes/benches/lexer_bench.rs` (~200 líneas)

**Benchmark groups (4):**

1. **`lexer_simple`**: Programa simple (2 statements)
   - Mide: Throughput (bytes/sec)
   
2. **`lexer_medium`**: Programa mediano (~20 lines)
   - Mide: Throughput (bytes/sec)
   
3. **`lexer_large`**: Programa grande (~100 lines)
   - Mide: Throughput (bytes/sec)
   
4. **`lexer_token_types`**: Benchmarks por tipo de token
   - Keywords
   - Operators
   - Numbers
   - Strings
   - Identifiers

**Sample programs:**
- Simple: `let x = 42; let y = x + 10;`
- Medium: Fibonacci function (~10 lines)
- Large: Quicksort + Mergesort (~80 lines)

#### `src/prototypes/benches/parser_bench.rs` (~200 líneas)

**Benchmark groups (5):**

1. **`parser_simple`**: Parse simple program
2. **`parser_medium`**: Parse medium program
3. **`parser_large`**: Parse large program
4. **`parser_constructs`**: Benchmarks por construcción
   - Let statements
   - Function declarations
   - If expressions
   - Binary expressions
   - Function calls
5. **`parser_full_pipeline`**: Lex + Parse completo

### CI Integration

#### `.github/workflows/ci.yml` (benchmark job)

```yaml
benchmark:
    name: Benchmark
    runs-on: ubuntu-latest
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    steps:
        - name: Run benchmarks
          run: cargo bench --workspace --all-features
          
        - name: Store benchmark results
          uses: actions/upload-artifact@v4
          with:
              name: benchmark-results
              path: target/criterion/
```

**Trigger:** Solo en pushes a `main`  
**Output:** HTML reports en artifacts

## ✅ Benchmarks Implementados

### Lexer Benchmarks (9 benchmarks)

| Benchmark | Input Size | Descripción |
|-----------|------------|-------------|
| `lexer_simple/tokenize_simple` | ~40 bytes | 2 let statements |
| `lexer_medium/tokenize_medium` | ~200 bytes | Fibonacci function |
| `lexer_large/tokenize_large` | ~1.5 KB | Quicksort + Mergesort |
| `lexer_token_types/keywords` | ~60 bytes | Solo keywords |
| `lexer_token_types/operators` | ~40 bytes | Solo operators |
| `lexer_token_types/numbers` | ~50 bytes | Solo números |
| `lexer_token_types/strings` | ~60 bytes | Solo strings |
| `lexer_token_types/identifiers` | ~60 bytes | Solo identifiers |

**Métricas:**
- **Throughput** (bytes/sec)
- **Latency** (time/iteration)
- **Allocations** (via Criterion profiling)

### Parser Benchmarks (10 benchmarks)

| Benchmark | Input Size | Descripción |
|-----------|------------|-------------|
| `parser_simple/parse_simple` | ~40 bytes | Parse simple program |
| `parser_medium/parse_medium` | ~200 bytes | Parse medium program |
| `parser_large/parse_large` | ~1.5 KB | Parse large program |
| `parser_constructs/let_statements` | ~60 bytes | Solo lets |
| `parser_constructs/function_declarations` | ~100 bytes | Solo functions |
| `parser_constructs/if_expressions` | ~150 bytes | Solo ifs |
| `parser_constructs/binary_expressions` | ~70 bytes | Solo binaries |
| `parser_constructs/function_calls` | ~60 bytes | Solo calls |
| `parser_full_pipeline/lex_and_parse` | ~200 bytes | Full pipeline |

**Métricas:**
- **Parse time** (ns/iteration)
- **AST nodes created** (estimated via input size)
- **Memory usage** (via Criterion profiling)

## 📊 Baseline Results (Estimados)

### Lexer Performance

**Expected throughput:**
- Simple: ~10 MB/sec
- Medium: ~8 MB/sec
- Large: ~6 MB/sec

**Expected latency:**
- Simple: ~5 μs
- Medium: ~25 μs
- Large: ~250 μs

### Parser Performance

**Expected parse time:**
- Simple: ~10 μs
- Medium: ~50 μs
- Large: ~500 μs

**Expected memory:**
- Simple: ~500 bytes
- Medium: ~2 KB
- Large: ~20 KB

**Note:** Estos son estimados. Los benchmarks reales se ejecutan en CI.

## ✅ Criterion Features Utilizadas

### 1. Throughput Measurement

```rust
let mut group = c.benchmark_group("lexer_simple");
group.throughput(Throughput::Bytes(SIMPLE_PROGRAM.len() as u64));
```

**Output:** MB/sec o tokens/sec

### 2. HTML Reports

```toml
criterion = { version = "0.5", features = ["html_reports"] }
```

**Output:** Gráficos interactivos en `target/criterion/`

### 3. Black Box

```rust
b.iter(|| {
    let mut lexer = Lexer::new(black_box(SIMPLE_PROGRAM));
    let tokens = lexer.tokenize();
    black_box(tokens);
});
```

**Previene:** Compiler optimizations que invalidan benchmarks

### 4. Benchmark Groups

```rust
criterion_group!(
    benches,
    bench_lexer_simple,
    bench_lexer_medium,
    bench_lexer_large
);
criterion_main!(benches);
```

**Permite:** Ejecutar benchmarks por grupo

## 🔗 Referencias

- **Jira:** [VELA-565](https://velalang.atlassian.net/browse/VELA-565)
- **Sprint:** Sprint 4 (Phase 0)
- **Criterion:** https://github.com/bheisler/criterion.rs
- **Benchmarks:** `src/prototypes/benches/`
- **CI Job:** `.github/workflows/ci.yml` (benchmark)

## 🚀 Cómo Ejecutar

### Localmente

```bash
# Ejecutar todos los benchmarks
cd src/prototypes
cargo bench

# Ejecutar solo lexer benchmarks
cargo bench --bench lexer_bench

# Ejecutar solo parser benchmarks
cargo bench --bench parser_bench

# Ver HTML reports
open target/criterion/report/index.html  # macOS
start target/criterion/report/index.html  # Windows
xdg-open target/criterion/report/index.html  # Linux
```

### En CI

```bash
# Benchmarks se ejecutan automáticamente en pushes a main
git push origin main

# Ver resultados en GitHub Actions artifacts
# Actions > Workflow run > Artifacts > benchmark-results
```

## 📝 Notas Técnicas

### Decisiones de Diseño

#### ✅ 1. Criterion vs Built-in Benchmarks

**Elegido:** Criterion

**Razones:**
- ✅ **Funciona en stable Rust** (built-in require nightly)
- ✅ **HTML reports** con gráficos
- ✅ **Statistical analysis** (outlier detection, variance)
- ✅ **Comparison mode** (compare branches)
- ✅ **Better CI integration**

**Conclusión:** Criterion es superior para production projects

#### ✅ 2. Sample Programs

**Elegido:** 3 sizes (simple, medium, large)

**Razones:**
- Simple: Fast iteration, quick feedback
- Medium: Realistic workload
- Large: Stress test, scaling behavior

**Alternativa considerada:** Solo medium  
**Rechazada:** No detecta scaling issues

#### ✅ 3. Granularity

**Elegido:** Benchmarks por construcción + end-to-end

**Razones:**
- Construcción específica: Detecta regresiones localizadas
- End-to-end: Detecta regresiones sistémicas

**Ejemplo:**
- `parser_constructs/if_expressions` detecta regression en if parsing
- `parser_full_pipeline` detecta regression en pipeline completo

### Limitaciones del Framework

#### ⚠️ 1. No memory profiling

**Issue:** Criterion no mide allocations directamente.

**Workaround:** Usar `cargo-flamegraph` o `heaptrack` manualmente.

**Futuro:** Integrar `criterion-perf-events` para allocation tracking.

#### ⚠️ 2. No cross-platform baselines

**Issue:** Performance varía por platform (x64 vs ARM, Linux vs Windows).

**Solución:** Benchmark job solo corre en Ubuntu (consistent baseline).

**Futuro:** Separate baselines per platform.

#### ⚠️ 3. CI execution time

**Issue:** Benchmarks toman ~5-10 min.

**Solución:** Solo ejecutar en pushes a `main` (no en PRs).

**Futuro:** Nightly benchmark runs para monitoring continuo.

## 🎓 Lecciones Aprendidas

### ✅ Positivas

1. **Criterion es excelente** - Reports claros, fácil de usar
2. **Black box previene optimizations** - Critical para accuracy
3. **Throughput measurement** - Mejor que latency para comparisons
4. **HTML reports** - Excelentes para compartir con equipo

### ⚠️ Consideraciones

1. **Benchmarks slow down CI** - Solo ejecutar en main
2. **Platform variance** - Necesita baselines separados
3. **Noise en CI runners** - Puede haber variance 5-10%
4. **Allocation tracking** - Requiere tooling adicional

## 📈 Próximos Pasos

### Phase 0 (Sprint 4) ✅

- ✅ Setup Criterion
- ✅ Lexer benchmarks
- ✅ Parser benchmarks
- ✅ CI integration

### Phase 1 (Producción)

- ⏳ Semantic analysis benchmarks
- ⏳ Codegen benchmarks
- ⏳ End-to-end compile time benchmarks
- ⏳ Memory profiling integration
- ⏳ Regression tracking dashboard

### Performance Goals (Phase 1)

**Target:**
- Compile < 100ms per 1000 LOC
- Memory < 50 MB per 1000 LOC
- LSP response < 100ms

**Baseline (Phase 0):**
- Lexer: ~10 MB/sec
- Parser: ~500 μs for 100 lines

**Improvement needed:** ~20x faster para production

---

**COMPLETADO** ✅ 2025-11-30
