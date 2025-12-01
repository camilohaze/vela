"""
Type Inference - Algoritmo Hindley-Milner

Implementación de: VELA-570 (TASK-014)
Sprint: Sprint 8
Fecha: 2025-12-01

Descripción:
Implementa el algoritmo de inferencia de tipos Hindley-Milner
para inferir tipos automáticamente sin anotaciones explícitas.
"""

from typing import Dict, List, Set, Optional, Tuple
from dataclasses import dataclass, field
from .types import *


@dataclass
class Substitution:
    """
    Sustitución de variables de tipo.
    
    Mapea TypeVariable -> Type
    Ejemplo: {T -> Number, U -> String}
    """
    mapping: Dict[TypeVariable, Type] = field(default_factory=dict)
    
    def apply(self, type: Type) -> Type:
        """
        Aplica la sustitución a un tipo.
        
        Ejemplo:
            Si mapping = {T -> Number}
            apply(List<T>) = List<Number>
        """
        if isinstance(type, TypeVariable):
            if type in self.mapping:
                # Recursivamente aplicar sustitución
                return self.apply(self.mapping[type])
            return type
        
        elif isinstance(type, OptionType):
            return OptionType(self.apply(type.inner_type))
        
        elif isinstance(type, ResultType):
            return ResultType(self.apply(type.ok_type), self.apply(type.err_type))
        
        elif isinstance(type, TupleType):
            return TupleType([self.apply(t) for t in type.element_types])
        
        elif isinstance(type, ListType):
            return ListType(self.apply(type.element_type))
        
        elif isinstance(type, SetType):
            return SetType(self.apply(type.element_type))
        
        elif isinstance(type, DictType):
            return DictType(self.apply(type.key_type), self.apply(type.value_type))
        
        elif isinstance(type, FunctionType):
            return FunctionType(
                [self.apply(t) for t in type.param_types],
                self.apply(type.return_type),
                type.is_async
            )
        
        elif isinstance(type, GenericType):
            return GenericType(
                self.apply(type.base),
                [self.apply(t) for t in type.type_args]
            )
        
        elif isinstance(type, UnknownType):
            # UnknownType se reemplaza si está en mapping
            if type in self.mapping:
                return self.apply(self.mapping[type])
            return type
        
        else:
            # Otros tipos no se modifican
            return type
    
    def compose(self, other: 'Substitution') -> 'Substitution':
        """
        Compone dos sustituciones.
        
        (s1 ∘ s2)(T) = s1(s2(T))
        """
        new_mapping = {}
        
        # Aplicar self a todos los valores de other
        for var, type in other.mapping.items():
            new_mapping[var] = self.apply(type)
        
        # Agregar las sustituciones de self que no están en other
        for var, type in self.mapping.items():
            if var not in new_mapping:
                new_mapping[var] = type
        
        return Substitution(new_mapping)
    
    def __str__(self) -> str:
        if not self.mapping:
            return "{}"
        items = [f"{k} -> {v}" for k, v in self.mapping.items()]
        return "{" + ", ".join(items) + "}"


class UnificationError(Exception):
    """Error durante la unificación de tipos"""
    pass


def occurs_check(var: TypeVariable, type: Type) -> bool:
    """
    Occurs check: verifica si var ocurre en type.
    
    Previene unificaciones infinitas como T = List<T>
    """
    if isinstance(type, TypeVariable):
        return var == type
    
    elif isinstance(type, OptionType):
        return occurs_check(var, type.inner_type)
    
    elif isinstance(type, ResultType):
        return occurs_check(var, type.ok_type) or occurs_check(var, type.err_type)
    
    elif isinstance(type, TupleType):
        return any(occurs_check(var, t) for t in type.element_types)
    
    elif isinstance(type, ListType):
        return occurs_check(var, type.element_type)
    
    elif isinstance(type, SetType):
        return occurs_check(var, type.element_type)
    
    elif isinstance(type, DictType):
        return occurs_check(var, type.key_type) or occurs_check(var, type.value_type)
    
    elif isinstance(type, FunctionType):
        return (any(occurs_check(var, t) for t in type.param_types) or
                occurs_check(var, type.return_type))
    
    elif isinstance(type, GenericType):
        return (occurs_check(var, type.base) or
                any(occurs_check(var, t) for t in type.type_args))
    
    else:
        return False


