# TASK-113CE: Implementar code generation desde .proto

## 📋 Información General
- **Historia:** VELA-113CC
- **Estado:** En curso 🚧
- **Fecha:** 2025-12-12

## 🎯 Objetivo
Permitir la generación automática de código Rust a partir de archivos `.proto` en el pipeline de gRPC de Vela, integrando el codegen en el flujo de compilación y runtime.

## 🔨 Implementación
- Integración de codegen Rust desde archivos `.proto` usando prost/tonic en el pipeline de Vela.
- Soporte para generación de structs, traits y servicios a partir de definiciones protobuf.
- Ejemplo de uso y tests unitarios.

### Archivos a modificar
- `compiler/src/grpc_decorators.rs` - Lógica de integración codegen
- `compiler/src/grpc_decorator_tests.rs` - Tests unitarios

## ✅ Criterios de Aceptación
- [ ] Se puede generar código Rust desde un archivo `.proto` dado
- [ ] El pipeline de Vela integra el codegen en la compilación
- [ ] Tests unitarios para codegen
- [ ] Documentación generada

## 🔗 Referencias
- **Jira:** [TASK-113CE](https://velalang.atlassian.net/browse/VELA-113CC)
- **Historia:** [VELA-113CC](https://velalang.atlassian.net/browse/VELA-113CC)
