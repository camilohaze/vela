# ADR-068: Tests de navegación

## Estado
✅ Aceptado

## Fecha
2025-12-09

## Contexto
Como parte del desarrollo de la Navigation API (VELA-067), necesitamos una suite completa de tests para validar la correctness del sistema de navegación programática. Los tests deben cubrir navegación básica, guards, path building y manejo de errores.

## Decisión
Implementar los tests de navegación como parte integral del módulo `navigation::service`, siguiendo el patrón de tests integrados con la implementación.

## Consecuencias

### Positivas
- ✅ Tests disponibles desde el inicio del desarrollo
- ✅ Validación inmediata de correctness
- ✅ Facilita TDD (Test-Driven Development)
- ✅ Cobertura completa de casos edge
- ✅ Documentación ejecutable del comportamiento esperado

### Negativas
- ⚠️ Tests acoplados a la implementación (cambio en uno afecta al otro)
- ⚠️ Mayor complejidad de refactorización

## Alternativas Consideradas
1. **Tests separados en módulo dedicado**
   - Rechazada porque: Mayor separación de concerns pero más difícil mantener sincronizados

2. **Tests solo de integración**
   - Rechazada porque: No cubre casos unitarios específicos

3. **Tests manuales**
   - Rechazada porque: No escalable, propenso a errores humanos

## Implementación
Los tests están ubicados en `runtime/ui/src/navigation/service.rs` junto con la implementación.

## Referencias
- Jira: [VELA-068](https://velalang.atlassian.net/browse/VELA-068)
- Dependencia: [VELA-067](https://velalang.atlassian.net/browse/VELA-067)
- Código: `runtime/ui/src/navigation/service.rs`</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\architecture\ADR-068-navigation-tests.md