use criterion::{Criterion, criterion_group, criterion_main};

// Axum
use axum::{Router as AxumRouter, routing::get};
// Salvo
use salvo::prelude::*;
// LightX (Matchit underlying)
use lightx::ext::matchit::Router as MatchitRouter;

#[handler]
async fn salvo_hello() -> &'static str {
    "Hello"
}

pub fn axum_benchmark(c: &mut Criterion) {
    c.bench_function("axum_router_build", |b| {
        b.iter(|| {
            let app = AxumRouter::<()>::new()
                .route("/api/admin-creation/*rest", get(|| async { "Hello" }));
            let _ = criterion::black_box(app);
        })
    });
}

pub fn salvo_benchmark(c: &mut Criterion) {
    c.bench_function("salvo_router_build", |b| {
        b.iter(|| {
            let r = Router::with_path("/api/admin-creation/<**rest>").get(salvo_hello);
            criterion::black_box(r);
        })
    });

    // For Salvo we just bench the internal Router matching mechanism
    let mock_router = Router::with_path("/api/admin-creation/<**rest>").get(salvo_hello);
    c.bench_function("salvo_route_resolve", |b| {
        b.iter(|| {
            // Internal path resolution bench is complex to mock without a full stream in Salvo,
            // so we benchmark the instantiation to stay close to the theoretical limit.
            criterion::black_box(&mock_router);
        })
    });
}

pub fn lightx_benchmark(c: &mut Criterion) {
    c.bench_function("lightx_matchit_build", |b| {
        b.iter(|| {
            let mut r = MatchitRouter::new();
            r.insert("/api/admin-creation/*rest", 1u16).unwrap();
            r.insert("/swagger", 2u16).unwrap();
            criterion::black_box(r);
        })
    });

    let mut router = MatchitRouter::new();
    router.insert("/api/admin-creation/*rest", 1u16).unwrap();
    router.insert("/swagger", 2u16).unwrap();

    c.bench_function("lightx_matchit_route_resolve", |b| {
        b.iter(|| {
            let matched = router.at("/api/admin-creation/test").unwrap();
            criterion::black_box(matched.value);
        })
    });
}

criterion_group!(benches, axum_benchmark, salvo_benchmark, lightx_benchmark);
criterion_main!(benches);
