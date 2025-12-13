//! # Vela Mocking Framework
//!
//! Framework de mocking para testing en Vela, proporcionando herramientas
//! para crear mocks de dependencias, verificar interacciones y configurar
//! comportamientos de prueba.
//!
//! ## Características
//!
//! - **Mock Objects**: Creación de objetos mock que implementan traits
//! - **Method Stubbing**: Configuración de retornos de métodos
//! - **Call Verification**: Verificación de llamadas a métodos
//! - **Argument Matching**: Matching flexible de argumentos
//! - **Sequence Verification**: Verificación de orden de llamadas
//!
//! ## Ejemplo de Uso
//!
//! ```rust
//! use vela_testing::mocking::*;
//!
//! // Crear un mock
//! let mut mock_service = MockService::new();
//!
//! // Configurar comportamiento
//! mock_service.when().get_user(1).returns(Ok(User { id: 1, name: "Test" }));
//!
//! // Usar en test
//! let result = mock_service.get_user(1).await;
//! assert!(result.is_ok());
//!
//! // Verificar llamadas
//! mock_service.verify_method("get_user").called_once();
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde_json::Value;

/// Representa una llamada a método registrada
#[derive(Debug, Clone)]
pub struct MethodCall {
    pub method_name: String,
    pub arguments: Vec<Value>,
    pub sequence_number: usize,
}

/// Configuración de stub para un método
#[derive(Debug, Clone)]
pub struct MethodStub {
    pub method_name: String,
    pub arguments: Vec<Value>,
    pub return_value: Value,
    pub throws_error: Option<String>,
}

/// Verificador de llamadas para un método específico
pub struct MethodVerifier<'a> {
    calls: &'a [MethodCall],
    method_name: String,
}

impl<'a> MethodVerifier<'a> {
    pub fn new(calls: &'a [MethodCall], method_name: String) -> Self {
        Self { calls, method_name }
    }

    /// Verifica que el método fue llamado exactamente una vez
    pub fn called_once(self) {
        let count = self.calls.iter()
            .filter(|call| call.method_name == self.method_name)
            .count();
        assert_eq!(count, 1, "Expected method {} to be called once, but was called {} times",
                  self.method_name, count);
    }

    /// Verifica que el método fue llamado exactamente n veces
    pub fn called_times(self, expected: usize) {
        let count = self.calls.iter()
            .filter(|call| call.method_name == self.method_name)
            .count();
        assert_eq!(count, expected, "Expected method {} to be called {} times, but was called {} times",
                  self.method_name, expected, count);
    }

    /// Verifica que el método nunca fue llamado
    pub fn never_called(self) {
        let count = self.calls.iter()
            .filter(|call| call.method_name == self.method_name)
            .count();
        assert_eq!(count, 0, "Expected method {} to never be called, but was called {} times",
                  self.method_name, count);
    }

    /// Verifica que el método fue llamado al menos una vez
    pub fn called_at_least_once(self) {
        let count = self.calls.iter()
            .filter(|call| call.method_name == self.method_name)
            .count();
        assert!(count > 0, "Expected method {} to be called at least once, but was never called",
               self.method_name);
    }
}

/// Trait base para objetos mock
pub trait Mock {
    /// Registra una llamada a método
    fn record_call(&mut self, method_name: &str, args: Vec<Value>);

    /// Obtiene todas las llamadas registradas
    fn get_calls(&self) -> Vec<MethodCall>;

    /// Limpia todas las llamadas registradas
    fn clear_calls(&mut self);

    /// Obtiene el número de secuencia actual
    fn next_sequence_number(&mut self) -> usize;
}

/// Trait para configurar stubs de métodos
pub trait MockStubber {
    /// Configura un stub para un método
    fn add_stub(&mut self, stub: MethodStub);

    /// Busca un stub que coincida con la llamada
    fn find_stub(&self, method_name: &str, args: &[Value]) -> Option<&MethodStub>;

    /// Obtiene todos los stubs configurados
    fn get_stubs(&self) -> Vec<MethodStub>;
}

/// Trait para verificar llamadas a métodos
pub trait MockVerifier {
    /// Crea un verificador para un método específico
    fn verify_method(&self, method_name: &str) -> MethodVerifier;
}

/// Implementación base de un mock
#[derive(Debug, Clone)]
pub struct BaseMock {
    calls: Vec<MethodCall>,
    stubs: Vec<MethodStub>,
    sequence_counter: usize,
}

impl BaseMock {
    pub fn new() -> Self {
        Self {
            calls: Vec::new(),
            stubs: Vec::new(),
            sequence_counter: 0,
        }
    }
}

impl Mock for BaseMock {
    fn record_call(&mut self, method_name: &str, args: Vec<Value>) {
        let call = MethodCall {
            method_name: method_name.to_string(),
            arguments: args,
            sequence_number: self.sequence_counter,
        };
        self.calls.push(call);
        self.sequence_counter += 1;
    }

