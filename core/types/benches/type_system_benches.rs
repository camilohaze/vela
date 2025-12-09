/*
Benchmark completo para el sistema de tipos de Vela

Este benchmark mide el rendimiento de las operaciones principales
del sistema de tipos: creación de contextos, variables de tipo,
operaciones de tipos, unificación e inferencia.
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use types::{Type, TypeContext, TypeVar, TypeScheme};
use std::collections::HashMap;

fn bench_simple(c: &mut Criterion) {
    c.bench_function("simple", |b| {
        b.iter(|| {
            let mut x = 0;
            for i in 0..1000 {
                x += i;
            }
            x
        })
    });
}

fn bench_type_context_creation(c: &mut Criterion) {
    c.bench_function("type_context_creation", |b| {
        b.iter(|| {
            let ctx = TypeContext::new();
            black_box(ctx);
        })
    });
}

fn bench_type_var_creation(c: &mut Criterion) {
    c.bench_function("type_var_creation", |b| {
        b.iter(|| {
            let tv = TypeVar::fresh();
            black_box(tv);
        })
    });
}

fn bench_type_operations(c: &mut Criterion) {
    let tv1 = TypeVar::fresh();
    let tv2 = TypeVar::fresh();

    // Crear tipos complejos para las pruebas
    let complex_type = Type::Function {
        params: vec![
            Type::Var(tv1),
            Type::Array(Box::new(Type::Int)),
            Type::Record({
                let mut fields = HashMap::new();
                fields.insert("name".to_string(), Type::String);
                fields.insert("age".to_string(), Type::Int);
                fields.insert("data".to_string(), Type::Var(tv2));
                fields
            }),
        ],
        ret: Box::new(Type::Result {
            ok: Box::new(Type::Tuple(vec![Type::Bool, Type::Var(tv1)])),
            err: Box::new(Type::String),
        }),
    };

    c.bench_function("type_free_vars", |b| {
        b.iter(|| {
            let vars = complex_type.free_vars();
            black_box(vars);
        })
    });

    c.bench_function("type_is_mono", |b| {
        b.iter(|| {
            let is_mono = complex_type.is_mono();
            black_box(is_mono);
        })
    });

    c.bench_function("type_apply_subst", |b| {
        let mut subst = HashMap::new();
        subst.insert(tv1, Type::String);
        subst.insert(tv2, Type::Float);

        b.iter(|| {
            let mut ty = complex_type.clone();
            ty.apply_subst(&subst);
            black_box(ty);
        })
    });
}

fn bench_context_operations(c: &mut Criterion) {
    c.bench_function("context_scope_operations", |b| {
        b.iter(|| {
            let mut ctx = TypeContext::new();

            // Entrar/salir de scopes múltiples veces
            for i in 0..10 {
                ctx.enter_scope();
                let var_name = format!("var_{}", i);
                let scheme = TypeScheme::mono(Type::Int);
                ctx.add_variable(var_name, scheme);

                // Verificar que la variable existe
                let _ = ctx.has_variable(&format!("var_{}", i));

                ctx.exit_scope().unwrap();
            }

            black_box(ctx);
        })
    });

    c.bench_function("context_variable_lookup", |b| {
        let mut ctx = TypeContext::new();

        // Agregar muchas variables
        for i in 0..100 {
            let var_name = format!("var_{}", i);
            let scheme = TypeScheme::mono(Type::Int);
            ctx.add_variable(var_name, scheme);
        }

        b.iter(|| {
            // Buscar variables aleatorias
            for i in (0..100).step_by(10) {
                let var_name = format!("var_{}", i);
                let _ = ctx.lookup_variable(&var_name);
            }
        })
    });
}

fn bench_type_scheme_operations(c: &mut Criterion) {
    c.bench_function("type_scheme_creation", |b| {
        b.iter(|| {
            // Crear esquemas monomórficos
            let mono_scheme = TypeScheme::mono(Type::Int);
            black_box(mono_scheme);

            // Crear esquemas polimórficos
            let vars = vec![TypeVar::fresh(), TypeVar::fresh()];
            let poly_scheme = TypeScheme::poly(vars, Type::Var(TypeVar::fresh()));
            black_box(poly_scheme);
        })
    });
}

fn bench_type_display(c: &mut Criterion) {
    let complex_type = Type::Function {
        params: vec![
            Type::Array(Box::new(Type::Int)),
            Type::Record({
                let mut fields = HashMap::new();
                fields.insert("name".to_string(), Type::String);
                fields.insert("items".to_string(), Type::Array(Box::new(Type::Bool)));
                fields
            }),
            Type::Option(Box::new(Type::Tuple(vec![Type::Float, Type::String]))),
        ],
        ret: Box::new(Type::Result {
            ok: Box::new(Type::Generic {
                name: "Vec".to_string(),
                args: vec![Type::Int],
            }),
            err: Box::new(Type::String),
        }),
    };

    c.bench_function("type_display_complex", |b| {
        b.iter(|| {
            let display = format!("{}", complex_type);
            black_box(display);
        })
    });
}

criterion_group!(
    benches,
    bench_simple,
    bench_type_context_creation,
    bench_type_var_creation,
    bench_type_operations,
    bench_context_operations,
    bench_type_scheme_operations,
    bench_type_display
);
criterion_main!(benches);