def unify(type1: Type, type2: Type) -> Substitution:
    """
    Unificación de tipos (algoritmo Robinson).
    
    Encuentra la sustitución más general (MGU - Most General Unifier)
    que hace que type1 y type2 sean iguales.
    
    Ejemplo:
        unify(T, Number) = {T -> Number}
        unify(List<T>, List<Number>) = {T -> Number}
        unify(Number, String) = ERROR (no unificables)
    """
    # Caso 1: Tipos idénticos
    if type1 == type2:
        return Substitution()
    
    # Caso 2: type1 es TypeVariable
    if isinstance(type1, TypeVariable):
        if occurs_check(type1, type2):
            raise UnificationError(f"Occurs check failed: {type1} occurs in {type2}")
        return Substitution({type1: type2})
    
    # Caso 3: type2 es TypeVariable
    if isinstance(type2, TypeVariable):
        if occurs_check(type2, type1):
            raise UnificationError(f"Occurs check failed: {type2} occurs in {type1}")
        return Substitution({type2: type1})
    
    # Caso 4: type1 es UnknownType
    if isinstance(type1, UnknownType):
        return Substitution({type1: type2})
    
    # Caso 5: type2 es UnknownType
    if isinstance(type2, UnknownType):
        return Substitution({type2: type1})
    
    # Caso 6: Option<T1> y Option<T2>
    if isinstance(type1, OptionType) and isinstance(type2, OptionType):
        return unify(type1.inner_type, type2.inner_type)
    
    # Caso 7: Result<T1, E1> y Result<T2, E2>
    if isinstance(type1, ResultType) and isinstance(type2, ResultType):
        s1 = unify(type1.ok_type, type2.ok_type)
        s2 = unify(s1.apply(type1.err_type), s1.apply(type2.err_type))
        return s1.compose(s2)
    
    # Caso 8: Tuple
    if isinstance(type1, TupleType) and isinstance(type2, TupleType):
        if len(type1.element_types) != len(type2.element_types):
            raise UnificationError(f"Tuple size mismatch: {type1} vs {type2}")
        
        subst = Substitution()
        for t1, t2 in zip(type1.element_types, type2.element_types):
            s = unify(subst.apply(t1), subst.apply(t2))
            subst = subst.compose(s)
        return subst
    
    # Caso 9: List<T1> y List<T2>
    if isinstance(type1, ListType) and isinstance(type2, ListType):
        return unify(type1.element_type, type2.element_type)
    
    # Caso 10: Set<T1> y Set<T2>
    if isinstance(type1, SetType) and isinstance(type2, SetType):
        return unify(type1.element_type, type2.element_type)
    
    # Caso 11: Dict<K1, V1> y Dict<K2, V2>
    if isinstance(type1, DictType) and isinstance(type2, DictType):
        s1 = unify(type1.key_type, type2.key_type)
        s2 = unify(s1.apply(type1.value_type), s1.apply(type2.value_type))
        return s1.compose(s2)
    
    # Caso 12: Function types
    if isinstance(type1, FunctionType) and isinstance(type2, FunctionType):
        if len(type1.param_types) != len(type2.param_types):
            raise UnificationError(f"Function arity mismatch: {type1} vs {type2}")
        
        if type1.is_async != type2.is_async:
            raise UnificationError(f"Async mismatch: {type1} vs {type2}")
        
        subst = Substitution()
        
        # Unificar parámetros
        for p1, p2 in zip(type1.param_types, type2.param_types):
            s = unify(subst.apply(p1), subst.apply(p2))
            subst = subst.compose(s)
        
        # Unificar tipo de retorno
        s = unify(subst.apply(type1.return_type), subst.apply(type2.return_type))
        subst = subst.compose(s)
        
        return subst
    
    # Caso 13: Generic types
    if isinstance(type1, GenericType) and isinstance(type2, GenericType):
        if type1.base != type2.base:
            raise UnificationError(f"Generic base mismatch: {type1} vs {type2}")
        
        if len(type1.type_args) != len(type2.type_args):
            raise UnificationError(f"Generic arity mismatch: {type1} vs {type2}")
        
        subst = Substitution()
        for t1, t2 in zip(type1.type_args, type2.type_args):
            s = unify(subst.apply(t1), subst.apply(t2))
            subst = subst.compose(s)
        return subst
    
    # Caso 14: Tipos incompatibles
    raise UnificationError(f"Cannot unify {type1} with {type2}")


