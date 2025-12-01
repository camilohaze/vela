"""
Tests Unit arios del Type System

Implementación de: VELA-570 (TASK-020)
Sprint: Sprint 8
Fecha: 2025-12-01

Suite completa de tests para el sistema de tipos de Vela.
"""

import pytest
import sys
import os

# Agregar src/ al path para imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..', '..', 'src'))

from type_system.types import *
from type_system.inference import *
from type_system.env import *
from type_system.checker import *


# ============================================================================
# TESTS DE REPRESENTACIÓN DE TIPOS
# ============================================================================

class TestTypeRepresentation:
    """Tests de representación de tipos"""
    
    def test_primitive_types(self):
        """Test de tipos primitivos"""
        assert str(NUMBER_TYPE) == "Number"
        assert str(STRING_TYPE) == "String"
        assert str(BOOL_TYPE) == "Bool"
        assert str(VOID_TYPE) == "void"
        assert str(NEVER_TYPE) == "never"
    
    def test_option_type(self):
        """Test de Option<T>"""
        opt_str = OptionType(STRING_TYPE)
        assert str(opt_str) == "Option<String>"
        assert opt_str.inner_type == STRING_TYPE
    
    def test_result_type(self):
        """Test de Result<T, E>"""
        result = ResultType(NUMBER_TYPE, STRING_TYPE)
        assert str(result) == "Result<Number, String>"
        assert result.ok_type == NUMBER_TYPE
        assert result.err_type == STRING_TYPE
    
    def test_list_type(self):
        """Test de List<T>"""
        list_num = ListType(NUMBER_TYPE)
        assert str(list_num) == "List<Number>"
        assert list_num.element_type == NUMBER_TYPE
    
    def test_dict_type(self):
        """Test de Dict<K, V>"""
        dict_type = DictType(STRING_TYPE, NUMBER_TYPE)
        assert str(dict_type) == "Dict<String, Number>"
        assert dict_type.key_type == STRING_TYPE
        assert dict_type.value_type == NUMBER_TYPE
    
    def test_function_type(self):
        """Test de tipos de función"""
        func = FunctionType([NUMBER_TYPE, NUMBER_TYPE], NUMBER_TYPE)
        assert str(func) == "(Number, Number) -> Number"
        
        async_func = FunctionType([STRING_TYPE], VOID_TYPE, is_async=True)
        assert str(async_func) == "async (String) -> void"
    
    def test_tuple_type(self):
        """Test de tuplas"""
        tuple_type = TupleType([NUMBER_TYPE, STRING_TYPE, BOOL_TYPE])
        assert str(tuple_type) == "(Number, String, Bool)"
    
    def test_struct_type(self):
        """Test de structs"""
        user = StructType("User", {
            "id": NUMBER_TYPE,
            "name": STRING_TYPE,
            "email": STRING_TYPE
        })
        assert str(user) == "User"
        assert "name" in user.fields
        assert user.fields["name"] == STRING_TYPE
    
    def test_enum_type(self):
        """Test de enums"""
        color = EnumType("Color", {
            "Red": None,
            "Green": None,
            "Blue": None,
            "Custom": [NUMBER_TYPE, NUMBER_TYPE, NUMBER_TYPE]
        })
        assert str(color) == "Color"
        assert "Red" in color.variants
    
    def test_type_variable(self):
        """Test de variables de tipo"""
        T = TypeVariable("T")
        assert str(T) == "T"
        
        T_constrained = TypeVariable("T", constraints=[NUMBER_TYPE])
        assert "Number" in str(T_constrained)
    
    def test_unknown_type(self):
        """Test de tipos desconocidos"""
        unknown = new_unknown_type()
        assert is_unknown(unknown)
        assert "?" in str(unknown)


# ============================================================================
# TESTS DE UNIFICACIÓN
# ============================================================================

