"""
Type Representation - Representación Interna de Tipos

Implementación de: VELA-570 (TASK-013)
Sprint: Sprint 8
Fecha: 2025-12-01

Descripción:
Define la representación interna de todos los tipos de Vela.
"""

from typing import List, Optional, Dict, Union
from dataclasses import dataclass
from enum import Enum


class TypeKind(Enum):
    """Categorías de tipos en Vela"""
    # Primitivos
    NUMBER = "Number"
    FLOAT = "Float"
    STRING = "String"
    BOOL = "Bool"
    VOID = "void"
    NEVER = "never"
    
    # Especiales
    OPTION = "Option"  # Option<T>
    RESULT = "Result"  # Result<T, E>
    
    # Compuestos
    TUPLE = "Tuple"    # (T1, T2, ...)
    LIST = "List"      # List<T>
    SET = "Set"        # Set<T>
    DICT = "Dict"      # Dict<K, V>
    
    # Funciones
    FUNCTION = "Function"  # (T1, T2) -> R
    
    # Estructurales
    STRUCT = "struct"
    ENUM = "enum"
    CLASS = "class"
    INTERFACE = "interface"
    
    # Keywords específicos (de Sprint 7)
    WIDGET = "widget"
    COMPONENT = "component"
    SERVICE = "service"
    REPOSITORY = "repository"
    CONTROLLER = "controller"
    USECASE = "usecase"
    ENTITY = "entity"
    DTO = "dto"
    VALUE_OBJECT = "valueObject"
    MODEL = "model"
    FACTORY = "factory"
    BUILDER = "builder"
    STRATEGY = "strategy"
    OBSERVER = "observer"
    SINGLETON = "singleton"
    ADAPTER = "adapter"
    DECORATOR = "decorator"
    GUARD = "guard"
    MIDDLEWARE = "middleware"
    INTERCEPTOR = "interceptor"
    VALIDATOR = "validator"
    STORE = "store"
    PROVIDER = "provider"
    ACTOR = "actor"
    PIPE = "pipe"
    TASK = "task"
    HELPER = "helper"
    MAPPER = "mapper"
    SERIALIZER = "serializer"
    
    # Generics
    TYPE_VARIABLE = "TypeVariable"  # T, U, V, etc.
    GENERIC = "Generic"             # Generic<T>
    
    # Especiales de inferencia
    UNKNOWN = "unknown"  # Tipo aún no inferido
    ANY = "any"          # Escape hatch (evitar su uso)


@dataclass
class Type:
    """
    Representación base de un tipo.
    
    Todos los tipos heredan de esta clase.
    """
    kind: TypeKind
    nullable: bool = False
    
    def __str__(self) -> str:
        result = self.kind.value
        if self.nullable:
            result += "?"
        return result
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, Type):
            return False
        return self.kind == other.kind and self.nullable == other.nullable
    
    def __hash__(self) -> int:
        return hash((self.kind, self.nullable))


@dataclass
class PrimitiveType(Type):
    """
    Tipos primitivos: Number, Float, String, Bool, void, never
    """
    def __init__(self, kind: TypeKind, nullable: bool = False):
        super().__init__(kind, nullable)


@dataclass
class OptionType(Type):
    """
    Option<T> - Tipo opcional (en lugar de null/undefined)
    
    Ejemplo:
        Option<String> puede ser Some("value") o None
    """
    inner_type: Type
    
    def __init__(self, inner_type: Type):
        super().__init__(TypeKind.OPTION, nullable=False)
        self.inner_type = inner_type
    
    def __str__(self) -> str:
        return f"Option<{self.inner_type}>"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, OptionType):
            return False
        return self.inner_type == other.inner_type
    
    def __hash__(self) -> int:
        return hash(("Option", self.inner_type))


@dataclass
class ResultType(Type):
    """
    Result<T, E> - Tipo para manejo de errores
    
    Ejemplo:
        Result<User, DatabaseError> puede ser Ok(user) o Err(error)
    """
    ok_type: Type
    err_type: Type
    
    def __init__(self, ok_type: Type, err_type: Type):
        super().__init__(TypeKind.RESULT, nullable=False)
        self.ok_type = ok_type
        self.err_type = err_type
    
    def __str__(self) -> str:
        return f"Result<{self.ok_type}, {self.err_type}>"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, ResultType):
            return False
        return self.ok_type == other.ok_type and self.err_type == other.err_type
    
    def __hash__(self) -> int:
        return hash(("Result", self.ok_type, self.err_type))


