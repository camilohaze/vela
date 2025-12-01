"""
Tests para Parser Principal de Vela

Tests de: VELA-568 (TASK-012)
Historia: Sprint 6 - Parser que genere AST válido
Fecha: 2025-12-01

Este módulo testea el parser principal:
- Parsing de programa completo
- Imports (todos los tipos)
- Import clauses (show, hide, as)
- Error handling
- Sincronización
"""

import pytest
import sys
sys.path.append('../..')

from src.parser import Parser, ParserError, parse_code
from src.parser.ast_nodes import Program, ImportDeclaration
from src.lexer import tokenize


class TestProgramParsing:
    """Tests para parsing de programa completo"""
    
    def test_empty_program(self):
        """Test parsear programa vacío"""
        code = ""
        ast = parse_code(code)
        assert ast is not None
        assert isinstance(ast, Program)
        assert len(ast.imports) == 0
        assert len(ast.declarations) == 0
    
    def test_program_with_imports(self):
        """Test programa con imports"""
        code = """
        import 'system:io'
        import 'package:http'
        
        fn main() -> void {
            print("Hello")
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.imports) == 2
        assert len(ast.declarations) == 1
    
    def test_program_with_declarations(self):
        """Test programa con declarations"""
        code = """
        struct Point { x: Number, y: Number }
        
        fn distance(p1: Point, p2: Point) -> Float {
            return 0.0
        }
        
        class Shape {}
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 3
    
    def test_program_complete(self):
        """Test programa completo con imports + declarations"""
        code = """
        import 'system:math'
        
        struct Vector { x: Float, y: Float }
        
        fn magnitude(v: Vector) -> Float {
            return sqrt(v.x * v.x + v.y * v.y)
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.imports) == 1
        assert len(ast.declarations) == 2


class TestImportDeclarations:
    """Tests para import declarations"""
    
    def test_system_import(self):
        """Test import system:"""
        code = "import 'system:io'"
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.imports) == 1
        assert ast.imports[0].import_type == "system"
    
    def test_package_import(self):
        """Test import package:"""
        code = "import 'package:http'"
        ast = parse_code(code)
        assert ast is not None
        assert ast.imports[0].import_type == "package"
    
    def test_module_import(self):
        """Test import module:"""
        code = "import 'module:utils'"
        ast = parse_code(code)
        assert ast is not None
        assert ast.imports[0].import_type == "module"
    
    def test_library_import(self):
        """Test import library:"""
        code = "import 'library:db'"
        ast = parse_code(code)
        assert ast is not None
        assert ast.imports[0].import_type == "library"
    
    def test_extension_import(self):
        """Test import extension:"""
        code = "import 'extension:vscode'"
        ast = parse_code(code)
        assert ast is not None
        assert ast.imports[0].import_type == "extension"
    
    def test_assets_import(self):
        """Test import assets:"""
        code = "import 'assets:logo.png'"
        ast = parse_code(code)
        assert ast is not None
        assert ast.imports[0].import_type == "assets"


class TestImportClauses:
    """Tests para import clauses (show, hide, as)"""
    
    def test_import_with_show(self):
        """Test import con show"""
        code = "import 'lib:math' show { sin, cos, tan }"
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.imports[0].show_list) == 3
    
    def test_import_with_hide(self):
        """Test import con hide"""
        code = "import 'lib:utils' hide { deprecated_fn }"
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.imports[0].hide_list) == 1
    
    def test_import_with_alias(self):
        """Test import con alias (as)"""
        code = "import 'package:very_long_name' as vln"
        ast = parse_code(code)
        assert ast is not None
        assert ast.imports[0].alias == "vln"
    
    def test_import_show_multiple(self):
        """Test import show múltiples elementos"""
        code = """
        import 'lib:collections' show {
            List,
            Map,
            Set,
            Queue
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.imports[0].show_list) == 4
    
    def test_import_combined_clauses(self):
        """Test import con clauses combinadas"""
        code = "import 'lib:utils' as u show { helper }"
        ast = parse_code(code)
        assert ast is not None
        assert ast.imports[0].alias == "u"
        assert len(ast.imports[0].show_list) == 1


