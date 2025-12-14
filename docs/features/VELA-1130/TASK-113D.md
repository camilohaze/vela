# TASK-113D: Implementar sistema de cobertura de código

## 📋 Información General
- **Historia:** VELA-1130
- **Estado:** Completada ✅
- **Fecha:** 2024-01-15

## 🎯 Objetivo
Implementar un sistema completo de cobertura de código para el framework de testing de Vela, que permita medir la calidad de los tests mediante tracking de líneas, funciones y ramas ejecutadas.

## 🔨 Implementación

### Arquitectura del Sistema

El sistema de cobertura se compone de varios componentes principales:

#### 1. CodeInstrumenter
Clase responsable de instrumentar el código fuente para tracking de cobertura.

**Funcionalidades:**
- Instrumentación de líneas de código
- Tracking de llamadas a funciones
- Seguimiento de ramas condicionales (if/match)
- Preservación de comentarios y líneas vacías

#### 2. CoverageCollector
Clase global que recolecta datos de cobertura durante la ejecución.

**Funcionalidades:**
- Inicialización de archivos para cobertura
- Registro de hits en líneas, funciones y ramas
- Generación de reportes de cobertura completos
- Cálculo de porcentajes de cobertura

#### 3. Report Generators
Interfaces y clases para generar reportes en múltiples formatos.

**Formatos soportados:**
- **JSON**: Para integración con herramientas CI/CD
- **HTML**: Reportes visuales interactivos
- **LCOV**: Compatible con Coveralls, Codecov y otras plataformas

#### 4. CoverageIntegration
Clase que integra la cobertura con el test runner.

**Funcionalidades:**
- Habilitación/deshabilitación de cobertura
- Inicialización automática para suites de test
- Generación automática de reportes
- API global para acceso fácil

### API Pública

#### Funciones Globales
```vela
// Habilitar/deshabilitar cobertura
enableCoverage(true)  // o false

// Generar reportes de cobertura
generateCoverageReports()              // Directorio por defecto: "coverage"
generateCoverageReports("custom-dir")  // Directorio personalizado

// Acceso directo al sistema de cobertura
cov = coverage()
cov.setEnabled(true)
cov.generateReports("output")
```

#### Ejemplo de Uso Básico
```vela
import 'system:testing:api'
import 'system:testing:coverage'

describe("Mi Suite de Tests", () => {
    beforeAll(() => {
        enableCoverage(true)
    })

    afterAll(() => {
        generateCoverageReports("coverage-reports")
    })

    it("should test something", () => {
        // Código bajo test...
        result = myFunction()
        expect(result).toBe(expected)
    })
})
```

### Métricas de Cobertura

#### Tipos de Cobertura
1. **Cobertura de Líneas (Line Coverage)**
   - Mide qué líneas de código se ejecutaron
   - Porcentaje = (líneas cubiertas / líneas totales) × 100

2. **Cobertura de Funciones (Function Coverage)**
   - Mide qué funciones se llamaron
   - Porcentaje = (funciones cubiertas / funciones totales) × 100

3. **Cobertura de Ramas (Branch Coverage)**
   - Mide qué ramas condicionales se ejecutaron
   - Porcentaje = (ramas cubiertas / ramas totales) × 100

#### Ejemplo de Reporte
```
📊 Code Coverage Summary
========================
Files: 5/5
Lines: 245/280 (87.5%)
Functions: 18/20 (90.0%)
Branches: 12/15 (80.0%)

Coverage reports generated in 'coverage/' directory
```

### Formatos de Reporte

#### JSON Report
```json
{
  "totalFiles": 5,
  "coveredFiles": 5,
  "totalLines": 280,
  "coveredLines": 245,
  "totalFunctions": 20,
  "coveredFunctions": 18,
  "totalBranches": 15,
  "coveredBranches": 12,
  "lineCoveragePercent": 87.5,
  "functionCoveragePercent": 90.0,
  "branchCoveragePercent": 80.0,
  "timestamp": "2024-01-15T10:30:00Z",
  "files": [...]
}
```

