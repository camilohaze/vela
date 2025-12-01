"""
Import Resolver con Prefijos

Implementación de: TASK-016L
Historia: VELA-572
Fecha: 2025-01-22

Descripción:
Resolver todos los prefijos de imports de Vela:
- system: desde stdlib (buscar en stdlib/)
- package: desde node_modules
- module: resolver @module declarations en proyecto
- library: resolver @library internas
- extension: resolver @extension
- assets: desde carpeta assets/

Manejo de errores específicos por prefijo.
"""

from enum import Enum, auto
from typing import Optional, Dict, List
from dataclasses import dataclass
from pathlib import Path


class ImportPrefix(Enum):
    """Prefijos de imports en Vela."""
    SYSTEM = "system"          # system:ui → buscar en stdlib/
    PACKAGE = "package"        # package:lodash → buscar en node_modules/
    MODULE = "module"          # module:auth → buscar @module declarations
    LIBRARY = "library"        # library:utils → buscar @library internas
    EXTENSION = "extension"    # extension:charts → buscar @extension
    ASSETS = "assets"          # assets:images → buscar en assets/


@dataclass
class ImportPath:
    """
    Representa un import path parseado.
    
    Ejemplo: 'system:ui' → prefix='system', path='ui'
    Ejemplo: 'package:lodash' → prefix='package', path='lodash'
    """
    prefix: ImportPrefix
    path: str                   # Ruta después del ':'
    original: str               # String original del import
    
    @classmethod
    def parse(cls, import_str: str) -> Optional['ImportPath']:
        """
        Parsea un string de import a ImportPath.
        
        Args:
            import_str: String del import (ej: 'system:ui')
            
        Returns:
            ImportPath si es válido, None si no tiene prefijo válido
        """
        if ':' not in import_str:
            return None  # No tiene prefijo
        
        prefix_str, path = import_str.split(':', 1)
        
        try:
            prefix = ImportPrefix(prefix_str)
            return cls(prefix=prefix, path=path, original=import_str)
        except ValueError:
            return None  # Prefijo no válido


@dataclass
class ResolvedImport:
    """
    Resultado de resolver un import.
    
    Contiene la información del import resuelto:
    - resolved_path: Ruta absoluta al archivo/módulo
    - prefix: Prefijo usado
    - exists: Si el archivo/módulo existe
    - error: Mensaje de error si no se pudo resolver
    """
    import_path: ImportPath
    resolved_path: Optional[Path] = None
    exists: bool = False
    error: Optional[str] = None
    module_name: Optional[str] = None  # Para imports de @module