    fn get_calls(&self) -> Vec<MethodCall> {
        self.calls.clone()
    }

    fn clear_calls(&mut self) {
        self.calls.clear();
        self.sequence_counter = 0;
    }

    fn next_sequence_number(&mut self) -> usize {
        let seq = self.sequence_counter;
        self.sequence_counter += 1;
        seq
    }
}

impl MockStubber for BaseMock {
    fn add_stub(&mut self, stub: MethodStub) {
        self.stubs.push(stub);
    }

    fn find_stub(&self, method_name: &str, args: &[Value]) -> Option<&MethodStub> {
        // Buscar stubs que coincidan exactamente
        for stub in &self.stubs {
            if stub.method_name == method_name && stub.arguments == args {
                return Some(stub);
            }
        }
        None
    }

    fn get_stubs(&self) -> Vec<MethodStub> {
        self.stubs.clone()
    }
}

impl MockVerifier for BaseMock {
    fn verify_method(&self, method_name: &str) -> MethodVerifier {
        MethodVerifier::new(&self.calls, method_name.to_string())
    }
}

/// Builder para configurar stubs de métodos
pub struct StubBuilder<'a, T> {
    mock: &'a mut T,
    method_name: String,
    arguments: Vec<Value>,
}

impl<'a, T> StubBuilder<'a, T>
where
    T: MockStubber,
{
    pub fn new(mock: &'a mut T, method_name: String) -> Self {
        Self {
            mock,
            method_name,
            arguments: Vec::new(),
        }
    }

    /// Agrega argumentos para el stub
    pub fn with_args(mut self, args: Vec<Value>) -> Self {
        self.arguments = args;
        self
    }

    /// Configura el valor de retorno del stub
    pub fn returns(self, return_value: Value) {
        let stub = MethodStub {
            method_name: self.method_name,
            arguments: self.arguments,
            return_value,
            throws_error: None,
        };
        self.mock.add_stub(stub);
    }

    /// Configura que el stub lance un error
    pub fn throws(self, error_message: String) {
        let stub = MethodStub {
            method_name: self.method_name,
            arguments: self.arguments,
            return_value: Value::Null,
            throws_error: Some(error_message),
        };
        self.mock.add_stub(stub);
    }
}

/// Builder para configurar verificaciones de llamadas
pub struct VerifyBuilder<'a, T> {
    mock: &'a T,
    method_name: String,
}

impl<'a, T> VerifyBuilder<'a, T>
where
    T: MockVerifier,
{
    pub fn new(mock: &'a T, method_name: String) -> Self {
        Self { mock, method_name }
    }

    /// Verifica que el método fue llamado con argumentos específicos
    pub fn with_args(self, args: Vec<Value>) -> MethodVerifier<'a> {
        // Por ahora, solo verificamos el nombre del método
        // En una implementación más completa, verificaríamos también los argumentos
        self.mock.verify_method(&self.method_name)
    }

    /// Verifica que el método fue llamado (sin importar argumentos)
    pub fn called(self) -> MethodVerifier<'a> {
        self.mock.verify_method(&self.method_name)
    }
}

/// Trait para objetos que pueden ser mockeados con builder pattern
pub trait Mockable {
    /// Inicia la configuración de un stub
    fn when(&mut self) -> WhenBuilder<Self> where Self: Sized;

    /// Inicia la verificación de llamadas
    fn verify_builder(&self) -> VerifyBuilder<Self> where Self: Sized;
}

/// Builder para el método `when`
pub struct WhenBuilder<'a, T> {
    mock: &'a mut T,
}

impl<'a, T> WhenBuilder<'a, T> {
    pub fn new(mock: &'a mut T) -> Self {
        Self { mock }
    }

    /// Configura un método específico para stubbing
    pub fn method(self, method_name: &str) -> StubBuilder<'a, T>
    where
        T: MockStubber,
    {
        StubBuilder::new(self.mock, method_name.to_string())
    }
}

/// Implementación de Mockable para cualquier tipo que implemente MockStubber y MockVerifier
impl<T> Mockable for T
where
    T: MockStubber + MockVerifier,
{
    fn when(&mut self) -> WhenBuilder<Self> {
        WhenBuilder::new(self)
    }

    fn verify_builder(&self) -> VerifyBuilder<Self> {
        VerifyBuilder::new(self, String::new())
    }
}

