"""
Tests para Declarations del Parser de Vela

Tests de: VELA-568 (TASK-012)
Historia: Sprint 6 - Parser que genere AST válido
Fecha: 2025-12-01

Este módulo testea el parsing de declarations:
- Functions (normales, async, con generics)
- Structs
- Enums
- Classes (con herencia, interfaces)
- Interfaces
- Type aliases
- Domain-specific (service, repository, controller, etc.)
"""

import pytest
import sys
sys.path.append('../..')

from src.parser import parse_code
from src.parser.ast_nodes import (
    FunctionDeclaration, StructDeclaration, EnumDeclaration,
    ClassDeclaration, InterfaceDeclaration, TypeAliasDeclaration,
    ServiceDeclaration, RepositoryDeclaration, ControllerDeclaration
)


class TestFunctionDeclarations:
    """Tests para function declarations"""
    
    def test_simple_function(self):
        """Test función simple"""
        code = """
        fn add(a: Number, b: Number) -> Number {
            return a + b
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], FunctionDeclaration)
        assert ast.declarations[0].name == "add"
    
    def test_function_no_params(self):
        """Test función sin parámetros"""
        code = """
        fn hello() -> void {
            print("Hello!")
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations[0].parameters) == 0
    
    def test_function_no_return_type(self):
        """Test función sin tipo de retorno"""
        code = """
        fn log(message: String) {
            print(message)
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_async_function(self):
        """Test función async"""
        code = """
        async fn fetchData() -> Result<String> {
            return await http.get("/api/data")
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert ast.declarations[0].is_async == True
    
    def test_public_function(self):
        """Test función pública"""
        code = """
        public fn publicAPI() -> String {
            return "public"
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert ast.declarations[0].is_public == True
    
    def test_generic_function(self):
        """Test función genérica"""
        code = """
        fn identity<T>(value: T) -> T {
            return value
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations[0].generic_params) == 1
    
    def test_function_with_default_params(self):
        """Test función con parámetros por defecto"""
        code = """
        fn greet(name: String = "World") -> String {
            return "Hello, ${name}!"
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestStructDeclarations:
    """Tests para struct declarations"""
    
    def test_simple_struct(self):
        """Test struct simple"""
        code = """
        struct Point {
            x: Number
            y: Number
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], StructDeclaration)
        assert ast.declarations[0].name == "Point"
        assert len(ast.declarations[0].fields) == 2
    
    def test_empty_struct(self):
        """Test struct vacío"""
        code = """
        struct Empty {}
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations[0].fields) == 0
    
    def test_generic_struct(self):
        """Test struct genérico"""
        code = """
        struct Box<T> {
            value: T
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations[0].generic_params) == 1
    
    def test_public_struct(self):
        """Test struct público"""
        code = """
        public struct User {
            id: Number
            name: String
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert ast.declarations[0].is_public == True


class TestEnumDeclarations:
    """Tests para enum declarations"""
    
    def test_simple_enum(self):
        """Test enum simple"""
        code = """
        enum Color {
            Red
            Green
            Blue
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert isinstance(ast.declarations[0], EnumDeclaration)
        assert len(ast.declarations[0].variants) == 3
    
    def test_enum_with_data(self):
        """Test enum con datos asociados"""
        code = """
        enum Option<T> {
            Some(value: T)
            None
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations[0].generic_params) == 1
    
    def test_enum_complex_variants(self):
        """Test enum con variantes complejas"""
        code = """
        enum Result<T, E> {
            Ok(value: T)
            Err(error: E)
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestClassDeclarations:
    """Tests para class declarations"""
    
    def test_simple_class(self):
        """Test clase simple"""
        code = """
        class Person {
            name: String
            age: Number
            
            constructor(name: String, age: Number) {
                this.name = name
                this.age = age
            }
            
            fn greet() -> String {
                return "Hello, I'm ${this.name}"
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert isinstance(ast.declarations[0], ClassDeclaration)
        assert ast.declarations[0].name == "Person"
    
    def test_class_with_inheritance(self):
        """Test clase con herencia"""
        code = """
        class Animal {
            name: String
        }
        
        class Dog extends Animal {
            breed: String
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 2
        assert ast.declarations[1].extends == "Animal"
    
    def test_class_with_interface(self):
        """Test clase implementando interface"""
        code = """
        interface Drawable {
            fn draw() -> void
        }
        
        class Circle implements Drawable {
            radius: Number
            
            fn draw() -> void {
                print("Drawing circle")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert "Drawable" in ast.declarations[1].implements
    
    def test_class_state_field(self):
        """Test clase con campo state"""
        code = """
        class Counter {
            state count: Number = 0
            
            fn increment() -> void {
                this.count = this.count + 1
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestInterfaceDeclarations:
    """Tests para interface declarations"""
    
    def test_simple_interface(self):
        """Test interface simple"""
        code = """
        interface Comparable {
            fn compareTo(other: Self) -> Number
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert isinstance(ast.declarations[0], InterfaceDeclaration)
        assert len(ast.declarations[0].methods) == 1
    
    def test_generic_interface(self):
        """Test interface genérico"""
        code = """
        interface Iterator<T> {
            fn next() -> Option<T>
            fn hasNext() -> Bool
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations[0].generic_params) == 1


class TestTypeAliases:
    """Tests para type aliases"""
    
    def test_simple_type_alias(self):
        """Test alias simple"""
        code = """
        type UserId = Number
        """
        ast = parse_code(code)
        assert ast is not None
        assert isinstance(ast.declarations[0], TypeAliasDeclaration)
        assert ast.declarations[0].name == "UserId"
    
    def test_union_type_alias(self):
        """Test alias con union type"""
        code = """
        type Status = "active" | "inactive" | "pending"
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_function_type_alias(self):
        """Test alias de función"""
        code = """
        type Callback = (data: String) -> void
        """
        ast = parse_code(code)
        assert ast is not None


class TestDomainSpecificDeclarations:
    """Tests para domain-specific declarations"""
    
    def test_service_declaration(self):
        """Test service"""
        code = """
        service UserService {
            fn createUser(name: String) -> Result<User> {
                return Ok(User { id: 1, name: name })
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert isinstance(ast.declarations[0], ServiceDeclaration)
    
    def test_repository_declaration(self):
        """Test repository"""
        code = """
        repository<User> UserRepository {
            fn findById(id: Number) -> Option<User> {
                return None
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert isinstance(ast.declarations[0], RepositoryDeclaration)
    
    def test_controller_declaration(self):
        """Test controller"""
        code = """
        controller UserController {
            # Routes here
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert isinstance(ast.declarations[0], ControllerDeclaration)
    
    def test_entity_declaration(self):
        """Test entity"""
        code = """
        entity User {
            id: Number
            name: String
            email: String
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_dto_declaration(self):
        """Test DTO"""
        code = """
        dto CreateUserDTO {
            name: String
            email: String
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestComplexDeclarations:
    """Tests para declarations complejas"""
    
    def test_multiple_declarations(self):
        """Test múltiples declarations"""
        code = """
        struct Point { x: Number, y: Number }
        
        fn distance(p1: Point, p2: Point) -> Float {
            dx: Number = p2.x - p1.x
            dy: Number = p2.y - p1.y
            return sqrt(dx * dx + dy * dy)
        }
        
        class Shape {
            fn area() -> Float
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 3
    
    def test_nested_generic_types(self):
        """Test tipos genéricos anidados"""
        code = """
        fn process<T>(items: [Option<T>]) -> [T] {
            return items
                .filter(|x| => x.isSome())
                .map(|x| => x.unwrap())
        }
        """
        ast = parse_code(code)
        assert ast is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