class ImportResolver:
    """
    Resuelve imports con prefijos en Vela.
    
    El resolver busca archivos/módulos según el prefijo:
    - system: busca en stdlib/
    - package: busca en node_modules/
    - module: busca @module declarations en el proyecto
    - library: busca @library en src/libraries/
    - extension: busca @extension en src/extensions/
    - assets: busca en assets/
    
    Funcionalidades:
    - Resolver import con prefijo
    - Verificar existencia de archivo/módulo
    - Manejo de errores específicos por prefijo
    - Búsqueda en múltiples directorios
    """
    
    def __init__(self, project_root: Path):
        """
        Inicializa el resolver con el directorio raíz del proyecto.
        
        Args:
            project_root: Path al directorio raíz del proyecto Vela
        """
        self.project_root = project_root
        
        # Directorios estándar
        self.stdlib_dir = project_root / "stdlib"
        self.node_modules_dir = project_root / "node_modules"
        self.src_dir = project_root / "src"
        self.libraries_dir = self.src_dir / "libraries"
        self.extensions_dir = self.src_dir / "extensions"
        self.assets_dir = project_root / "assets"
        
        # Cache de @module declarations (se llena durante semantic analysis)
        self.module_declarations: Dict[str, Path] = {}
        
        # Cache de @library declarations
        self.library_declarations: Dict[str, Path] = {}
        
        # Cache de @extension declarations
        self.extension_declarations: Dict[str, Path] = {}
    
    def resolve(self, import_str: str) -> ResolvedImport:
        """
        Resuelve un import string completo.
        
        Args:
            import_str: String del import (ej: 'system:ui', 'package:lodash')
            
        Returns:
            ResolvedImport con resultado de la resolución
        """
        # Parsear import path
        import_path = ImportPath.parse(import_str)
        if not import_path:
            return ResolvedImport(
                import_path=ImportPath(prefix=ImportPrefix.SYSTEM, path="", original=import_str),
                error=f"Invalid import format: '{import_str}' (expected 'prefix:path')"
            )
        
        # Resolver según el prefijo
        if import_path.prefix == ImportPrefix.SYSTEM:
            return self._resolve_system(import_path)
        elif import_path.prefix == ImportPrefix.PACKAGE:
            return self._resolve_package(import_path)
        elif import_path.prefix == ImportPrefix.MODULE:
            return self._resolve_module(import_path)
        elif import_path.prefix == ImportPrefix.LIBRARY:
            return self._resolve_library(import_path)
        elif import_path.prefix == ImportPrefix.EXTENSION:
            return self._resolve_extension(import_path)
        elif import_path.prefix == ImportPrefix.ASSETS:
            return self._resolve_assets(import_path)
        else:
            return ResolvedImport(
                import_path=import_path,
                error=f"Unknown import prefix: {import_path.prefix}"
            )
    
    def _resolve_system(self, import_path: ImportPath) -> ResolvedImport:
        """
        Resuelve import con prefijo system: (stdlib).
        
        Busca en stdlib/{path}.vela o stdlib/{path}/index.vela
        
        Args:
            import_path: Parsed import path
            
        Returns:
            ResolvedImport con resultado
        """
        # Buscar stdlib/{path}.vela
        file_path = self.stdlib_dir / f"{import_path.path}.vela"
        if file_path.exists():
            return ResolvedImport(
                import_path=import_path,
                resolved_path=file_path,
                exists=True
            )
        
        # Buscar stdlib/{path}/index.vela
        index_path = self.stdlib_dir / import_path.path / "index.vela"
        if index_path.exists():
            return ResolvedImport(
                import_path=import_path,
                resolved_path=index_path,
                exists=True
            )
        
        return ResolvedImport(
            import_path=import_path,
            error=f"System module '{import_path.path}' not found in stdlib/"
        )
    
    def _resolve_package(self, import_path: ImportPath) -> ResolvedImport:
        """
        Resuelve import con prefijo package: (node_modules).
        
        Busca en node_modules/{path}/
        
        Args:
            import_path: Parsed import path
            
        Returns:
            ResolvedImport con resultado
        """
        # Buscar node_modules/{path}/package.json
        package_dir = self.node_modules_dir / import_path.path
        package_json = package_dir / "package.json"
        
        if package_json.exists():
            return ResolvedImport(
                import_path=import_path,
                resolved_path=package_dir,
                exists=True
            )
        
        return ResolvedImport(
            import_path=import_path,
            error=f"Package '{import_path.path}' not found in node_modules/ (run 'vela install')"
        )
    
    def _resolve_module(self, import_path: ImportPath) -> ResolvedImport:
        """
        Resuelve import con prefijo module: (@module declarations).
        
        Busca en el cache de @module declarations registradas durante
        el semantic analysis.
        
        Args:
            import_path: Parsed import path
            
        Returns:
            ResolvedImport con resultado
        """
        module_name = import_path.path
        
        if module_name in self.module_declarations:
            return ResolvedImport(
                import_path=import_path,
                resolved_path=self.module_declarations[module_name],
                exists=True,
                module_name=module_name
            )
        
        return ResolvedImport(
            import_path=import_path,
            error=f"Module '{module_name}' not found (no @module declaration with name '{module_name}')"
        )
    
    def _resolve_library(self, import_path: ImportPath) -> ResolvedImport:
        """
        Resuelve import con prefijo library: (@library internas).
        
        Busca en src/libraries/{path}.vela o cache de @library.
        
        Args:
            import_path: Parsed import path
            
        Returns:
            ResolvedImport con resultado
        """
        library_name = import_path.path
        
        # Buscar en cache de @library
        if library_name in self.library_declarations:
            return ResolvedImport(
                import_path=import_path,
                resolved_path=self.library_declarations[library_name],
                exists=True
            )
        
        # Buscar src/libraries/{path}.vela
        file_path = self.libraries_dir / f"{library_name}.vela"
        if file_path.exists():
            return ResolvedImport(
                import_path=import_path,
                resolved_path=file_path,
                exists=True
            )
        
        return ResolvedImport(
            import_path=import_path,
            error=f"Library '{library_name}' not found in src/libraries/"
        )
    
    def _resolve_extension(self, import_path: ImportPath) -> ResolvedImport:
        """
        Resuelve import con prefijo extension: (@extension).
        
        Busca en src/extensions/{path}.vela o cache de @extension.
        
        Args:
            import_path: Parsed import path
            
        Returns:
            ResolvedImport con resultado
        """
        extension_name = import_path.path
        
        # Buscar en cache de @extension
        if extension_name in self.extension_declarations:
            return ResolvedImport(
                import_path=import_path,
                resolved_path=self.extension_declarations[extension_name],
                exists=True
            )
        
        # Buscar src/extensions/{path}.vela
        file_path = self.extensions_dir / f"{extension_name}.vela"
        if file_path.exists():
            return ResolvedImport(
                import_path=import_path,
                resolved_path=file_path,
                exists=True
            )
        
        return ResolvedImport(
            import_path=import_path,
            error=f"Extension '{extension_name}' not found in src/extensions/"
        )
    
    def _resolve_assets(self, import_path: ImportPath) -> ResolvedImport:
        """
        Resuelve import con prefijo assets: (archivos estáticos).
        
        Busca en assets/{path}
        
        Args:
            import_path: Parsed import path
            
        Returns:
            ResolvedImport con resultado
        """
        # Buscar assets/{path}
        file_path = self.assets_dir / import_path.path
        
        if file_path.exists():
            return ResolvedImport(
                import_path=import_path,
                resolved_path=file_path,
                exists=True
            )
        
        return ResolvedImport(
            import_path=import_path,
            error=f"Asset '{import_path.path}' not found in assets/"
        )
    
    def register_module(self, module_name: str, file_path: Path) -> None:
        """
        Registra un @module declaration para resolución futura.
        
        Args:
            module_name: Nombre del módulo (sin prefijo)
            file_path: Path al archivo donde está declarado
        """
        self.module_declarations[module_name] = file_path
    
    def register_library(self, library_name: str, file_path: Path) -> None:
        """
        Registra un @library declaration para resolución futura.
        
        Args:
            library_name: Nombre de la librería
            file_path: Path al archivo donde está declarada
        """
        self.library_declarations[library_name] = file_path
    
    def register_extension(self, extension_name: str, file_path: Path) -> None:
        """
        Registra un @extension declaration para resolución futura.
        
        Args:
            extension_name: Nombre de la extensión
            file_path: Path al archivo donde está declarada
        """
        self.extension_declarations[extension_name] = file_path


if __name__ == "__main__":
    # Ejemplo de uso
    from pathlib import Path
    
    # Crear resolver
    project_root = Path(__file__).parent.parent.parent
    resolver = ImportResolver(project_root)
    
    # Parsear import path
    import_path = ImportPath.parse("system:ui")
    print(f"Parsed: {import_path}")
    
    # Resolver diferentes prefijos
    test_imports = [
        "system:ui",
        "package:lodash",
        "module:auth",
        "library:utils",
        "extension:charts",
        "assets:images"
    ]
    
    for import_str in test_imports:
        result = resolver.resolve(import_str)
        print(f"\n{import_str}:")
        print(f"  Exists: {result.exists}")
        if result.resolved_path:
            print(f"  Path: {result.resolved_path}")
        if result.error:
            print(f"  Error: {result.error}")
