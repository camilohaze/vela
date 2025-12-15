# TASK-173: Implementar function inlining

## 📋 Información General
- **Historia:** VELA-1184
- **Estado:** En curso ✅
- **Fecha:** 2025-12-15

## 🎯 Objetivo
Implementar optimización de function inlining en el compilador Vela para reducir overhead de llamadas a funciones pequeñas, reemplazando llamadas con el cuerpo de la función inlineado.

## 🔨 Implementación
Function inlining conservador que identifica funciones candidatas pequeñas (menos de 5 instrucciones) y reemplaza llamadas con su implementación inlineada.

### Archivos generados
- `compiler/src/codegen/ir_to_bytecode.rs` - Extensión de IROptimizer con function_inlining
- `compiler/src/tests/test_codegen_pipeline.rs` - Tests unitarios para inlining
- `docs/architecture/ADR-173-function-inlining.md` - Decisión arquitectónica

## ✅ Criterios de Aceptación
- [x] Function inlining implementado para funciones pequeñas
- [x] Tests unitarios pasando
- [x] Documentación completa
- [x] ADR creado

## 🔗 Referencias
- **Jira:** [TASK-173](https://velalang.atlassian.net/browse/TASK-173)
- **Historia:** [VELA-1184](https://velalang.atlassian.net/browse/VELA-1184)</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-1184\TASK-173.md