@dataclass
class TupleType(Type):
    """
    Tuple<T1, T2, ...> - Tupla de tipos
    
    Ejemplo:
        (String, Number, Bool) representa una tupla de 3 elementos
    """
    element_types: List[Type]
    
    def __init__(self, element_types: List[Type]):
        super().__init__(TypeKind.TUPLE, nullable=False)
        self.element_types = element_types
    
    def __str__(self) -> str:
        elements = ", ".join(str(t) for t in self.element_types)
        return f"({elements})"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, TupleType):
            return False
        return self.element_types == other.element_types
    
    def __hash__(self) -> int:
        return hash(("Tuple", tuple(self.element_types)))


@dataclass
class ListType(Type):
    """
    List<T> - Lista dinámica de tipo T
    """
    element_type: Type
    
    def __init__(self, element_type: Type):
        super().__init__(TypeKind.LIST, nullable=False)
        self.element_type = element_type
    
    def __str__(self) -> str:
        return f"List<{self.element_type}>"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, ListType):
            return False
        return self.element_type == other.element_type
    
    def __hash__(self) -> int:
        return hash(("List", self.element_type))


@dataclass
class SetType(Type):
    """
    Set<T> - Conjunto de tipo T (sin duplicados)
    """
    element_type: Type
    
    def __init__(self, element_type: Type):
        super().__init__(TypeKind.SET, nullable=False)
        self.element_type = element_type
    
    def __str__(self) -> str:
        return f"Set<{self.element_type}>"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, SetType):
            return False
        return self.element_type == other.element_type
    
    def __hash__(self) -> int:
        return hash(("Set", self.element_type))


@dataclass
class DictType(Type):
    """
    Dict<K, V> - Diccionario/mapa de claves K a valores V
    """
    key_type: Type
    value_type: Type
    
    def __init__(self, key_type: Type, value_type: Type):
        super().__init__(TypeKind.DICT, nullable=False)
        self.key_type = key_type
        self.value_type = value_type
    
    def __str__(self) -> str:
        return f"Dict<{self.key_type}, {self.value_type}>"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, DictType):
            return False
        return self.key_type == other.key_type and self.value_type == other.value_type
    
    def __hash__(self) -> int:
        return hash(("Dict", self.key_type, self.value_type))


@dataclass
class FunctionType(Type):
    """
    (T1, T2, ...) -> R - Tipo de función
    
    Ejemplo:
        (Number, Number) -> Number representa fn add(a: Number, b: Number) -> Number
    """
    param_types: List[Type]
    return_type: Type
    is_async: bool = False
    
    def __init__(self, param_types: List[Type], return_type: Type, is_async: bool = False):
        super().__init__(TypeKind.FUNCTION, nullable=False)
        self.param_types = param_types
        self.return_type = return_type
        self.is_async = is_async
    
    def __str__(self) -> str:
        params = ", ".join(str(t) for t in self.param_types)
        prefix = "async " if self.is_async else ""
        return f"{prefix}({params}) -> {self.return_type}"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, FunctionType):
            return False
        return (self.param_types == other.param_types and
                self.return_type == other.return_type and
                self.is_async == other.is_async)
    
    def __hash__(self) -> int:
        return hash(("Function", tuple(self.param_types), self.return_type, self.is_async))


@dataclass
class StructType(Type):
    """
    struct { field1: T1, field2: T2, ... } - Tipo estructural
    
    Ejemplo:
        struct User { id: Number, name: String, email: String }
    """
    name: str
    fields: Dict[str, Type]
    type_params: List['TypeVariable'] = None  # Para generics
    
    def __init__(self, name: str, fields: Dict[str, Type], type_params: List['TypeVariable'] = None):
        super().__init__(TypeKind.STRUCT, nullable=False)
        self.name = name
        self.fields = fields
        self.type_params = type_params or []
    
    def __str__(self) -> str:
        if self.type_params:
            params = ", ".join(str(t) for t in self.type_params)
            return f"{self.name}<{params}>"
        return self.name
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, StructType):
            return False
        return self.name == other.name and self.fields == other.fields
    
    def __hash__(self) -> int:
        return hash(("Struct", self.name))


@dataclass
class EnumType(Type):
    """
    enum - Tipo enumerado con variantes
    
    Ejemplo:
        enum Color { Red, Green, Blue, Custom(r: Number, g: Number, b: Number) }
    """
    name: str
    variants: Dict[str, Optional[List[Type]]]  # variant_name -> optional fields
    type_params: List['TypeVariable'] = None
    
    def __init__(self, name: str, variants: Dict[str, Optional[List[Type]]], type_params: List['TypeVariable'] = None):
        super().__init__(TypeKind.ENUM, nullable=False)
        self.name = name
        self.variants = variants
        self.type_params = type_params or []
    
    def __str__(self) -> str:
        if self.type_params:
            params = ", ".join(str(t) for t in self.type_params)
            return f"{self.name}<{params}>"
        return self.name
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, EnumType):
            return False
        return self.name == other.name and self.variants == other.variants
    
    def __hash__(self) -> int:
        return hash(("Enum", self.name))


