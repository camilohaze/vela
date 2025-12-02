"""
Dependency Graph Analyzer

Implementación de: TASK-035H
Historia: VELA-575
Fecha: 2025-12-02

Descripción:
Analizador de grafo de dependencias para el DI Container.
Detecta ciclos, visualiza dependencias, sugiere refactorings.
"""

from typing import Type, List, Dict, Set, Optional, Tuple
from dataclasses import dataclass, field
import inspect


@dataclass
class DependencyNode:
    """
    Nodo en el grafo de dependencias.
    
    Attributes:
        type_: Tipo (clase) del nodo
        dependencies: List de tipos que este nodo requiere
        dependents: List de tipos que requieren este nodo
        scope: Scope del provider (si está registrado)
    """
    type_: Type
    dependencies: List[Type] = field(default_factory=list)
    dependents: List[Type] = field(default_factory=list)
    scope: Optional[str] = None
    
    def __hash__(self):
        return hash(self.type_)
    
    def __eq__(self, other):
        if isinstance(other, DependencyNode):
            return self.type_ == other.type_
        return False


class DependencyGraph:
    """
    Grafo de dependencias del DI Container.
    
    Permite:
    - Construir grafo desde Injector
    - Detectar TODOS los ciclos (DFS con backtracking)
    - Visualizar dependencias (ASCII tree)
    - Sugerir refactorings
    
    Ejemplo:
        >>> from src.runtime.di import Injector
        >>> injector = Injector()
        >>> # ... register providers ...
        >>> graph = DependencyGraph(injector)
        >>> cycles = graph.find_cycles()
        >>> print(graph.visualize())
    """
    
    def __init__(self, injector=None):
        """
        Inicializar grafo.
        
        Args:
            injector: Injector desde el cual construir el grafo.
                      Si es None, se crea grafo vacío.
        """
        self.nodes: Dict[Type, DependencyNode] = {}
        
        if injector:
            self._build_from_injector(injector)
    
    def add_node(self, type_: Type, scope: Optional[str] = None) -> DependencyNode:
        """
        Agregar nodo al grafo.
        
        Args:
            type_: Tipo del nodo
            scope: Scope del provider (opcional)
            
        Returns:
            Nodo creado o existente
        """
        if type_ not in self.nodes:
            self.nodes[type_] = DependencyNode(type_=type_, scope=scope)
        return self.nodes[type_]
    
    def add_dependency(self, from_type: Type, to_type: Type) -> None:
        """
        Agregar edge from_type → to_type.
        
        Args:
            from_type: Tipo que depende
            to_type: Tipo requerido
        """
        # Asegurar que ambos nodos existen
        from_node = self.add_node(from_type)
        to_node = self.add_node(to_type)
        
        # Agregar edge
        if to_type not in from_node.dependencies:
            from_node.dependencies.append(to_type)
        
        if from_type not in to_node.dependents:
            to_node.dependents.append(from_type)
    
    def _build_from_injector(self, injector) -> None:
        """
        Construir grafo desde Injector.
        
        Inspecciona todos los providers registrados y extrae dependencies.
        
        Args:
            injector: Injector instance
        """
        # Importar aquí para evitar circular imports
        from .injector import Injector
        
        if not isinstance(injector, Injector):
            raise TypeError(f"Expected Injector, got {type(injector)}")
        
        # Iterar sobre todos los providers registrados
        # Acceder al registry interno: _registry._providers
        for token, entry in injector._registry._providers.items():
            # Agregar nodo
            scope_name = entry.scope.name if hasattr(entry.scope, 'name') else str(entry.scope)
            node = self.add_node(token, scope=scope_name)
            
            # Agregar dependencies
            for dep in entry.dependencies:
                self.add_dependency(token, dep)
    
    def _extract_dependencies(self, cls: Type) -> List[Type]:
        """
        Extraer dependencies de una clase inspeccionando su __init__.
        
        Args:
            cls: Clase a inspeccionar
            
        Returns:
            Lista de tipos requeridos
        """
        try:
            sig = inspect.signature(cls.__init__)
            deps = []
            
            for param_name, param in sig.parameters.items():
                if param_name == 'self':
                    continue
                
                # Obtener type hint
                if param.annotation != inspect.Parameter.empty:
                    deps.append(param.annotation)
            
            return deps
        except Exception:
            return []
    
    def find_cycles(self) -> List[List[Type]]:
        """
        Encontrar TODOS los ciclos en el grafo.
        
        Usa DFS con backtracking para encontrar todos los ciclos.
        
        Returns:
            Lista de ciclos. Cada ciclo es una lista de tipos.
            Ejemplo: [[A, B, A], [C, D, E, C]]
        """
        cycles = []
        visited = set()
        path = []
        path_set = set()
        
        def dfs(node_type: Type):
            """DFS recursivo con detección de ciclos."""
            if node_type in path_set:
                # Ciclo detectado
                # Extraer el ciclo desde donde empieza
                cycle_start = path.index(node_type)
                cycle = path[cycle_start:] + [node_type]
                cycles.append(cycle)
                return
            
            if node_type in visited:
                return
            
            # Marcar como visitado
            visited.add(node_type)
            path.append(node_type)
            path_set.add(node_type)
            
            # Explorar dependencies
            if node_type in self.nodes:
                node = self.nodes[node_type]
                for dep in node.dependencies:
                    dfs(dep)
            
            # Backtrack
            path.pop()
            path_set.remove(node_type)
        
        # Ejecutar DFS desde cada nodo
        for node_type in self.nodes.keys():
            if node_type not in visited:
                dfs(node_type)
        
        return cycles
    
    def visualize(self, root: Optional[Type] = None, max_depth: int = 10) -> str:
        """
        Generar visualización ASCII del grafo.
        
        Args:
            root: Nodo raíz desde donde visualizar. Si es None, usa todos los nodos.
            max_depth: Profundidad máxima de visualización
            
        Returns:
            String con representación ASCII del árbol.
            
        Example:
            A
            ├─ B
            │  └─ C
            │     └─ A (CYCLE!)
            └─ D
        """
        if not self.nodes:
            return "(empty graph)"
        
        lines = []
        visited = set()
        
        def _render_node(node_type: Type, prefix: str = "", is_last: bool = True, depth: int = 0):
            """Renderizar nodo recursivamente."""
            if depth > max_depth:
                return
            
            # Check si es ciclo
            is_cycle = node_type in visited
            
            # Nombre del nodo
            node_name = node_type.__name__ if hasattr(node_type, '__name__') else str(node_type)
            
            if is_cycle:
                node_name += " (CYCLE!)"
            
            # Prefijo de línea
            connector = "└─ " if is_last else "├─ "
            lines.append(f"{prefix}{connector}{node_name}")
            
            # Si es ciclo, no seguir
            if is_cycle:
                return
            
            visited.add(node_type)
            
            # Renderizar dependencies
            if node_type in self.nodes:
                node = self.nodes[node_type]
                deps = node.dependencies
                
                for i, dep in enumerate(deps):
                    is_last_dep = (i == len(deps) - 1)
                    extension = "   " if is_last else "│  "
                    _render_node(dep, prefix + extension, is_last_dep, depth + 1)
            
            visited.remove(node_type)
        
        # Si se especifica root, usarlo
        if root:
            _render_node(root)
        else:
            # Visualizar todos los nodos raíz (sin dependents)
            roots = [node_type for node_type, node in self.nodes.items() if not node.dependents]
            
            if not roots:
                # Si no hay raíces, usar el primer nodo
                roots = [list(self.nodes.keys())[0]]
            
            for i, root_type in enumerate(roots):
                _render_node(root_type, "", is_last=(i == len(roots) - 1))
        
        return "\n".join(lines)
    
    def suggest_fixes(self, cycle: List[Type]) -> List[str]:
        """
        Sugerir refactorings para romper ciclo.
        
        Args:
            cycle: Ciclo a analizar
            
        Returns:
            Lista de sugerencias
        """
        suggestions = []
        
        if len(cycle) <= 1:
            return suggestions
        
        # Ciclo simple A -> A (self-dependency)
        if len(cycle) == 2 and cycle[0] == cycle[1]:
            suggestions.append(
                f"Self-dependency detected on {cycle[0].__name__}. "
                "Remove the dependency on itself."
            )
            return suggestions
        
        # Ciclo de 2 nodos (A -> B -> A)
        if len(cycle) == 3:
            a, b = cycle[0], cycle[1]
            suggestions.append(
                f"Simple cycle between {a.__name__} and {b.__name__}. "
                f"Consider:\n"
                f"  - Use @lazy() to inject {b.__name__} lazily\n"
                f"  - Introduce interface to break dependency\n"
                f"  - Use event-driven communication"
            )
            return suggestions
        
        # Ciclo largo (A -> B -> C -> ... -> A)
        cycle_str = " -> ".join(t.__name__ for t in cycle[:-1])
        suggestions.append(
            f"Long cycle detected: {cycle_str}\n"
            f"Consider:\n"
            f"  - Break at weakest dependency (least coupled)\n"
            f"  - Introduce intermediate service\n"
            f"  - Refactor to layer architecture\n"
            f"  - Use event bus for communication"
        )
        
        return suggestions
    
    def get_statistics(self) -> Dict[str, any]:
        """
        Obtener estadísticas del grafo.
        
        Returns:
            Dict con métricas del grafo
        """
        total_nodes = len(self.nodes)
        total_edges = sum(len(node.dependencies) for node in self.nodes.values())
        
        # Nodos sin dependencies (leaves)
        leaves = [t for t, node in self.nodes.items() if not node.dependencies]
        
        # Nodos sin dependents (roots)
        roots = [t for t, node in self.nodes.items() if not node.dependents]
        
        # Ciclos
        cycles = self.find_cycles()
        
        return {
            "total_nodes": total_nodes,
            "total_edges": total_edges,
            "leaves": len(leaves),
            "roots": len(roots),
            "cycles_count": len(cycles),
            "has_cycles": len(cycles) > 0
        }


def analyze_injector(injector) -> Dict[str, any]:
    """
    Analizar Injector y retornar estadísticas.
    
    Args:
        injector: Injector instance
        
    Returns:
        Dict con análisis completo
    """
    graph = DependencyGraph(injector)
    stats = graph.get_statistics()
    cycles = graph.find_cycles()
    
    return {
        "statistics": stats,
        "cycles": cycles,
        "visualization": graph.visualize() if stats["total_nodes"] <= 20 else "(too large to visualize)"
    }


def verify_no_cycles(injector) -> None:
    """
    Verificar que no hay ciclos en el Injector.
    
    Lanza CircularDependencyError si se encuentran ciclos.
    
    Args:
        injector: Injector instance
        
    Raises:
        CircularDependencyError: Si hay ciclos
    """
    from .injector import CircularDependencyError
    
    graph = DependencyGraph(injector)
    cycles = graph.find_cycles()
    
    if cycles:
        # Lanzar con el primer ciclo encontrado
        raise CircularDependencyError(cycles[0])


if __name__ == "__main__":
    # Ejemplo de uso
    print("DependencyGraph Analyzer")
    print("=" * 40)
    print("Import this module and use DependencyGraph(injector)")