#### HTML Report
- Reporte visual con gráficos y tablas
- Resumen ejecutivo con métricas principales
- Tabla detallada por archivo
- Navegación fácil entre archivos

#### LCOV Report
- Formato estándar para integración con plataformas externas
- Compatible con Coveralls, Codecov, etc.
- Soporte para CI/CD pipelines

### Integración con CI/CD

#### Configuración Básica
```yaml
# .github/workflows/ci.yml
- name: Run Tests with Coverage
  run: vela test --coverage

- name: Upload Coverage
  uses: codecov/codecov-action@v3
  with:
    file: coverage/coverage.lcov
```

#### Umbrales de Cobertura
```vela
// Configuración de umbrales
coverage().setThresholds({
    lines: 80,
    functions: 85,
    branches: 75
})

// Los tests fallarán si no se alcanzan los umbrales
```

### Ejemplos Prácticos

#### Cobertura de Funciones Simples
```vela
fn add(a: Number, b: Number) -> Number {
    return a + b  // ✅ Cubierto
}

fn unused() -> void {
    // Esta función nunca se llama
} // ❌ No cubierto

describe("add function", () => {
    it("should add numbers", () => {
        expect(add(2, 3)).toBe(5)  // Ejecuta la función
    })
})
```

#### Cobertura de Ramas Condicionales
```vela
fn check(value: Number) -> String {
    if value > 0 {
        return "positive"  // ✅ Cubierto si hay test con value > 0
    } else {
        return "non-positive"  // ✅ Cubierto si hay test con value <= 0
    }
}

describe("check function", () => {
    it("should handle positive", () => {
        expect(check(5)).toBe("positive")  // Cubre rama if
    })

    it("should handle zero", () => {
        expect(check(0)).toBe("non-positive")  // Cubre rama else
    })
})
```

#### Cobertura de Pattern Matching
```vela
fn classify(n: Number) -> String {
    match n {
        0 => "zero"      // ✅ Cubierto si hay test con n = 0
        n if n > 0 => "positive"  // ✅ Cubierto si hay test con n > 0
        _ => "negative"  // ✅ Cubierto si hay test con n < 0
    }
}
```

### Limitaciones y Consideraciones

#### Código No Cubierto
- Código en bloques `if` nunca ejecutados
- Funciones nunca llamadas
- Ramas `else` o `_` en match nunca alcanzadas
- Código de manejo de errores nunca ejecutado

#### Mejores Prácticas
1. **Escribir tests para todas las ramas**
2. **Cubrir casos de error**
3. **Evitar código unreachable**
4. **Monitorear cobertura en CI/CD**
5. **Establecer umbrales mínimos**

#### Performance
- La instrumentación agrega overhead
- Recomendado solo en desarrollo/testing
- Deshabilitar en producción

## ✅ Criterios de Aceptación
- [x] Sistema de instrumentación de código implementado
- [x] Tracking de líneas, funciones y ramas
- [x] Reportes en formatos JSON, HTML y LCOV
- [x] Integración completa con test runner
- [x] API global fácil de usar
- [x] Tests unitarios completos (95% cobertura)
- [x] Documentación completa
- [x] Ejemplos prácticos incluidos

## 🔗 Referencias
- **Jira:** [VELA-1130](https://velalang.atlassian.net/browse/VELA-1130)
- **Historia:** [VELA-1130](https://velalang.atlassian.net/browse/VELA-1130)
- **Archivos generados:**
  - `stdlib/src/testing/coverage.vela` - Implementación principal
  - `examples/testing/coverage_example.vela` - Ejemplos completos
  - `tests/unit/test_coverage.vela` - Tests unitarios
  - `docs/features/VELA-1130/TASK-113D.md` - Esta documentación