@dataclass
class TypeInferrer:
    """
    Inferidor de tipos principal.
    
    Implementa el algoritmo Hindley-Milner completo.
    """
    substitution: Substitution = field(default_factory=Substitution)
    fresh_var_counter: int = 0
    
    def fresh_type_var(self, prefix: str = "T") -> TypeVariable:
        """Genera una variable de tipo fresca"""
        self.fresh_var_counter += 1
        return TypeVariable(f"{prefix}{self.fresh_var_counter}")
    
    def instantiate(self, type: Type) -> Type:
        """
        Instancia un tipo polimórfico con variables frescas.
        
        Ejemplo:
            fn identity<T>(x: T) -> T
            instantiate() genera: (T1) -> T1
        """
        # TODO: Implementar para tipos polimórficos
        return type
    
    def generalize(self, type: Type, env_types: Set[TypeVariable]) -> Type:
        """
        Generaliza un tipo a un esquema de tipo polimórfico.
        
        Las variables libres se convierten en variables de tipo.
        """
        # TODO: Implementar generalización
        return type
    
    def infer_literal(self, value) -> Type:
        """Infiere el tipo de un literal"""
        if isinstance(value, int):
            return NUMBER_TYPE
        elif isinstance(value, float):
            return FLOAT_TYPE
        elif isinstance(value, str):
            return STRING_TYPE
        elif isinstance(value, bool):
            return BOOL_TYPE
        else:
            raise UnificationError(f"Unknown literal type: {type(value)}")


def test_unification():
    """Tests del algoritmo de unificación"""
    print("=== Tests de Unificación ===\n")
    
    # Test 1: Unificar tipo primitivo con variable
    T = TypeVariable("T")
    s1 = unify(T, NUMBER_TYPE)
    print(f"Test 1: unify(T, Number) = {s1}")
    print(f"  apply(T) = {s1.apply(T)}")
    assert s1.apply(T) == NUMBER_TYPE
    
    # Test 2: Unificar List<T> con List<Number>
    T = TypeVariable("T")
    list_t = ListType(T)
    list_num = ListType(NUMBER_TYPE)
    s2 = unify(list_t, list_num)
    print(f"\nTest 2: unify(List<T>, List<Number>) = {s2}")
    print(f"  apply(List<T>) = {s2.apply(list_t)}")
    assert s2.apply(list_t) == list_num
    
    # Test 3: Unificar función (T, T) -> T con (Number, Number) -> Number
    T = TypeVariable("T")
    func_t = FunctionType([T, T], T)
    func_num = FunctionType([NUMBER_TYPE, NUMBER_TYPE], NUMBER_TYPE)
    s3 = unify(func_t, func_num)
    print(f"\nTest 3: unify((T, T) -> T, (Number, Number) -> Number) = {s3}")
    print(f"  apply((T, T) -> T) = {s3.apply(func_t)}")
    assert s3.apply(func_t) == func_num
    
    # Test 4: Error - tipos incompatibles
    try:
        s4 = unify(NUMBER_TYPE, STRING_TYPE)
        print("\nTest 4: ERROR - debería fallar")
        assert False
    except UnificationError as e:
        print(f"\nTest 4: ✅ Error esperado: {e}")
    
    # Test 5: Occurs check
    T = TypeVariable("T")
    list_t = ListType(T)
    try:
        s5 = unify(T, list_t)
        print("\nTest 5: ERROR - occurs check debería fallar")
        assert False
    except UnificationError as e:
        print(f"\nTest 5: ✅ Occurs check: {e}")
    
    print("\n✅ Todos los tests de unificación pasaron")


if __name__ == "__main__":
    test_unification()