class TestUnification:
    """Tests del algoritmo de unificación"""
    
    def test_unify_identical_types(self):
        """Test unificar tipos idénticos"""
        s = unify(NUMBER_TYPE, NUMBER_TYPE)
        assert len(s.mapping) == 0  # Sustitución vacía
    
    def test_unify_type_variable_with_concrete(self):
        """Test unificar variable de tipo con tipo concreto"""
        T = TypeVariable("T")
        s = unify(T, NUMBER_TYPE)
        assert s.apply(T) == NUMBER_TYPE
    
    def test_unify_list_types(self):
        """Test unificar List<T> con List<Number>"""
        T = TypeVariable("T")
        list_t = ListType(T)
        list_num = ListType(NUMBER_TYPE)
        
        s = unify(list_t, list_num)
        assert s.apply(list_t) == list_num
    
    def test_unify_function_types(self):
        """Test unificar tipos de función"""
        T = TypeVariable("T")
        func_t = FunctionType([T, T], T)
        func_num = FunctionType([NUMBER_TYPE, NUMBER_TYPE], NUMBER_TYPE)
        
        s = unify(func_t, func_num)
        assert s.apply(func_t) == func_num
    
    def test_unify_dict_types(self):
        """Test unificar Dict<K, V>"""
        K = TypeVariable("K")
        V = TypeVariable("V")
        dict_generic = DictType(K, V)
        dict_concrete = DictType(STRING_TYPE, NUMBER_TYPE)
        
        s = unify(dict_generic, dict_concrete)
        assert s.apply(dict_generic) == dict_concrete
    
    def test_unify_incompatible_types_error(self):
        """Test error al unificar tipos incompatibles"""
        with pytest.raises(UnificationError):
            unify(NUMBER_TYPE, STRING_TYPE)
    
    def test_occurs_check_error(self):
        """Test error en occurs check"""
        T = TypeVariable("T")
        list_t = ListType(T)
        
        with pytest.raises(UnificationError):
            unify(T, list_t)
    
    def test_unify_option_types(self):
        """Test unificar Option<T>"""
        T = TypeVariable("T")
        opt_t = OptionType(T)
        opt_num = OptionType(NUMBER_TYPE)
        
        s = unify(opt_t, opt_num)
        assert s.apply(opt_t) == opt_num
    
    def test_unify_result_types(self):
        """Test unificar Result<T, E>"""
        T = TypeVariable("T")
        E = TypeVariable("E")
        result_generic = ResultType(T, E)
        result_concrete = ResultType(NUMBER_TYPE, STRING_TYPE)
        
        s = unify(result_generic, result_concrete)
        assert s.apply(result_generic) == result_concrete
    
    def test_composition_of_substitutions(self):
        """Test composición de sustituciones"""
        T = TypeVariable("T")
        U = TypeVariable("U")
        
        s1 = Substitution({T: NUMBER_TYPE})
        s2 = Substitution({U: T})
        
        composed = s1.compose(s2)
        assert composed.apply(U) == NUMBER_TYPE


# ============================================================================
# TESTS DE TYPE ENVIRONMENT
# ============================================================================

