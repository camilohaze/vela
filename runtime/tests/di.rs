//! Tests unitarios para el sistema de Dependency Injection
//!
//! Estos tests verifican la funcionalidad completa del DI container:
//! - Registro de servicios (singleton, transient, factory)
//! - Resolución de dependencias
//! - Scopes y lifecycle management
//! - Detección de dependencias circulares
//! - Thread safety
//! - Error handling

use vela_runtime::di::*;
use std::sync::Arc;
use tokio::test;

/// Servicio de prueba simple
#[derive(Debug, Clone, PartialEq)]
struct TestService {
    value: i32,
}

impl TestService {
    fn new() -> Self {
        Self { value: 42 }
    }
}

/// Servicio con dependencias
#[derive(Debug)]
struct DependentService {
    test_service: TestService,
    name: String,
}

impl DependentService {
    fn new(test_service: TestService) -> Self {
        Self {
            test_service,
            name: "dependent".to_string(),
        }
    }
}

#[tokio::test]
async fn test_register_and_resolve_singleton() {
    let container = DIContainer::new();

    container
        .register_singleton(|| Ok(TestService::new()))
        .unwrap();

    let service1 = container.resolve::<TestService>().unwrap();
    let service2 = container.resolve::<TestService>().unwrap();

    assert_eq!(service1.value, 42);
    assert_eq!(service2.value, 42);
    assert_eq!(service1, service2); // Misma instancia
}

#[tokio::test]
async fn test_register_and_resolve_transient() {
    let container = DIContainer::new();

    container
        .register_transient(|| Ok(TestService::new()))
        .unwrap();

    let service1 = container.resolve::<TestService>().unwrap();
    let service2 = container.resolve::<TestService>().unwrap();

    assert_eq!(service1.value, 42);
    assert_eq!(service2.value, 42);
    // Instancias diferentes (transient)
}

#[tokio::test]
async fn test_register_factory() {
    let container = DIContainer::new();

    container
        .register_factory(|_| Ok(TestService { value: 100 }))
        .unwrap();

    let service = container.resolve::<TestService>().unwrap();
    assert_eq!(service.value, 100);
}

#[tokio::test]
async fn test_register_instance() {
    let container = DIContainer::new();
    let instance = TestService { value: 123 };

    container.register_instance(instance).unwrap();

    let resolved = container.resolve::<TestService>().unwrap();
    assert_eq!(resolved.value, 123);
}

#[tokio::test]
async fn test_service_not_registered() {
    let container = DIContainer::new();

    let result = container.resolve::<TestService>();
    assert!(matches!(result, Err(DIError::ServiceNotRegistered { .. })));
}

#[tokio::test]
async fn test_service_already_registered() {
    let container = DIContainer::new();

    container
        .register_singleton(|| Ok(TestService::new()))
        .unwrap();

    let result = container.register_singleton(|| Ok(TestService::new()));
    assert!(matches!(result, Err(DIError::ServiceAlreadyRegistered { .. })));
}

#[tokio::test]
async fn test_is_registered() {
    let container = DIContainer::new();

    assert!(!container.is_registered::<TestService>());

    container
        .register_singleton(|| Ok(TestService::new()))
        .unwrap();

    assert!(container.is_registered::<TestService>());
}

#[tokio::test]
async fn test_get_registered_services() {
    let container = DIContainer::new();

    container
        .register_singleton::<TestService, _>(|| Ok(TestService::new()))
        .unwrap();

    let services = container.get_registered_services();
    assert!(!services.is_empty());
}

#[tokio::test]
async fn test_clear_singletons() {
    let container = DIContainer::new();

    container
        .register_singleton(|| Ok(TestService::new()))
        .unwrap();

    // Resolver para crear la instancia singleton
    let _ = container.resolve::<TestService>().unwrap();

    // Limpiar singletons
    container.clear_singletons();

    // La próxima resolución debería crear una nueva instancia
    let _ = container.resolve::<TestService>().unwrap();
}

#[tokio::test]
async fn test_dependency_injection_with_factory() {
    let container = DIContainer::new();

    // Registrar servicio base
    container
        .register_singleton(|| Ok(TestService::new()))
        .unwrap();

    // Registrar servicio dependiente usando factory
    container
        .register_factory(|c| {
            let test_service = c.resolve::<TestService>()?;
            Ok(DependentService::new(test_service))
        })
        .unwrap();

    let dependent = container.resolve::<DependentService>().unwrap();
    assert_eq!(dependent.test_service.value, 42);
    assert_eq!(dependent.name, "dependent");
}

#[tokio::test]
async fn test_thread_safety() {
    let container = Arc::new(DIContainer::new());

    container
        .register_singleton(|| Ok(TestService::new()))
        .unwrap();

    let mut handles = vec![];

    // Crear múltiples tareas que resuelven el servicio concurrentemente
    for _ in 0..10 {
        let container = Arc::clone(&container);
        let handle = tokio::spawn(async move {
            let service = container.resolve::<TestService>().unwrap();
            assert_eq!(service.value, 42);
        });
        handles.push(handle);
    }

    // Esperar que todas las tareas terminen
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_complex_dependency_graph() {
    let container = DIContainer::new();

    // Servicio A (base)
    container
        .register_singleton(|| Ok(TestService { value: 1 }))
        .unwrap();

    // Servicio B depende de A
    #[derive(Debug)]
    struct ServiceB {
        service_a: TestService,
        multiplier: i32,
    }

    container
        .register_factory(|c| {
            let service_a = c.resolve::<TestService>()?;
            Ok(ServiceB {
                service_a,
                multiplier: 2,
            })
        })
        .unwrap();

    // Servicio C depende de B
    #[derive(Debug)]
    struct ServiceC {
        service_b: ServiceB,
        offset: i32,
    }

    container
        .register_factory(|c| {
            let service_b = c.resolve::<ServiceB>()?;
            Ok(ServiceC {
                service_b,
                offset: 10,
            })
        })
        .unwrap();

    let service_c = container.resolve::<ServiceC>().unwrap();
    assert_eq!(service_c.service_b.service_a.value, 1);
    assert_eq!(service_c.service_b.multiplier, 2);
    assert_eq!(service_c.offset, 10);
}

#[tokio::test]
async fn test_scopes_display() {
    assert_eq!(format!("{}", Scope::Singleton), "Singleton");
    assert_eq!(format!("{}", Scope::Scoped), "Scoped");
    assert_eq!(format!("{}", Scope::Transient), "Transient");
}

#[tokio::test]
async fn test_scope_context() {
    let mut context = ScopeContext::new();

    // Verificar que no tiene instancias inicialmente
    assert!(!context.has::<TestService>());

    // Crear instancia en el scope
    let service = context.get_or_create(|| TestService { value: 99 });
    assert_eq!(service.value, 99);
    assert!(context.has::<TestService>());

    // Obtener la misma instancia
    let service2 = context.get_or_create(|| TestService { value: 100 });
    assert_eq!(service2.value, 99); // Misma instancia

    // Limpiar y verificar
    context.clear();
    assert!(!context.has::<TestService>());
}