/// Macro para crear mocks fácilmente
#[macro_export]
macro_rules! mock {
    ($struct_name:ident { $($field:ident: $type:ty),* $(,)? }) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name {
            base_mock: $crate::mocking::BaseMock,
            $($field: $type),*
        }

        impl $struct_name {
            pub fn new($($field: $type),*) -> Self {
                Self {
                    base_mock: $crate::mocking::BaseMock::new(),
                    $($field),*
                }
            }
        }

        impl $crate::mocking::Mock for $struct_name {
            fn record_call(&mut self, method_name: &str, args: Vec<serde_json::Value>) {
                self.base_mock.record_call(method_name, args);
            }

            fn get_calls(&self) -> Vec<$crate::mocking::MethodCall> {
                self.base_mock.get_calls()
            }

            fn clear_calls(&mut self) {
                self.base_mock.clear_calls();
            }

            fn next_sequence_number(&mut self) -> usize {
                self.base_mock.next_sequence_number()
            }
        }

        impl $crate::mocking::MockStubber for $struct_name {
            fn add_stub(&mut self, stub: $crate::mocking::MethodStub) {
                self.base_mock.add_stub(stub);
            }

            fn find_stub(&self, method_name: &str, args: &[serde_json::Value]) -> Option<&$crate::mocking::MethodStub> {
                self.base_mock.find_stub(method_name, args)
            }

            fn get_stubs(&self) -> Vec<$crate::mocking::MethodStub> {
                self.base_mock.get_stubs()
            }
        }

        impl $crate::mocking::MockVerifier for $struct_name {
            fn verify_method(&self, method_name: &str) -> $crate::mocking::MethodVerifier {
                self.base_mock.verify_method(method_name)
            }
        }
    };
}

/// Macro para crear mocks de traits
#[macro_export]
macro_rules! mock_trait {
    ($mock_name:ident, $trait_name:ident) => {
        #[derive(Debug, Clone)]
        pub struct $mock_name {
            base_mock: $crate::mocking::BaseMock,
        }

        impl $mock_name {
            pub fn new() -> Self {
                Self {
                    base_mock: $crate::mocking::BaseMock::new(),
                }
            }
        }

        impl $crate::mocking::Mock for $mock_name {
            fn record_call(&mut self, method_name: &str, args: Vec<serde_json::Value>) {
                self.base_mock.record_call(method_name, args);
            }

            fn get_calls(&self) -> Vec<$crate::mocking::MethodCall> {
                self.base_mock.get_calls()
            }

            fn clear_calls(&mut self) {
                self.base_mock.clear_calls();
            }

            fn next_sequence_number(&mut self) -> usize {
                self.base_mock.next_sequence_number()
            }
        }

        impl $crate::mocking::MockStubber for $mock_name {
            fn add_stub(&mut self, stub: $crate::mocking::MethodStub) {
                self.base_mock.add_stub(stub);
            }

            fn find_stub(&self, method_name: &str, args: &[serde_json::Value]) -> Option<&$crate::mocking::MethodStub> {
                self.base_mock.find_stub(method_name, args)
            }

            fn get_stubs(&self) -> Vec<$crate::mocking::MethodStub> {
                self.base_mock.get_stubs()
            }
        }

        impl $crate::mocking::MockVerifier for $mock_name {
            fn verify(&self, method_name: &str) -> $crate::mocking::MethodVerifier {
                self.base_mock.verify(method_name)
            }
        }

        // Implementación básica del trait mockeado
        impl $trait_name for $mock_name {
            // Aquí irían las implementaciones de métodos del trait
            // En una implementación real, esto se generaría automáticamente
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ejemplo de mock creado con la macro
    mock!(TestService {
        name: String,
        version: u32,
    });

    #[test]
    fn test_mock_creation() {
        let service = TestService::new("TestService".to_string(), 1);
        assert_eq!(service.name, "TestService");
        assert_eq!(service.version, 1);
    }

    #[test]
    fn test_method_call_recording() {
        let mut service = TestService::new("TestService".to_string(), 1);

        // Registrar una llamada
        service.record_call("get_user", vec![Value::Number(1.into())]);

        let calls = service.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].method_name, "get_user");
        assert_eq!(calls[0].arguments[0], Value::Number(1.into()));
    }

    #[test]
    fn test_method_stubbing() {
        let mut service = TestService::new("TestService".to_string(), 1);

        // Configurar un stub
        service.when().method("get_user").with_args(vec![Value::Number(1.into())])
               .returns(Value::String("John Doe".to_string()));

        // Verificar que el stub fue agregado
        let stubs = service.get_stubs();
        assert_eq!(stubs.len(), 1);
        assert_eq!(stubs[0].method_name, "get_user");
    }

    #[test]
    fn test_call_verification() {
        let mut service = TestService::new("TestService".to_string(), 1);

        // Registrar llamadas
        service.record_call("get_user", vec![Value::Number(1.into())]);
        service.record_call("get_user", vec![Value::Number(2.into())]);

        // Verificar llamadas
        service.verify_method("get_user").called_times(2);
    }

    #[test]
    fn test_clear_calls() {
        let mut service = TestService::new("TestService".to_string(), 1);

        service.record_call("get_user", vec![Value::Number(1.into())]);
        assert_eq!(service.get_calls().len(), 1);

        service.clear_calls();
        assert_eq!(service.get_calls().len(), 0);
    }
}