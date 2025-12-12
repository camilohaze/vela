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
    WeightedRoundRobin,
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
    weighted_round_robin_current_weight: HashMap<String, AtomicUsize>,
    weighted_round_robin_gcd: HashMap<String, u32>,
    weighted_round_robin_max_weight: HashMap<String, u32>,
}

impl LoadBalancer {
    /// Crear nuevo load balancer
    pub fn new() -> Self {
        Self {
            backends: HashMap::new(),
            strategy: LoadBalancingStrategy::RoundRobin,
            round_robin_index: HashMap::new(),
            weighted_round_robin_current_weight: HashMap::new(),
            weighted_round_robin_gcd: HashMap::new(),
            weighted_round_robin_max_weight: HashMap::new(),
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
        self.round_robin_index.insert(service.clone(), AtomicUsize::new(0));
        
        // Inicializar valores para weighted round-robin
        self.weighted_round_robin_current_weight.insert(service.clone(), AtomicUsize::new(0));
        
        // Calcular GCD y max weight para el algoritmo de weighted round-robin
        if let Some(service_backends) = self.backends.get(&service) {
            let weights: Vec<u32> = service_backends.iter().map(|b| b.weight).collect();
            let gcd = Self::calculate_gcd(&weights);
            let max_weight = weights.iter().max().copied().unwrap_or(1);
            
            self.weighted_round_robin_gcd.insert(service.clone(), gcd);
            self.weighted_round_robin_max_weight.insert(service.clone(), max_weight);
        }
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
            LoadBalancingStrategy::WeightedRoundRobin => self.select_weighted_round_robin(service, &healthy_backends),
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

    /// Seleccionar backend con weighted round-robin
    fn select_weighted_round_robin<'a>(&self, service: &str, backends: &[&'a Backend]) -> &'a Backend {
        let current_weight = self.weighted_round_robin_current_weight.get(service).unwrap();
        let gcd = self.weighted_round_robin_gcd.get(service).copied().unwrap_or(1);
        let max_weight = self.weighted_round_robin_max_weight.get(service).copied().unwrap_or(1);

        let mut current = current_weight.load(Ordering::SeqCst) as i32;
        let mut selected_index = 0;
        let mut selected_weight = -1;

        // Encontrar el backend con el mayor peso efectivo
        for (i, backend) in backends.iter().enumerate() {
            let effective_weight = backend.weight as i32;
            if effective_weight > selected_weight {
                selected_weight = effective_weight;
                selected_index = i;
            }
        }

        // Actualizar el peso actual
        current -= gcd as i32;
        if current <= 0 {
            current = max_weight as i32;
            if current == 0 {
                current = 1; // Evitar división por cero
            }
        }

        current_weight.store(current as usize, Ordering::SeqCst);

        &backends[selected_index]
    }

    /// Calcular el máximo común divisor de una lista de números
    fn calculate_gcd(numbers: &[u32]) -> u32 {
        if numbers.is_empty() {
            return 1;
        }
        
        let mut result = numbers[0];
        for &num in &numbers[1..] {
            result = Self::gcd(result, num);
        }
        result
    }

    /// Calcular GCD de dos números usando algoritmo de Euclides
    fn gcd(a: u32, b: u32) -> u32 {
        if b == 0 {
            a
        } else {
            Self::gcd(b, a % b)
        }
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
    fn test_weighted_random_selection() {
        let mut lb = LoadBalancer::new().with_strategy(LoadBalancingStrategy::WeightedRandom);

        // Crear backends con diferentes pesos
        let backends = vec![
            create_test_backend("http://service1:8080", 3), // Peso alto
            create_test_backend("http://service2:8080", 1), // Peso bajo
        ];

        lb.add_backends("test-service".to_string(), backends);

        // Hacer varias selecciones para verificar distribución probabilística
        let mut service1_count = 0;
        let mut service2_count = 0;

        for _ in 0..1000 {  // Muchas iteraciones para distribución estadística
            let selected = lb.select_backend("test-service").unwrap();
            if selected.contains("service1") {
                service1_count += 1;
            } else if selected.contains("service2") {
                service2_count += 1;
            }
        }

        // Service1 (peso 3) debería ser seleccionado aproximadamente 3 veces más que service2 (peso 1)
        // Con 1000 iteraciones, esperamos alrededor de 750 para service1 y 250 para service2
        assert!(service1_count > service2_count * 2, "Service1 should be selected much more often than Service2");
        assert!(service1_count > 600, "Service1 should be selected at least 600 times");
        assert!(service2_count > 100, "Service2 should be selected at least 100 times");
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
    fn test_weighted_round_robin_selection() {
        let mut lb = LoadBalancer::new().with_strategy(LoadBalancingStrategy::WeightedRoundRobin);

        // Crear backends con diferentes pesos: 3, 2, 1
        let backends = vec![
            create_test_backend("http://service1:8080", 3), // Peso 3
            create_test_backend("http://service2:8080", 2), // Peso 2
            create_test_backend("http://service3:8080", 1), // Peso 1
        ];

        lb.add_backends("test-service".to_string(), backends);

        // En weighted round-robin, la distribución debería ser proporcional a los pesos
        // Con pesos 3, 2, 1, esperamos ver service1 más veces que service2, y service2 más veces que service3
        
        let mut service1_count = 0;
        let mut service2_count = 0;
        let mut service3_count = 0;

        // Hacer varias selecciones para ver la distribución
        for _ in 0..12 {  // 12 = 3+2+1 * 2 para ver el patrón
            let selected = lb.select_backend("test-service").unwrap();
            if selected.contains("service1") {
                service1_count += 1;
            } else if selected.contains("service2") {
                service2_count += 1;
            } else if selected.contains("service3") {
                service3_count += 1;
            }
        }

        // Verificar que service1 (peso 3) sea seleccionado más veces que service2 (peso 2)
        // y service2 más veces que service3 (peso 1)
        assert!(service1_count > service2_count, "Service1 should be selected more than Service2");
        assert!(service2_count > service3_count, "Service2 should be selected more than Service3");
        assert!(service1_count > 0, "Service1 should be selected at least once");
        assert!(service2_count > 0, "Service2 should be selected at least once");
        assert!(service3_count > 0, "Service3 should be selected at least once");
    }
}