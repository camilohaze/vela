//! Load Balancer para API Gateway
//!
//! Implementa múltiples estrategias de balanceo de carga:
//! - Round-robin
//! - Least-connections
//! - Weighted random
//! - IP hash

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::gateway::GatewayError;

/// Estrategia de load balancing
#[derive(Debug, Clone, PartialEq)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRandom,
    IpHash,
}

/// Backend service endpoint
#[derive(Debug)]
pub struct Backend {
    pub url: String,
    pub weight: u32,
    pub connections: AtomicUsize,
    pub healthy: bool,
}

/// Load balancer principal
#[derive(Debug)]
pub struct LoadBalancer {
    backends: HashMap<String, Vec<Backend>>,
    strategy: LoadBalancingStrategy,
    round_robin_index: HashMap<String, AtomicUsize>,
}

impl LoadBalancer {
    /// Crear nuevo load balancer
    pub fn new() -> Self {
        Self {
            backends: HashMap::new(),
            strategy: LoadBalancingStrategy::RoundRobin,
            round_robin_index: HashMap::new(),
        }
    }

    /// Configurar estrategia de load balancing
    pub fn with_strategy(mut self, strategy: LoadBalancingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Agregar backends para un servicio
    pub fn add_backends(&mut self, service: String, backends: Vec<Backend>) {
        self.backends.insert(service.clone(), backends);
        self.round_robin_index.insert(service, AtomicUsize::new(0));
    }

    /// Seleccionar backend para un servicio
    pub fn select_backend(&mut self, service: &str) -> Result<String, GatewayError> {
        let backends = self.backends.get(service)
            .ok_or_else(|| GatewayError::ServiceUnavailable(format!("Service {} not found", service)))?;

        let healthy_backends: Vec<&Backend> = backends.iter()
            .filter(|b| b.healthy)
            .collect();

        if healthy_backends.is_empty() {
            return Err(GatewayError::ServiceUnavailable(format!("No healthy backends for service {}", service)));
        }

        let selected = match self.strategy {
            LoadBalancingStrategy::RoundRobin => self.select_round_robin(service, &healthy_backends),
            LoadBalancingStrategy::LeastConnections => self.select_least_connections(&healthy_backends),
            LoadBalancingStrategy::WeightedRandom => self.select_weighted_random(&healthy_backends),
            LoadBalancingStrategy::IpHash => {
                // Para IP hash necesitaríamos la IP del cliente
                // Por simplicidad usamos round-robin
                self.select_round_robin(service, &healthy_backends)
            }
        };

        // Incrementar contador de conexiones
        selected.connections.fetch_add(1, Ordering::SeqCst);

        // Incrementar índice de round-robin si se usó esa estrategia
        if matches!(self.strategy, LoadBalancingStrategy::RoundRobin | LoadBalancingStrategy::IpHash) {
            if let Some(index) = self.round_robin_index.get(service) {
                index.fetch_add(1, Ordering::SeqCst);
            }
        }

        Ok(selected.url.clone())
    }

    /// Seleccionar backend con round-robin
    fn select_round_robin<'a>(&self, service: &str, backends: &[&'a Backend]) -> &'a Backend {
        let index = self.round_robin_index.get(service).unwrap();
        let current = index.load(Ordering::SeqCst);
        // No modificamos aquí, lo haremos después
        &backends[current % backends.len()]
    }

    /// Seleccionar backend con menos conexiones
    fn select_least_connections<'a>(&self, backends: &[&'a Backend]) -> &'a Backend {
        backends.iter()
            .min_by_key(|b| b.connections.load(Ordering::SeqCst))
            .unwrap()
    }

    /// Seleccionar backend con peso aleatorio
    fn select_weighted_random<'a>(&self, backends: &[&'a Backend]) -> &'a Backend {
        use rand::Rng;

        let total_weight: u32 = backends.iter().map(|b| b.weight).sum();
        let mut rng = rand::thread_rng();
        let mut random = rng.gen_range(0..total_weight);

        for backend in backends {
            if random < backend.weight {
                return backend;
            }
            random -= backend.weight;
        }

        // Fallback al primero
        backends[0]
    }

    /// Marcar backend como healthy/unhealthy
    pub fn set_backend_health(&mut self, service: &str, url: &str, healthy: bool) {
        if let Some(backends) = self.backends.get_mut(service) {
            for backend in backends {
                if backend.url == url {
                    backend.healthy = healthy;
                    break;
                }
            }
        }
    }

    /// Decrementar contador de conexiones
    pub fn release_connection(&self, service: &str, url: &str) {
        if let Some(backends) = self.backends.get(service) {
            for backend in backends {
                if backend.url == url {
                    backend.connections.fetch_sub(1, Ordering::SeqCst);
                    break;
                }
            }
        }
    }

    /// Obtener estadísticas de backends
    pub fn get_stats(&self, service: &str) -> Option<Vec<(String, usize, bool)>> {
        self.backends.get(service).map(|backends| {
            backends.iter()
                .map(|b| (
                    b.url.clone(),
                    b.connections.load(Ordering::SeqCst),
                    b.healthy
                ))
                .collect()
        })
    }
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    fn create_test_backend(url: &str, weight: u32) -> Backend {
        Backend {
            url: url.to_string(),
            weight,
            connections: AtomicUsize::new(0),
            healthy: true,
        }
    }

    #[test]
    fn test_load_balancer_creation() {
        let lb = LoadBalancer::new();
        assert!(lb.backends.is_empty());
    }

    #[test]
    fn test_add_backends() {
        let mut lb = LoadBalancer::new();

        let backends = vec![
            create_test_backend("http://service1:8080", 1),
            create_test_backend("http://service2:8080", 1),
        ];

        lb.add_backends("test-service".to_string(), backends);

        assert_eq!(lb.backends.len(), 1);
        assert_eq!(lb.backends["test-service"].len(), 2);
    }

    #[test]
    fn test_round_robin_selection() {
        let mut lb = LoadBalancer::new().with_strategy(LoadBalancingStrategy::RoundRobin);

        let backends = vec![
            create_test_backend("http://service1:8080", 1),
            create_test_backend("http://service2:8080", 1),
        ];

        lb.add_backends("test-service".to_string(), backends);

        // Primera selección
        let selected1 = lb.select_backend("test-service").unwrap();
        assert!(selected1.contains("service1"));

        // Segunda selección
        let selected2 = lb.select_backend("test-service").unwrap();
        assert!(selected2.contains("service2"));

        // Tercera selección (debería volver al primero)
        let selected3 = lb.select_backend("test-service").unwrap();
        assert!(selected3.contains("service1"));
    }

    #[test]
    fn test_least_connections_selection() {
        let mut lb = LoadBalancer::new().with_strategy(LoadBalancingStrategy::LeastConnections);

        let backends = vec![
            create_test_backend("http://service1:8080", 1),
            create_test_backend("http://service2:8080", 1),
        ];

        // Simular conexiones existentes
        backends[0].connections.store(5, Ordering::SeqCst);
        backends[1].connections.store(2, Ordering::SeqCst);

        lb.add_backends("test-service".to_string(), backends);

        let selected = lb.select_backend("test-service").unwrap();
        assert!(selected.contains("service2")); // Debería elegir el que tiene menos conexiones
    }

    #[test]
    fn test_service_not_found() {
        let mut lb = LoadBalancer::new();

        let result = lb.select_backend("non-existent-service");
        assert!(result.is_err());
    }

    #[test]
    fn test_no_healthy_backends() {
        let mut lb = LoadBalancer::new();

        let mut backends = vec![create_test_backend("http://service1:8080", 1)];
        backends[0].healthy = false;

        lb.add_backends("test-service".to_string(), backends);

        let result = lb.select_backend("test-service");
        assert!(result.is_err());
    }

    #[test]
    fn test_backend_health_management() {
        let mut lb = LoadBalancer::new();

        let backends = vec![create_test_backend("http://service1:8080", 1)];
        lb.add_backends("test-service".to_string(), backends);

        // Marcar como unhealthy
        lb.set_backend_health("test-service", "http://service1:8080", false);

        let result = lb.select_backend("test-service");
        assert!(result.is_err()); // No debería haber backends healthy

        // Marcar como healthy de nuevo
        lb.set_backend_health("test-service", "http://service1:8080", true);

        let result = lb.select_backend("test-service");
        assert!(result.is_ok());
    }
}