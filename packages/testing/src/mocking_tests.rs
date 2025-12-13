//! Tests unitarios para el mocking framework de Vela
//!
//! Estos tests verifican la funcionalidad completa del framework de mocking,
//! incluyendo creación de mocks, stubbing, verificación de llamadas y macros.

use super::mocking::*;
use serde_json::Value;

mod mocking_tests {
    use super::*;

    // Definir la macro mock! localmente para los tests
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

    // Mock de ejemplo para testing
    mock!(ExampleService {
        name: String,
        version: u32,
    });

    #[tokio::test]
    async fn test_mock_creation() {
        let service = ExampleService::new("TestService".to_string(), 1);
        assert_eq!(service.name, "TestService");
        assert_eq!(service.version, 1);
        assert_eq!(service.get_calls().len(), 0);
    }

    #[tokio::test]
    async fn test_method_call_recording() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        // Registrar una llamada
        service.record_call("get_user", vec![Value::Number(1.into())]);

        let calls = service.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].method_name, "get_user");
        assert_eq!(calls[0].arguments.len(), 1);
        assert_eq!(calls[0].arguments[0], Value::Number(1.into()));
        assert_eq!(calls[0].sequence_number, 0);
    }

    #[tokio::test]
    async fn test_multiple_method_calls() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        // Registrar múltiples llamadas
        service.record_call("get_user", vec![Value::Number(1.into())]);
        service.record_call("save_user", vec![Value::String("John".to_string())]);
        service.record_call("get_user", vec![Value::Number(2.into())]);

        let calls = service.get_calls();
        assert_eq!(calls.len(), 3);

        // Verificar primera llamada
        assert_eq!(calls[0].method_name, "get_user");
        assert_eq!(calls[0].sequence_number, 0);

        // Verificar segunda llamada
        assert_eq!(calls[1].method_name, "save_user");
        assert_eq!(calls[1].sequence_number, 1);

        // Verificar tercera llamada
        assert_eq!(calls[2].method_name, "get_user");
        assert_eq!(calls[2].sequence_number, 2);
    }

    #[tokio::test]
    async fn test_method_stubbing() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        // Configurar un stub
        service.when().method("get_user")
               .with_args(vec![Value::Number(1.into())])
               .returns(Value::String("John Doe".to_string()));

        // Verificar que el stub fue agregado
        let stubs = service.get_stubs();
        assert_eq!(stubs.len(), 1);

        let stub = &stubs[0];
        assert_eq!(stub.method_name, "get_user");
        assert_eq!(stub.arguments.len(), 1);
        assert_eq!(stub.arguments[0], Value::Number(1.into()));
        assert_eq!(stub.return_value, Value::String("John Doe".to_string()));
        assert!(stub.throws_error.is_none());
    }

    #[tokio::test]
    async fn test_stub_with_error() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        // Configurar un stub que lanza error
        service.when().method("delete_user")
               .with_args(vec![Value::Number(999.into())])
               .throws("User not found".to_string());

        let stubs = service.get_stubs();
        assert_eq!(stubs.len(), 1);

        let stub = &stubs[0];
        assert_eq!(stub.method_name, "delete_user");
        assert_eq!(stub.throws_error, Some("User not found".to_string()));
    }

    #[tokio::test]
    async fn test_find_stub() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        // Configurar stubs
        service.when().method("get_user")
               .with_args(vec![Value::Number(1.into())])
               .returns(Value::String("John".to_string()));

        service.when().method("get_user")
               .with_args(vec![Value::Number(2.into())])
               .returns(Value::String("Jane".to_string()));

        // Encontrar stub que coincide
        let stub1 = service.find_stub("get_user", &[Value::Number(1.into())]);
        assert!(stub1.is_some());
        assert_eq!(stub1.unwrap().return_value, Value::String("John".to_string()));

        let stub2 = service.find_stub("get_user", &[Value::Number(2.into())]);
        assert!(stub2.is_some());
        assert_eq!(stub2.unwrap().return_value, Value::String("Jane".to_string()));

        // No encontrar stub que no existe
        let no_stub = service.find_stub("get_user", &[Value::Number(3.into())]);
        assert!(no_stub.is_none());

        let wrong_method = service.find_stub("save_user", &[Value::Number(1.into())]);
        assert!(wrong_method.is_none());
    }

    #[tokio::test]
    async fn test_call_verification_called_once() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        service.record_call("get_user", vec![Value::Number(1.into())]);

        // Esto debería pasar
        service.verify_method("get_user").called_once();
    }

    #[tokio::test]
    #[should_panic(expected = "Expected method get_user to be called once, but was called 0 times")]
    async fn test_call_verification_called_once_fails_when_never_called() {
        let service = ExampleService::new("TestService".to_string(), 1);

        // Esto debería fallar porque nunca se llamó
        service.verify_method("get_user").called_once();
    }

    #[tokio::test]
    #[should_panic(expected = "Expected method get_user to be called once, but was called 2 times")]
    async fn test_call_verification_called_once_fails_when_called_twice() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        service.record_call("get_user", vec![Value::Number(1.into())]);
        service.record_call("get_user", vec![Value::Number(2.into())]);

        // Esto debería fallar porque se llamó 2 veces
        service.verify_method("get_user").called_once();
    }

    #[tokio::test]
    async fn test_call_verification_called_times() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        service.record_call("get_user", vec![Value::Number(1.into())]);
        service.record_call("get_user", vec![Value::Number(2.into())]);
        service.record_call("get_user", vec![Value::Number(3.into())]);

        // Verificar que se llamó exactamente 3 veces
        service.verify_method("get_user").called_times(3);
    }

    #[tokio::test]
    #[should_panic(expected = "Expected method get_user to be called 2 times, but was called 3 times")]
    async fn test_call_verification_called_times_fails() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        service.record_call("get_user", vec![Value::Number(1.into())]);
        service.record_call("get_user", vec![Value::Number(2.into())]);
        service.record_call("get_user", vec![Value::Number(3.into())]);

        // Esto debería fallar porque se llamó 3 veces, no 2
        service.verify_method("get_user").called_times(2);
    }

    #[tokio::test]
    async fn test_call_verification_never_called() {
        let service = ExampleService::new("TestService".to_string(), 1);

        // Esto debería pasar porque nunca se llamó
        service.verify_method("get_user").never_called();
    }

    #[tokio::test]
    #[should_panic(expected = "Expected method get_user to never be called, but was called 1 times")]
    async fn test_call_verification_never_called_fails() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        service.record_call("get_user", vec![Value::Number(1.into())]);

        // Esto debería fallar porque se llamó 1 vez
        service.verify_method("get_user").never_called();
    }

    #[tokio::test]
    async fn test_call_verification_called_at_least_once() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        service.record_call("get_user", vec![Value::Number(1.into())]);
        service.record_call("get_user", vec![Value::Number(2.into())]);

        // Esto debería pasar porque se llamó al menos 1 vez
        service.verify_method("get_user").called_at_least_once();
    }

    #[tokio::test]
    #[should_panic(expected = "Expected method get_user to be called at least once, but was never called")]
    async fn test_call_verification_called_at_least_once_fails() {
        let service = ExampleService::new("TestService".to_string(), 1);

        // Esto debería fallar porque nunca se llamó
        service.verify_method("get_user").called_at_least_once();
    }

    #[tokio::test]
    async fn test_clear_calls() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        service.record_call("get_user", vec![Value::Number(1.into())]);
        service.record_call("save_user", vec![Value::String("John".to_string())]);

        assert_eq!(service.get_calls().len(), 2);

        service.clear_calls();

        assert_eq!(service.get_calls().len(), 0);

        // Verificar que el sequence_counter se reinicia
        service.record_call("get_user", vec![Value::Number(2.into())]);
        let calls = service.get_calls();
        assert_eq!(calls[0].sequence_number, 0);
    }

    #[tokio::test]
    async fn test_sequence_number_increment() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        assert_eq!(service.next_sequence_number(), 0);
        assert_eq!(service.next_sequence_number(), 1);
        assert_eq!(service.next_sequence_number(), 2);
    }

    #[tokio::test]
    async fn test_multiple_stubs_same_method() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        // Configurar múltiples stubs para el mismo método con diferentes argumentos
        service.when().method("process_data")
               .with_args(vec![Value::String("input1".to_string())])
               .returns(Value::String("output1".to_string()));

        service.when().method("process_data")
               .with_args(vec![Value::String("input2".to_string())])
               .returns(Value::String("output2".to_string()));

        service.when().method("process_data")
               .with_args(vec![Value::String("error_input".to_string())])
               .throws("Processing error".to_string());

        let stubs = service.get_stubs();
        assert_eq!(stubs.len(), 3);

        // Todos los stubs deberían tener el mismo nombre de método
        for stub in &stubs {
            assert_eq!(stub.method_name, "process_data");
        }
    }

    #[tokio::test]
    async fn test_complex_arguments() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        // Crear argumentos complejos
        let complex_args = vec![
            Value::Number(42.into()),
            Value::String("test".to_string()),
            Value::Bool(true),
            Value::Array(vec![Value::Number(1.into()), Value::Number(2.into())]),
        ];

        service.record_call("complex_method", complex_args.clone());

        let calls = service.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].method_name, "complex_method");
        assert_eq!(calls[0].arguments, complex_args);
    }

    #[tokio::test]
    async fn test_empty_arguments() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        service.record_call("no_args_method", vec![]);

        let calls = service.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].method_name, "no_args_method");
        assert_eq!(calls[0].arguments.len(), 0);
    }

    #[tokio::test]
    async fn test_different_methods_verification() {
        let mut service = ExampleService::new("TestService".to_string(), 1);

        service.record_call("method_a", vec![Value::Number(1.into())]);
        service.record_call("method_b", vec![Value::Number(2.into())]);
        service.record_call("method_a", vec![Value::Number(3.into())]);

        // Verificar method_a se llamó 2 veces
        service.verify_method("method_a").called_times(2);

        // Verificar method_b se llamó 1 vez
        service.verify_method("method_b").called_once();

        // Verificar method_c nunca se llamó
        service.verify_method("method_c").never_called();
    }
}