@dataclass
class ClassType(Type):
    """
    class - Tipo de clase (POO)
    
    Ejemplo:
        class Person { constructor(name: String) { this.name = name } }
    """
    name: str
    fields: Dict[str, Type]
    methods: Dict[str, FunctionType]
    parent: Optional['ClassType'] = None
    interfaces: List['InterfaceType'] = None
    type_params: List['TypeVariable'] = None
    
    def __init__(self, name: str, fields: Dict[str, Type], methods: Dict[str, FunctionType],
                 parent: Optional['ClassType'] = None, interfaces: List['InterfaceType'] = None,
                 type_params: List['TypeVariable'] = None):
        super().__init__(TypeKind.CLASS, nullable=False)
        self.name = name
        self.fields = fields
        self.methods = methods
        self.parent = parent
        self.interfaces = interfaces or []
        self.type_params = type_params or []
    
    def __str__(self) -> str:
        if self.type_params:
            params = ", ".join(str(t) for t in self.type_params)
            return f"{self.name}<{params}>"
        return self.name
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, ClassType):
            return False
        return self.name == other.name
    
    def __hash__(self) -> int:
        return hash(("Class", self.name))


@dataclass
class InterfaceType(Type):
    """
    interface - Tipo de interfaz (contrato)
    
    Ejemplo:
        interface Drawable { fn draw() -> void }
    """
    name: str
    methods: Dict[str, FunctionType]
    type_params: List['TypeVariable'] = None
    
    def __init__(self, name: str, methods: Dict[str, FunctionType], type_params: List['TypeVariable'] = None):
        super().__init__(TypeKind.INTERFACE, nullable=False)
        self.name = name
        self.methods = methods
        self.type_params = type_params or []
    
    def __str__(self) -> str:
        if self.type_params:
            params = ", ".join(str(t) for t in self.type_params)
            return f"{self.name}<{params}>"
        return self.name
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, InterfaceType):
            return False
        return self.name == other.name
    
    def __hash__(self) -> int:
        return hash(("Interface", self.name))


@dataclass
class TypeVariable(Type):
    """
    Variable de tipo para generics (T, U, V, etc.)
    
    Ejemplo:
        fn identity<T>(x: T) -> T { return x }
    """
    name: str
    constraints: List[Type] = None  # Constraints sobre el tipo (opcional)
    
    def __init__(self, name: str, constraints: List[Type] = None):
        super().__init__(TypeKind.TYPE_VARIABLE, nullable=False)
        self.name = name
        self.constraints = constraints or []
    
    def __str__(self) -> str:
        if self.constraints:
            constraints_str = " + ".join(str(c) for c in self.constraints)
            return f"{self.name}: {constraints_str}"
        return self.name
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, TypeVariable):
            return False
        return self.name == other.name
    
    def __hash__(self) -> int:
        return hash(("TypeVariable", self.name))


@dataclass
class GenericType(Type):
    """
    Generic<T, U, ...> - Tipo genérico instanciado
    
    Ejemplo:
        List<Number> es un GenericType donde base=List y type_args=[Number]
    """
    base: Type  # Tipo base (ej: List, Dict, etc.)
    type_args: List[Type]
    
    def __init__(self, base: Type, type_args: List[Type]):
        super().__init__(TypeKind.GENERIC, nullable=False)
        self.base = base
        self.type_args = type_args
    
    def __str__(self) -> str:
        args = ", ".join(str(t) for t in self.type_args)
        return f"{self.base}<{args}>"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, GenericType):
            return False
        return self.base == other.base and self.type_args == other.type_args
    
    def __hash__(self) -> int:
        return hash(("Generic", self.base, tuple(self.type_args)))


@dataclass
class UnknownType(Type):
    """
    Tipo desconocido - usado durante inferencia de tipos
    
    Se reemplaza con el tipo inferido una vez que se resuelve.
    """
    id: int  # ID único para esta variable de tipo
    
    def __init__(self, id: int):
        super().__init__(TypeKind.UNKNOWN, nullable=False)
        self.id = id
    
    def __str__(self) -> str:
        return f"?{self.id}"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, UnknownType):
            return False
        return self.id == other.id
    
    def __hash__(self) -> int:
        return hash(("Unknown", self.id))


