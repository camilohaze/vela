use vela_compiler::gateway::*;
use vela_compiler::rate_limiter::*;
use vela_compiler::load_balancer::*;
use vela_compiler::router::*;
use vela_compiler::dynamic_router::*;
use vela_compiler::plugins::*;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    println!("🚀 Ejecutando tests del API Gateway...");

    // Test básico del rate limiter
    println!("📊 Testing Rate Limiter...");
    let rate_limiter = Arc::new(RwLock::new(RateLimiter::new(10, 60)));
    let mut limiter = rate_limiter.write().await;

    // Test rate limiting básico
    let key = "test_ip";
    for i in 0..12 {
        let allowed = limiter.check_rate_limit(key).await;
        if i < 10 {
            assert!(allowed, "Request {} should be allowed", i);
        } else {
            assert!(!allowed, "Request {} should be blocked", i);
        }
    }
    println!("✅ Rate Limiter tests passed");

    // Test básico del load balancer
    println!("⚖️ Testing Load Balancer...");
    let mut load_balancer = LoadBalancer::new();
    load_balancer.add_backend("http://backend1:8080".to_string());
    load_balancer.add_backend("http://backend2:8080".to_string());

    for _ in 0..10 {
        let backend = load_balancer.get_next_backend();
        assert!(backend.is_some(), "Should get a backend");
        assert!(backend.unwrap().starts_with("http://backend"), "Backend URL should be valid");
    }
    println!("✅ Load Balancer tests passed");

    // Test básico del router
    println!("🛣️ Testing Router...");
    let mut router = Router::new();
    router.add_route("/api/users".to_string(), "GET".to_string(), "users_service".to_string());
    router.add_route("/api/orders".to_string(), "POST".to_string(), "orders_service".to_string());

    let route1 = router.match_route("/api/users", "GET");
    assert_eq!(route1, Some("users_service".to_string()));

    let route2 = router.match_route("/api/orders", "POST");
    assert_eq!(route2, Some("orders_service".to_string()));

    let route3 = router.match_route("/api/unknown", "GET");
    assert_eq!(route3, None);
    println!("✅ Router tests passed");

    println!("🎉 All basic gateway tests passed!");
}