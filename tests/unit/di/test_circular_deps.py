"""
Tests comprehensivos para circular dependency detection.

Implementación de: TASK-035H
Historia: VELA-575
Sprint: 13

Tests para:
- CircularDependencyError con mensajes mejorados
- DependencyGraph: find_cycles, visualize, suggest_fixes
- Verificación estática: verify_no_cycles
- Advertencias preventivas: check_near_cycles
"""

import pytest
from typing import get_type_hints

from src.runtime.di.injector import Injector, CircularDependencyError
from src.runtime.di.injectable import injectable
from src.runtime.di.inject import inject
from src.runtime.di.scopes import Scope
from src.runtime.di.graph_analyzer import (
    DependencyGraph,
    DependencyNode,
    analyze_injector,
    verify_no_cycles
)
from src.runtime.di.cycle_warnings import (
    CycleWarningDetector,
    check_near_cycles
)


# ========================================
# Test CircularDependencyError Messages
# ========================================

def test_circular_dependency_error_message_has_suggestions():
    """Test que CircularDependencyError incluye sugerencias para romper ciclos."""
    from src.runtime.di.injector import CircularDependencyError
    
    @injectable
    class ServiceA:
        pass
    
    @injectable
    class ServiceB:
        pass
    
    cycle = [ServiceA, ServiceB, ServiceA]
    
    error = CircularDependencyError(cycle)
    error_message = str(error)
    
    # Verificar que contiene el ciclo
    assert "ServiceA" in error_message
    assert "ServiceB" in error_message
    assert "→" in error_message or "->" in error_message
    
    # Verificar que contiene sugerencias
    assert "Suggestions" in error_message or "sugerencias" in error_message.lower()
    assert "@lazy" in error_message or "lazy" in error_message.lower()
    assert "intermediate" in error_message.lower() or "interface" in error_message.lower()


# ========================================
# Test Simple Cycles (2 nodes)
# ========================================

def test_simple_cycle_2_nodes():
    """Test ciclo simple: A → B → A."""
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self, a: ServiceA = inject(ServiceA)):
            self.a = a
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    
    # Verificar que DependencyGraph detecta el ciclo
    graph = DependencyGraph(injector)
    cycles = graph.find_cycles()
    
    assert len(cycles) > 0
    
    # Verificar que contiene ServiceA y ServiceB
    cycle = cycles[0]
    cycle_names = [t.__name__ for t in cycle]
    
    assert "ServiceA" in cycle_names
    assert "ServiceB" in cycle_names


def test_simple_cycle_runtime_detection():
    """Test que ResolutionContext detecta ciclo en runtime."""
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self, a: ServiceA = inject(ServiceA)):
            self.a = a
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    
    # Intentar resolver (debe lanzar CircularDependencyError)
    with pytest.raises(CircularDependencyError) as exc_info:
        injector.get(ServiceA)
    
    error_message = str(exc_info.value)
    
    # Verificar mensaje de error
    assert "ServiceA" in error_message
    assert "ServiceB" in error_message


# ========================================
# Test Long Cycles (4+ nodes)
# ========================================

def test_long_cycle_4_nodes():
    """Test ciclo largo: A → B → C → D → A."""
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self, c: 'ServiceC' = inject('ServiceC')):
            self.c = c
    
    @injectable
    class ServiceC:
        def __init__(self, d: 'ServiceD' = inject('ServiceD')):
            self.d = d
    
    @injectable
    class ServiceD:
        def __init__(self, a: ServiceA = inject(ServiceA)):
            self.a = a
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    injector.register(ServiceC)
    injector.register(ServiceD)
    
    # Detectar con DependencyGraph
    graph = DependencyGraph(injector)
    cycles = graph.find_cycles()
    
    assert len(cycles) > 0
    
    cycle = cycles[0]
    assert len(cycle) == 5  # A → B → C → D → A (5 nodos: A aparece 2 veces)


def test_multiple_cycles():
    """Test grafo con múltiples ciclos separados."""
    
    # Ciclo 1: A → B → A
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self, a: ServiceA = inject(ServiceA)):
            self.a = a
    
    # Ciclo 2: C → D → C
    @injectable
    class ServiceC:
        def __init__(self, d: 'ServiceD' = inject('ServiceD')):
            self.d = d
    
    @injectable
    class ServiceD:
        def __init__(self, c: ServiceC = inject(ServiceC)):
            self.c = c
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    injector.register(ServiceC)
    injector.register(ServiceD)
    
    # Detectar múltiples ciclos
    graph = DependencyGraph(injector)
    cycles = graph.find_cycles()
    
    # Debe encontrar al menos 2 ciclos
    assert len(cycles) >= 2


