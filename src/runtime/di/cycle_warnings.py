"""
Advertencias preventivas para near-cycles en el grafo de dependencias.

Implementación de: TASK-035H
Historia: VELA-575
Sprint: 13

Este módulo detecta patrones que podrían formar ciclos si se registran
nuevas dependencias, permitiendo advertir temprano al desarrollador.

Ejemplo:
    # Estado actual: A → B → C
    # Si registramos C → A, se forma ciclo
    
    injector = Injector()
    injector.register(ServiceA)  # A depende de B
    injector.register(ServiceB)  # B depende de C
    injector.register(ServiceC)  # C depende de nada
    
    # WARNING: Si ahora C depende de A, se forma ciclo A → B → C → A
    # CycleWarningDetector puede advertir antes de registrar
"""

from typing import Type, List, Set, Dict, Optional
from dataclasses import dataclass
import logging


logger = logging.getLogger(__name__)


@dataclass
class NearCycleWarning:
    """
    Advertencia de near-cycle.
    
    Representa una situación donde una nueva dependencia
    podría cerrar un ciclo existente.
    """
    
    # Nueva dependencia propuesta: from_type → to_type
    from_type: Type
    to_type: Type
    
    # Cadena actual que se cerraría (A → B → C, si agregamos C → A)
    existing_chain: List[Type]
    
    # Ciclo que se formaría
    potential_cycle: List[Type]
    
    def __str__(self) -> str:
        chain_str = " → ".join([t.__name__ for t in self.existing_chain])
        cycle_str = " → ".join([t.__name__ for t in self.potential_cycle])
        
        return (
            f"⚠️  Near-cycle detected!\n"
            f"   Existing chain: {chain_str}\n"
            f"   Adding: {self.from_type.__name__} → {self.to_type.__name__}\n"
            f"   Would form cycle: {cycle_str}\n"
            f"   Suggestion: Refactor dependencies to avoid circular reference"
        )


class CycleWarningDetector:
    """
    Detector de advertencias para near-cycles.
    
    Analiza el grafo de dependencias y detecta patrones que
    podrían formar ciclos si se agregan nuevas dependencias.
    
    Ejemplo:
        detector = CycleWarningDetector()
        
        # Construir grafo
        detector.add_dependency(ServiceA, ServiceB)
        detector.add_dependency(ServiceB, ServiceC)
        
        # Detectar near-cycle
        warnings = detector.check_new_dependency(ServiceC, ServiceA)
        if warnings:
            for warning in warnings:
                print(warning)  # ⚠️ Near-cycle detected!
    """
    
    def __init__(self):
        # Grafo de dependencias: {from_type: [to_type1, to_type2, ...]}
        self._graph: Dict[Type, List[Type]] = {}
    
    def add_dependency(self, from_type: Type, to_type: Type) -> None:
        """
        Agregar dependencia al grafo.
        
        Args:
            from_type: Tipo que depende de to_type.
            to_type: Tipo del que depende from_type.
        """
        if from_type not in self._graph:
            self._graph[from_type] = []
        
        if to_type not in self._graph[from_type]:
            self._graph[from_type].append(to_type)
    
    def check_new_dependency(
        self,
        from_type: Type,
        to_type: Type
    ) -> List[NearCycleWarning]:
        """
        Verificar si una nueva dependencia cerraría un ciclo.
        
        Detecta patrones como:
        - Cadena A → B → C, agregar C → A forma ciclo
        - Cadena A → B → C → D, agregar D → B forma ciclo
        
        Args:
            from_type: Nuevo tipo que dependería de to_type.
            to_type: Tipo del que dependería from_type.
        
        Returns:
            Lista de advertencias (vacía si no hay near-cycles).
        
        Ejemplo:
            # Grafo actual: A → B → C
            warnings = detector.check_new_dependency(C, A)
            # Retorna: [NearCycleWarning(chain=[A, B, C], cycle=[A, B, C, A])]
        """
        warnings: List[NearCycleWarning] = []
        
        # Buscar todas las cadenas desde to_type hasta from_type
        # (porque agregaríamos from_type → to_type, cerraría ciclo)
        chains = self._find_all_paths(to_type, from_type)
        
        for chain in chains:
            # Ciclo se formaría: chain + [from_type → to_type]
            potential_cycle = chain + [to_type]
            
            warning = NearCycleWarning(
                from_type=from_type,
                to_type=to_type,
                existing_chain=chain,
                potential_cycle=potential_cycle
            )
            
            warnings.append(warning)
        
        return warnings
    
    def _find_all_paths(
        self,
        start: Type,
        end: Type,
        max_depth: int = 10
    ) -> List[List[Type]]:
        """
        Encontrar todos los caminos desde start hasta end.
        
        Usa DFS con backtracking para encontrar todos los caminos posibles.
        
        Args:
            start: Nodo inicial.
            end: Nodo final.
            max_depth: Profundidad máxima de búsqueda.
        
        Returns:
            Lista de caminos (cada camino es lista de tipos).
        """
        paths: List[List[Type]] = []
        
        def dfs(current: Type, path: List[Type], visited: Set[Type]) -> None:
            # Condición de salida: encontramos end
            if current == end:
                paths.append(path[:])  # Copiar path
                return
            
            # Límite de profundidad
            if len(path) >= max_depth:
                return
            
            # Explorar vecinos
            for neighbor in self._graph.get(current, []):
                if neighbor not in visited:
                    visited.add(neighbor)
                    path.append(neighbor)
                    
                    dfs(neighbor, path, visited)
                    
                    # Backtracking
                    path.pop()
                    visited.remove(neighbor)
        
        # Iniciar DFS
        if start in self._graph:
            dfs(start, [start], {start})
        
        return paths
    
    def build_from_injector(self, injector) -> None:
        """
        Construir grafo de dependencias desde un Injector.
        
        Args:
            injector: Instancia de Injector.
        
        Ejemplo:
            detector = CycleWarningDetector()
            detector.build_from_injector(injector)
            
            # Ahora detector tiene grafo completo
            warnings = detector.check_new_dependency(ServiceC, ServiceA)
        """
        from .graph_analyzer import DependencyGraph
        
        # Usar DependencyGraph para construir grafo
        graph = DependencyGraph(injector)
        
        # Copiar grafo a detector
        for node in graph.nodes.values():
            for dep in node.dependencies:
                self.add_dependency(node.type_, dep)
    
    def log_warning(self, warning: NearCycleWarning) -> None:
        """
        Loggear advertencia de near-cycle.
        
        Args:
            warning: Advertencia a loggear.
        """
        logger.warning(str(warning))