class TestTypeEnvironment:
    """Tests del entorno de tipos"""
    
    def test_define_and_lookup(self):
        """Test definir y buscar símbolos"""
        env = TypeEnvironment()
        env.define("x", Symbol("x", NUMBER_TYPE))
        
        assert env.is_defined("x")
        symbol = env.lookup("x")
        assert symbol is not None
        assert symbol.type == NUMBER_TYPE
    
    def test_nested_scopes(self):
        """Test scopes anidados"""
        env = TypeEnvironment()
        env.define("x", Symbol("x", NUMBER_TYPE))
        
        env.enter_scope()
        env.define("y", Symbol("y", STRING_TYPE))
        
        assert env.is_defined("x")  # Visible desde scope padre
        assert env.is_defined("y")  # Visible en scope actual
        
        env.exit_scope()
        assert env.is_defined("x")  # Aún visible
        assert not env.is_defined("y")  # Ya no visible
    
    def test_shadowing(self):
        """Test shadowing de variables"""
        env = TypeEnvironment()
        env.define("x", Symbol("x", NUMBER_TYPE))
        
        env.enter_scope()
        env.define("x", Symbol("x", STRING_TYPE))  # Shadow
        
        assert env.lookup("x").type == STRING_TYPE  # Nueva x
        
        env.exit_scope()
        assert env.lookup("x").type == NUMBER_TYPE  # x original
    
    def test_mutable_variables(self):
        """Test variables mutables (state)"""
        env = TypeEnvironment()
        env.define("count", Symbol("count", NUMBER_TYPE, mutable=True))
        env.define("name", Symbol("name", STRING_TYPE, mutable=False))
        
        assert env.is_mutable("count")
        assert not env.is_mutable("name")
    
    def test_duplicate_definition_error(self):
        """Test error al redefinir en mismo scope"""
        env = TypeEnvironment()
        env.define("x", Symbol("x", NUMBER_TYPE))
        
        with pytest.raises(NameError):
            env.define("x", Symbol("x", STRING_TYPE))
    
    def test_undefined_variable_lookup(self):
        """Test buscar variable indefinida"""
        env = TypeEnvironment()
        assert env.lookup("undefined") is None
        assert not env.is_defined("undefined")


# ============================================================================
# TESTS DE TYPE CHECKER
# ============================================================================

class TestTypeChecker:
    """Tests del type checker"""
    
    def setup_method(self):
        """Setup para cada test"""
        self.checker = TypeChecker()
    
    def test_literal_inference(self):
        """Test inferencia de literales"""
        from dataclasses import dataclass
        
        @dataclass
        class Literal:
            value: any
        
        # Number literal
        expr1 = Literal(42)
        type1 = self.checker.check_expression(expr1)
        assert type1 == NUMBER_TYPE
        
        # String literal
        expr2 = Literal("hello")
        type2 = self.checker.check_expression(expr2)
        assert type2 == STRING_TYPE
        
        # Bool literal
        expr3 = Literal(True)
        type3 = self.checker.check_expression(expr3)
        assert type3 == BOOL_TYPE
    
    def test_arithmetic_operations(self):
        """Test operaciones aritméticas"""
        from dataclasses import dataclass
        
        @dataclass
        class Literal:
            value: any
        
        @dataclass
        class BinaryOp:
            operator: str
            left: any
            right: any
        
        # 10 + 20
        expr = BinaryOp("+", Literal(10), Literal(20))
        result_type = self.checker.check_binary_op(expr)
        assert result_type == NUMBER_TYPE
        
        # 5 * 3
        expr2 = BinaryOp("*", Literal(5), Literal(3))
        result_type2 = self.checker.check_binary_op(expr2)
        assert result_type2 == NUMBER_TYPE
    
    def test_comparison_operations(self):
        """Test operaciones de comparación"""
        from dataclasses import dataclass
        
        @dataclass
        class Literal:
            value: any
        
        @dataclass
        class BinaryOp:
            operator: str
            left: any
            right: any
        
        # 10 == 20
        expr = BinaryOp("==", Literal(10), Literal(20))
        result_type = self.checker.check_binary_op(expr)
        assert result_type == BOOL_TYPE
        
        # "hello" != "world"
        expr2 = BinaryOp("!=", Literal("hello"), Literal("world"))
        result_type2 = self.checker.check_binary_op(expr2)
        assert result_type2 == BOOL_TYPE
    
    def test_logical_operations(self):
        """Test operaciones lógicas"""
        from dataclasses import dataclass
        
        @dataclass
        class Literal:
            value: any
        
        @dataclass
        class BinaryOp:
            operator: str
            left: any
            right: any
        
        # true and false
        expr = BinaryOp("and", Literal(True), Literal(False))
        result_type = self.checker.check_binary_op(expr)
        assert result_type == BOOL_TYPE
    
    def test_type_error_detection(self):
        """Test detección de errores de tipo"""
        from dataclasses import dataclass
        
        @dataclass
        class Literal:
            value: any
        
        @dataclass
        class BinaryOp:
            operator: str
            left: any
            right: any
        
        # 10 + "hello" (error)
        expr = BinaryOp("+", Literal(10), Literal("hello"))
        self.checker.check_binary_op(expr)
        
        assert self.checker.has_errors()
        assert "requires Number" in self.checker.errors[-1]
    
    def test_variable_declaration(self):
        """Test declaración de variables"""
        from dataclasses import dataclass
        
        @dataclass
        class Literal:
            value: any
        
        @dataclass
        class VarDecl:
            name: str
            initializer: any
            is_state: bool = False
        
        # name: String = "Vela"
        stmt = VarDecl("name", Literal("Vela"))
        self.checker.check_var_declaration(stmt)
        
        assert self.checker.env.is_defined("name")
        assert self.checker.env.lookup("name").type == STRING_TYPE
    
    def test_state_variable_mutability(self):
        """Test variables state (mutables)"""
        from dataclasses import dataclass
        
        @dataclass
        class Literal:
            value: any
        
        @dataclass
        class VarDecl:
            name: str
            initializer: any
            is_state: bool = False
        
        # state count: Number = 0
        stmt = VarDecl("count", Literal(0), is_state=True)
        self.checker.check_var_declaration(stmt)
        
        assert self.checker.env.is_mutable("count")