# ========================================
# Test Self-Dependency
# ========================================

def test_self_dependency():
    """Test self-dependency: A → A."""
    
    @injectable
    class ServiceA:
        def __init__(self, a: 'ServiceA' = inject('ServiceA')):
            self.a = a
    
    injector = Injector()
    injector.register(ServiceA)
    
    # Detectar self-dependency
    graph = DependencyGraph(injector)
    cycles = graph.find_cycles()
    
    assert len(cycles) > 0
    
    cycle = cycles[0]
    assert len(cycle) == 2  # A → A (2 nodos: A aparece 2 veces)
    assert cycle[0] == cycle[1]  # Mismo tipo


# ========================================
# Test DependencyGraph Methods
# ========================================

def test_dependency_graph_find_cycles():
    """Test DependencyGraph.find_cycles() encuentra todos los ciclos."""
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self, c: 'ServiceC' = inject('ServiceC')):
            self.c = c
    
    @injectable
    class ServiceC:
        def __init__(self, a: ServiceA = inject(ServiceA)):
            self.a = a
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    injector.register(ServiceC)
    
    graph = DependencyGraph(injector)
    cycles = graph.find_cycles()
    
    assert len(cycles) > 0
    
    # Verificar que el ciclo contiene A, B, C
    cycle = cycles[0]
    cycle_names = {t.__name__ for t in cycle}
    
    assert "ServiceA" in cycle_names
    assert "ServiceB" in cycle_names
    assert "ServiceC" in cycle_names


def test_dependency_graph_visualize():
    """Test DependencyGraph.visualize() genera ASCII tree."""
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self, c: 'ServiceC' = inject('ServiceC')):
            self.c = c
    
    @injectable
    class ServiceC:
        def __init__(self):
            pass
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    injector.register(ServiceC)
    
    graph = DependencyGraph(injector)
    visualization = graph.visualize(root=ServiceA, max_depth=3)
    
    # Verificar que contiene nombres de servicios
    assert "ServiceA" in visualization
    assert "ServiceB" in visualization
    assert "ServiceC" in visualization
    
    # Verificar estructura de árbol (caracteres de línea)
    assert "├─" in visualization or "└─" in visualization


def test_dependency_graph_visualize_with_cycle():
    """Test DependencyGraph.visualize() marca ciclos con (CYCLE!)."""
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self, a: ServiceA = inject(ServiceA)):
            self.a = a
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    
    graph = DependencyGraph(injector)
    visualization = graph.visualize(root=ServiceA, max_depth=3)
    
    # Verificar que marca el ciclo
    assert "(CYCLE!)" in visualization or "CYCLE" in visualization


def test_dependency_graph_suggest_fixes():
    """Test DependencyGraph.suggest_fixes() retorna sugerencias."""
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self, a: ServiceA = inject(ServiceA)):
            self.a = a
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    
    graph = DependencyGraph(injector)
    cycles = graph.find_cycles()
    
    assert len(cycles) > 0
    
    cycle = cycles[0]
    suggestions = graph.suggest_fixes(cycle)
    
    # Verificar que retorna sugerencias
    assert len(suggestions) > 0
    assert any("@lazy" in s or "lazy" in s.lower() for s in suggestions)


def test_dependency_graph_get_statistics():
    """Test DependencyGraph.get_statistics() retorna métricas correctas."""
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self, c: 'ServiceC' = inject('ServiceC')):
            self.c = c
    
    @injectable
    class ServiceC:
        def __init__(self):
            pass
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    injector.register(ServiceC)
    
    graph = DependencyGraph(injector)
    stats = graph.get_statistics()
    
    # Verificar métricas
    assert "total_nodes" in stats
    assert "total_edges" in stats
    assert "leaves" in stats
    assert "roots" in stats
    assert "cycles_count" in stats
    
    assert stats["total_nodes"] == 3  # A, B, C
    assert stats["total_edges"] == 2  # A→B, B→C
    assert stats["leaves"] == 1  # C (no depende de nadie)
    assert stats["cycles_count"] == 0  # Sin ciclos


# ========================================
# Test Static Verification
# ========================================