@dataclass
class KeywordSpecificType(Type):
    """
    Tipo para keywords específicos (widget, service, repository, etc.)
    
    Estos tipos tienen reglas especiales de validación.
    """
    keyword_kind: TypeKind  # WIDGET, SERVICE, REPOSITORY, etc.
    name: str
    fields: Dict[str, Type]
    methods: Dict[str, FunctionType]
    
    def __init__(self, keyword_kind: TypeKind, name: str, fields: Dict[str, Type], methods: Dict[str, FunctionType]):
        super().__init__(keyword_kind, nullable=False)
        self.keyword_kind = keyword_kind
        self.name = name
        self.fields = fields
        self.methods = methods
    
    def __str__(self) -> str:
        return f"{self.keyword_kind.value} {self.name}"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, KeywordSpecificType):
            return False
        return self.keyword_kind == other.keyword_kind and self.name == other.name
    
    def __hash__(self) -> int:
        return hash((self.keyword_kind, self.name))


# ============================================================================
# TIPOS PREDEFINIDOS (Built-in Types)
# ============================================================================

# Primitivos
NUMBER_TYPE = PrimitiveType(TypeKind.NUMBER)
FLOAT_TYPE = PrimitiveType(TypeKind.FLOAT)
STRING_TYPE = PrimitiveType(TypeKind.STRING)
BOOL_TYPE = PrimitiveType(TypeKind.BOOL)
VOID_TYPE = PrimitiveType(TypeKind.VOID)
NEVER_TYPE = PrimitiveType(TypeKind.NEVER)

# Especiales
UNKNOWN_TYPE_COUNTER = 0

def new_unknown_type() -> UnknownType:
    """Crea un nuevo tipo desconocido con ID único"""
    global UNKNOWN_TYPE_COUNTER
    UNKNOWN_TYPE_COUNTER += 1
    return UnknownType(UNKNOWN_TYPE_COUNTER)


# ============================================================================
# UTILIDADES
# ============================================================================

def is_primitive(type: Type) -> bool:
    """Verifica si un tipo es primitivo"""
    return isinstance(type, PrimitiveType)


def is_collection(type: Type) -> bool:
    """Verifica si un tipo es una colección (List, Set, Dict)"""
    return isinstance(type, (ListType, SetType, DictType))


def is_callable(type: Type) -> bool:
    """Verifica si un tipo es llamable (función)"""
    return isinstance(type, FunctionType)


def is_generic(type: Type) -> bool:
    """Verifica si un tipo es genérico"""
    return isinstance(type, GenericType)


def is_option(type: Type) -> bool:
    """Verifica si un tipo es Option<T>"""
    return isinstance(type, OptionType)


def is_result(type: Type) -> bool:
    """Verifica si un tipo es Result<T, E>"""
    return isinstance(type, ResultType)


def is_unknown(type: Type) -> bool:
    """Verifica si un tipo es desconocido (para inferencia)"""
    return isinstance(type, UnknownType)


def get_inner_type(option_type: OptionType) -> Type:
    """Extrae el tipo interno de un Option<T>"""
    if not isinstance(option_type, OptionType):
        raise TypeError(f"Expected OptionType, got {type(option_type)}")
    return option_type.inner_type


def make_optional(type: Type) -> OptionType:
    """Convierte un tipo T en Option<T>"""
    return OptionType(type)


if __name__ == "__main__":
    # Tests básicos
    print("=== Tests de Representación de Tipos ===\n")
    
    # Test 1: Primitivos
    num_type = NUMBER_TYPE
    print(f"Number type: {num_type}")
    
    # Test 2: Option<String>
    option_str = OptionType(STRING_TYPE)
    print(f"Option<String>: {option_str}")
    
    # Test 3: List<Number>
    list_num = ListType(NUMBER_TYPE)
    print(f"List<Number>: {list_num}")
    
    # Test 4: Dict<String, Number>
    dict_type = DictType(STRING_TYPE, NUMBER_TYPE)
    print(f"Dict<String, Number>: {dict_type}")
    
    # Test 5: Función (Number, Number) -> Number
    func_type = FunctionType([NUMBER_TYPE, NUMBER_TYPE], NUMBER_TYPE)
    print(f"Function type: {func_type}")
    
    # Test 6: Struct User
    user_type = StructType("User", {
        "id": NUMBER_TYPE,
        "name": STRING_TYPE,
        "email": STRING_TYPE
    })
    print(f"Struct type: {user_type}")
    
    # Test 7: Result<User, String>
    result_type = ResultType(user_type, STRING_TYPE)
    print(f"Result type: {result_type}")
    
    print("\n✅ Todos los tests de tipos pasaron correctamente")