# ============================================================================
# TESTS DE GENERICS
# ============================================================================

class TestGenerics:
    """Tests de soporte para generics (TASK-017)"""
    
    def test_generic_list(self):
        """Test List<T> genérico"""
        T = TypeVariable("T")
        list_t = ListType(T)
        
        # Instanciar con Number
        s = unify(T, NUMBER_TYPE)
        list_num = s.apply(list_t)
        
        assert list_num == ListType(NUMBER_TYPE)
    
    def test_generic_function(self):
        """Test función genérica"""
        T = TypeVariable("T")
        identity = FunctionType([T], T)
        
        # Llamar con Number
        s = unify(T, NUMBER_TYPE)
        identity_num = s.apply(identity)
        
        assert identity_num == FunctionType([NUMBER_TYPE], NUMBER_TYPE)
    
    def test_multiple_type_variables(self):
        """Test múltiples variables de tipo"""
        K = TypeVariable("K")
        V = TypeVariable("V")
        dict_kv = DictType(K, V)
        
        # Instanciar
        s1 = unify(K, STRING_TYPE)
        s2 = unify(V, NUMBER_TYPE)
        composed = s1.compose(s2)
        
        dict_concrete = composed.apply(dict_kv)
        assert dict_concrete == DictType(STRING_TYPE, NUMBER_TYPE)


# ============================================================================
# TESTS DE OPTION<T> SAFETY
# ============================================================================

class TestOptionSafety:
    """Tests de Option<T> safety (TASK-018)"""
    
    def test_option_type_creation(self):
        """Test crear Option<T>"""
        opt_num = OptionType(NUMBER_TYPE)
        assert opt_num.inner_type == NUMBER_TYPE
    
    def test_make_optional(self):
        """Test hacer tipo opcional"""
        opt = make_optional(STRING_TYPE)
        assert isinstance(opt, OptionType)
        assert opt.inner_type == STRING_TYPE
    
    def test_get_inner_type(self):
        """Test extraer tipo interno"""
        opt = OptionType(NUMBER_TYPE)
        inner = get_inner_type(opt)
        assert inner == NUMBER_TYPE
    
    def test_option_unification(self):
        """Test unificación de Option<T>"""
        T = TypeVariable("T")
        opt_t = OptionType(T)
        opt_num = OptionType(NUMBER_TYPE)
        
        s = unify(opt_t, opt_num)
        assert s.apply(opt_t) == opt_num


# ============================================================================
# MAIN
# ============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