def test_verify_no_cycles_success():
    """Test verify_no_cycles() no lanza error si no hay ciclos."""
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self, c: 'ServiceC' = inject('ServiceC')):
            self.c = c
    
    @injectable
    class ServiceC:
        def __init__(self):
            pass
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    injector.register(ServiceC)
    
    # No debe lanzar error
    verify_no_cycles(injector)


def test_verify_no_cycles_failure():
    """Test verify_no_cycles() lanza CircularDependencyError si hay ciclos."""
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self, a: ServiceA = inject(ServiceA)):
            self.a = a
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    
    # Debe lanzar CircularDependencyError
    with pytest.raises(CircularDependencyError):
        verify_no_cycles(injector)


def test_injector_validate_no_cycles_method():
    """Test Injector.validate_no_cycles() method."""
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self):
            pass
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    
    # No debe lanzar error
    injector.validate_no_cycles()


def test_injector_validate_cycles_option():
    """Test Injector(validate_cycles=True) opción de validación automática."""
    
    @injectable
    class ServiceA:
        def __init__(self):
            pass
    
    @injectable
    class ServiceB:
        def __init__(self, a: ServiceA = inject(ServiceA)):
            self.a = a
    
    # Con validate_cycles=True, debe validar después de cada registro
    injector = Injector(validate_cycles=True)
    
    # Primero registrar ServiceA (sin ciclos, OK)
    injector.register(ServiceA)
    
    # Registrar ServiceB (sin ciclos, OK)
    injector.register(ServiceB)
    
    # Sin errores hasta aquí
    assert injector.has_provider(ServiceA)
    assert injector.has_provider(ServiceB)


# ========================================
# Test Analyze Injector
# ========================================

def test_analyze_injector():
    """Test analyze_injector() retorna análisis completo."""
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self):
            pass
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    
    analysis = analyze_injector(injector)
    
    # Verificar estructura del análisis
    assert "statistics" in analysis
    assert "cycles" in analysis
    assert "visualization" in analysis
    
    # Verificar estadísticas
    stats = analysis["statistics"]
    assert stats["total_nodes"] == 2
    assert stats["cycles_count"] == 0
    
    # Verificar visualización
    viz = analysis["visualization"]
    assert "ServiceA" in viz or "ServiceB" in viz


# ========================================
# Test Near-Cycle Warnings
# ========================================

def test_check_near_cycles_no_warnings():
    """Test check_near_cycles() sin advertencias cuando no hay near-cycles."""
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self):
            pass
    
    @injectable
    class ServiceC:
        def __init__(self):
            pass
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    injector.register(ServiceC)
    
    # Verificar near-cycle si ServiceC dependiera de ServiceA
    # NO forma ciclo porque: A→B (cadena), C→A (nueva dependencia) no cierra ningún ciclo
    warnings = check_near_cycles(injector, ServiceC, ServiceA, log_warnings=False)
    
    # No debe haber advertencias
    assert len(warnings) == 0


def test_check_near_cycles_with_warning():
    """Test check_near_cycles() detecta advertencia cuando se cerraría ciclo."""
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB' = inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self, c: 'ServiceC' = inject('ServiceC')):
            self.c = c
    
    @injectable
    class ServiceC:
        def __init__(self):
            pass
    
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    injector.register(ServiceC)
    
    # Verificar near-cycle si ServiceC dependiera de ServiceA (cerraría ciclo A→B→C→A)
    warnings = check_near_cycles(injector, ServiceC, ServiceA, log_warnings=False)
    
    # Debe haber advertencias
    assert len(warnings) > 0
    
    warning = warnings[0]
    assert warning.from_type == ServiceC
    assert warning.to_type == ServiceA


def test_cycle_warning_detector():
    """Test CycleWarningDetector directamente."""
    
    @injectable
    class ServiceA:
        pass
    
    @injectable
    class ServiceB:
        pass
    
    @injectable
    class ServiceC:
        pass
    
    detector = CycleWarningDetector()
    
    # Construir grafo: A → B → C
    detector.add_dependency(ServiceA, ServiceB)
    detector.add_dependency(ServiceB, ServiceC)
    
    # Verificar near-cycle si agregamos C → A
    warnings = detector.check_new_dependency(ServiceC, ServiceA)
    
    assert len(warnings) > 0
    
    warning = warnings[0]
    assert warning.from_type == ServiceC
    assert warning.to_type == ServiceA
    
    # Verificar mensaje de advertencia
    warning_str = str(warning)
    assert "ServiceA" in warning_str
    assert "ServiceB" in warning_str
    assert "ServiceC" in warning_str


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