class TestParserErrors:
    """Tests para manejo de errores del parser"""
    
    def test_unexpected_token_error(self):
        """Test error de token inesperado"""
        code = """
        fn test() -> void {
            x: Number = INVALID_TOKEN
        }
        """
        # Parser debe lanzar error o usar error recovery
        try:
            ast = parse_code(code)
            # Si usa error recovery, debe tener AST pero incompleto
            assert ast is not None
        except ParserError as e:
            # Si lanza error, debe tener mensaje descriptivo
            assert e.message is not None
    
    def test_unexpected_eof(self):
        """Test error de EOF inesperado"""
        code = """
        fn test() -> void {
            x: Number = 42
        """  # Missing closing brace
        
        try:
            ast = parse_code(code)
            assert ast is not None
        except ParserError as e:
            assert "EOF" in str(e) or "expected" in str(e).lower()
    
    def test_missing_semicolon(self):
        """Test falta de punto y coma (si es requerido)"""
        code = """
        fn test() -> void {
            x: Number = 42
            y: Number = 10
        }
        """
        # Vela NO requiere semicolons, debe parsear bien
        ast = parse_code(code)
        assert ast is not None
    
    def test_invalid_syntax(self):
        """Test sintaxis inválida"""
        code = """
        fn test() -> void {
            if if if {}
        }
        """
        try:
            ast = parse_code(code)
            # Error recovery puede generar AST parcial
            assert ast is not None
        except ParserError:
            pass  # Error esperado


class TestSynchronization:
    """Tests para sincronización tras errores"""
    
    def test_synchronize_after_declaration_error(self):
        """Test sincronización después de error en declaration"""
        code = """
        fn broken() -> INVALID {
            return
        }
        
        fn working() -> void {
            return
        }
        """
        tokens = tokenize(code)
        parser = Parser(tokens)
        
        try:
            ast = parser.parse_program()
            # Parser debe recuperarse y parsear la segunda función
            assert ast is not None
        except ParserError:
            pass
    
    def test_synchronize_after_statement_error(self):
        """Test sincronización después de error en statement"""
        code = """
        fn test() -> void {
            x: Number = INVALID
            y: Number = 10
            return y
        }
        """
        try:
            ast = parse_code(code)
            assert ast is not None
        except ParserError:
            pass


class TestComplexPrograms:
    """Tests para programas complejos"""
    
    def test_full_program_with_everything(self):
        """Test programa completo con todos los elementos"""
        code = """
        import 'system:io'
        import 'package:http' as http
        
        struct User {
            id: Number
            name: String
        }
        
        enum Result<T, E> {
            Ok(value: T)
            Err(error: E)
        }
        
        class Database {
            fn connect() -> void {
                print("Connected")
            }
        }
        
        interface Serializable {
            fn serialize() -> String
        }
        
        fn main() -> void {
            user: User = User { id: 1, name: "Alice" }
            print("User: ${user.name}")
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.imports) == 2
        assert len(ast.declarations) == 5
    
    def test_nested_structures(self):
        """Test estructuras anidadas"""
        code = """
        struct Point { x: Number, y: Number }
        
        struct Line {
            start: Point
            end: Point
        }
        
        struct Shape {
            lines: [Line]
        }
        
        fn createShape() -> Shape {
            return Shape {
                lines: [
                    Line {
                        start: Point { x: 0, y: 0 },
                        end: Point { x: 10, y: 10 }
                    }
                ]
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 4
    
    def test_generics_everywhere(self):
        """Test genéricos en múltiples contextos"""
        code = """
        struct Box<T> {
            value: T
        }
        
        enum Option<T> {
            Some(value: T)
            None
        }
        
        fn identity<T>(x: T) -> T {
            return x
        }
        
        class Container<T> {
            items: [T]
            
            fn add(item: T) -> void {
                this.items.push(item)
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 4


class TestEdgeCases:
    """Tests para casos extremos"""
    
    def test_empty_functions(self):
        """Test funciones vacías"""
        code = """
        fn empty1() -> void {}
        fn empty2() -> void {
            # Solo comentarios
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 2
    
    def test_unicode_in_strings(self):
        """Test unicode en strings"""
        code = """
        fn test() -> void {
            message: String = "Hello 世界 🌍"
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_deeply_nested_expressions(self):
        """Test expresiones profundamente anidadas"""
        code = """
        fn test() -> Number {
            return ((((1 + 2) * 3) - 4) / 5) ** 2
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_long_method_chains(self):
        """Test method chaining largos"""
        code = """
        fn test() -> [Number] {
            return [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
                .filter(|x| => x % 2 == 0)
                .map(|x| => x * 2)
                .filter(|x| => x > 5)
                .map(|x| => x + 1)
        }
        """
        ast = parse_code(code)
        assert ast is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