def check_near_cycles(
    injector,
    new_from_type: Type,
    new_to_type: Type,
    log_warnings: bool = True
) -> List[NearCycleWarning]:
    """
    Helper function para verificar near-cycles antes de registrar.
    
    Args:
        injector: Instancia de Injector.
        new_from_type: Nuevo tipo que dependería de new_to_type.
        new_to_type: Tipo del que dependería new_from_type.
        log_warnings: Si True, loggea advertencias automáticamente.
    
    Returns:
        Lista de advertencias (vacía si no hay near-cycles).
    
    Ejemplo:
        # Antes de registrar ServiceC
        warnings = check_near_cycles(injector, ServiceC, ServiceA)
        
        if warnings:
            print("⚠️ WARNING: Registering ServiceC would create cycle!")
            for w in warnings:
                print(w)
        else:
            injector.register(ServiceC)  # Seguro
    """
    detector = CycleWarningDetector()
    detector.build_from_injector(injector)
    
    warnings = detector.check_new_dependency(new_from_type, new_to_type)
    
    if log_warnings:
        for warning in warnings:
            detector.log_warning(warning)
    
    return warnings


if __name__ == "__main__":
    # Ejemplo de uso
    from .injector import Injector
    from .injectable import injectable
    from .inject import inject
    
    @injectable
    class ServiceA:
        def __init__(self, b=inject('ServiceB')):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self, c=inject('ServiceC')):
            self.c = c
    
    @injectable
    class ServiceC:
        def __init__(self):
            pass
    
    # Construir injector
    injector = Injector()
    injector.register(ServiceA)
    injector.register(ServiceB)
    injector.register(ServiceC)
    
    # Verificar near-cycle si ServiceC dependiera de ServiceA
    print("\n=== Verificar near-cycle ===")
    warnings = check_near_cycles(injector, ServiceC, ServiceA)
    
    if warnings:
        print(f"Found {len(warnings)} near-cycle warnings:")
        for w in warnings:
            print(w)
    else:
        print("✓ No near-cycles detected")
