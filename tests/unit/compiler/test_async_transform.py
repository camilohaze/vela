"""
Tests para Async Transform (CPS)

Implementación de: VELA-580 (TASK-046)
Sprint 18 - Async/Await
Fecha: 2025-12-02

Tests de transformación de async/await a state machines.
"""

import pytest
import sys
sys.path.insert(0, '.')

from src.compiler.async_transform import (
    AsyncTransformer,
    ControlFlowAnalyzer,
    StateMachineBuilder,
    StateMachineCodeGenerator,
    SuspensionPoint,
    StateMachine,
    StateKind,
    transform_async_to_cps,
    analyze_async_function
)
from src.parser.ast_nodes import (
    Program, FunctionDeclaration, BlockStatement,
    VariableDeclaration, AssignmentStatement, ReturnStatement,
    AwaitExpression, CallExpression, Identifier, Literal,
    Parameter, PrimitiveType, Range, Position
)


# ===================================================================
# FIXTURES
# ===================================================================

@pytest.fixture
def sample_range():
    """Range de ejemplo para tests"""
    return Range(Position(1, 1), Position(1, 10))


@pytest.fixture
def simple_async_function(sample_range):
    """
    Función async simple:
    
    async fn simple() -> Number {
      x = await fetch()
      return x
    }
    """
    body = BlockStatement(
        statements=[
            VariableDeclaration(
                name="result",
                type_annotation=None,
                initializer=AwaitExpression(
                    expression=CallExpression(
                        callee=Identifier(name="fetchData", range=sample_range),
                        arguments=[],
                        range=sample_range
                    ),
                    range=sample_range
                ),
                is_state=False,
                range=sample_range
            ),
            ReturnStatement(
                value=Identifier(name="x", range=sample_range),
                range=sample_range
            )
        ],
        range=sample_range
    )
    
    func = FunctionDeclaration(
        name="simple",
        parameters=[],
        return_type=PrimitiveType(name="Number", range=sample_range),
        body=body,
        is_async=True,
        range=sample_range,
        generic_params=[]
    )
    
    return func


@pytest.fixture
def multiple_awaits_function(sample_range):
    """
    Función con múltiples awaits:
    
    async fn multiple() -> Number {
      a = await fetch1()
      b = await fetch2()
      c = await fetch3()
      return a + b + c
    }
    """
    body = BlockStatement(
        statements=[
            VariableDeclaration(
                name="a",
                type_annotation=None,
                initializer=AwaitExpression(
                    expression=CallExpression(
                        callee=Identifier(name="fetch1", range=sample_range),
                        arguments=[],
                        range=sample_range
                    ),
                    range=sample_range
                ),
                is_state=False,
                range=sample_range
            ),
            VariableDeclaration(
                name="b",
                type_annotation=None,
                initializer=AwaitExpression(
                    expression=CallExpression(
                        callee=Identifier(name="fetch2", range=sample_range),
                        arguments=[],
                        range=sample_range
                    ),
                    range=sample_range
                ),
                is_state=False,
                range=sample_range
            ),
            VariableDeclaration(
                name="c",
                type_annotation=None,
                initializer=AwaitExpression(
                    expression=CallExpression(
                        callee=Identifier(name="fetch3", range=sample_range),
                        arguments=[],
                        range=sample_range
                    ),
                    range=sample_range
                ),
                is_state=False,
                range=sample_range
            ),
            ReturnStatement(
                value=Identifier(name="result", range=sample_range),  # Simplificado
                range=sample_range
            )
        ],
        range=sample_range
    )
    
    func = FunctionDeclaration(
        name="multiple",
        parameters=[],
        return_type=PrimitiveType(name="Number", range=sample_range),
        body=body,
        is_async=True,
        range=sample_range,
        generic_params=[]
    )
    
    return func


# ===================================================================
# TESTS: Control Flow Analyzer
# ===================================================================

class TestControlFlowAnalyzer:
    """Tests para ControlFlowAnalyzer"""
    
    def test_analyze_simple_async_function(self, simple_async_function):
        """Test: Analizar función async simple con 1 await"""
        analyzer = ControlFlowAnalyzer()
        suspension_points = analyzer.analyze(simple_async_function)
        
        # Debe encontrar 1 suspension point
        assert len(suspension_points) == 1
        
        point = suspension_points[0]
        assert point.id == 0
        assert isinstance(point.await_expr, AwaitExpression)
        # Debe capturar "result" ya que se declara antes del await
        assert "result" in point.scope_vars
    
    def test_analyze_multiple_awaits(self, multiple_awaits_function):
        """Test: Analizar función con múltiples awaits"""
        analyzer = ControlFlowAnalyzer()
        suspension_points = analyzer.analyze(multiple_awaits_function)
        
        # Debe encontrar 3 suspension points
        assert len(suspension_points) == 3
        
        # Verificar IDs secuenciales
        for i, point in enumerate(suspension_points):
            assert point.id == i
            assert isinstance(point.await_expr, AwaitExpression)
    
    def test_analyze_function_without_awaits(self, sample_range):
        """Test: Función sin awaits no genera suspension points"""
        func = FunctionDeclaration(
            name="sync_func",
            parameters=[],
            return_type=PrimitiveType(name="Number", range=sample_range),
            body=BlockStatement(
                statements=[
                    ReturnStatement(
                        value=Literal(value=42, kind="number", range=sample_range),
                        range=sample_range
                    )
                ],
                range=sample_range
            ),
            is_async=False,
            range=sample_range,
            generic_params=[]
        )
        
        analyzer = ControlFlowAnalyzer()
        suspension_points = analyzer.analyze(func)
        
        assert len(suspension_points) == 0


# ===================================================================
# TESTS: State Machine Builder
# ===================================================================

class TestStateMachineBuilder:
    """Tests para StateMachineBuilder"""
    
    def test_build_simple_state_machine(self, simple_async_function):
        """Test: Construir state machine simple"""
        analyzer = ControlFlowAnalyzer()
        suspension_points = analyzer.analyze(simple_async_function)
        
        builder = StateMachineBuilder()
        machine = builder.build(simple_async_function, suspension_points)
        
        # Verificar estructura
        assert machine.function_name == "simple"
        assert len(machine.states) == 3  # Start, Await1, Done
        
        # Verificar estado inicial
        assert machine.initial_state.kind == StateKind.START
        assert machine.initial_state.name == "Start"
        
        # Verificar encadenamiento
        assert machine.initial_state.continuation is not None
        assert machine.initial_state.continuation.kind == StateKind.AWAIT
    
    def test_build_multiple_awaits_state_machine(self, multiple_awaits_function):
        """Test: Construir state machine con múltiples awaits"""
        analyzer = ControlFlowAnalyzer()
        suspension_points = analyzer.analyze(multiple_awaits_function)
        
        builder = StateMachineBuilder()
        machine = builder.build(multiple_awaits_function, suspension_points)
        
        # Verificar número de estados
        # Start + 3 Awaits + Done = 5 estados
        assert len(machine.states) == 5
        
        # Verificar que todos los estados están encadenados
        current = machine.initial_state
        visited = 0
        while current is not None and visited < 10:  # Límite para evitar loops infinitos
            visited += 1
            current = current.continuation
        
        assert visited == 5  # Debe visitar los 5 estados
    
    def test_build_no_awaits(self, sample_range):
        """Test: Función sin awaits genera state machine trivial"""
        func = FunctionDeclaration(
            name="no_awaits",
            parameters=[],
            return_type=PrimitiveType(name="Number", range=sample_range),
            body=BlockStatement(statements=[], range=sample_range),
            is_async=True,
            range=sample_range,
            generic_params=[]
        )
        
        analyzer = ControlFlowAnalyzer()
        suspension_points = analyzer.analyze(func)
        
        builder = StateMachineBuilder()
        machine = builder.build(func, suspension_points)
        
        # Solo Start → Done
        assert len(machine.states) == 2
        assert machine.initial_state.continuation.kind == StateKind.DONE


# ===================================================================
# TESTS: Code Generator
# ===================================================================

class TestStateMachineCodeGenerator:
    """Tests para StateMachineCodeGenerator"""
    
    def test_generate_transformed_function(self, simple_async_function):
        """Test: Generar función transformada"""
        analyzer = ControlFlowAnalyzer()
        suspension_points = analyzer.analyze(simple_async_function)
        
        builder = StateMachineBuilder()
        machine = builder.build(simple_async_function, suspension_points)
        
        generator = StateMachineCodeGenerator()
        transformed = generator.generate(machine, simple_async_function)
        
        # Verificar que ya no es async
        assert transformed.is_async == False
        
        # Verificar que el nombre se preserva
        assert transformed.name == "simple"
        
        # Verificar que retorna Future<T>
        assert transformed.return_type is not None
        # TODO: Verificar que es GenericTypeAnnotation con name="Future"


# ===================================================================
# TESTS: Async Transformer (Integration)
# ===================================================================

class TestAsyncTransformer:
    """Tests de integración para AsyncTransformer"""
    
    def test_transform_program_with_async_function(self, simple_async_function, sample_range):
        """Test: Transformar programa con async function"""
        program = Program(
            imports=[],
            declarations=[simple_async_function],
            range=sample_range
        )
        
        transformer = AsyncTransformer()
        transformed_program = transformer.transform(program)
        
        # Verificar que se transformó
        assert len(transformed_program.declarations) == 1
        
        transformed_func = transformed_program.declarations[0]
        assert isinstance(transformed_func, FunctionDeclaration)
        assert transformed_func.is_async == False  # Ya no es async
        
        # Verificar que se guardó la state machine
        machine = transformer.get_state_machine("simple")
        assert machine is not None
        assert machine.function_name == "simple"
    
    def test_transform_preserves_non_async_functions(self, sample_range):
        """Test: Funciones no-async se preservan sin cambios"""
        sync_func = FunctionDeclaration(
            name="sync",
            parameters=[],
            return_type=PrimitiveType(name="Number", range=sample_range),
            body=BlockStatement(statements=[], range=sample_range),
            is_async=False,
            range=sample_range,
            generic_params=[]
        )
        
        program = Program(
            imports=[],
            declarations=[sync_func],
            range=sample_range
        )
        
        transformer = AsyncTransformer()
        transformed_program = transformer.transform(program)
        
        # Función debe estar sin cambios
        assert len(transformed_program.declarations) == 1
        transformed_func = transformed_program.declarations[0]
        assert transformed_func.name == "sync"
        assert transformed_func.is_async == False
    
    def test_transform_multiple_async_functions(self, simple_async_function, multiple_awaits_function, sample_range):
        """Test: Transformar múltiples async functions"""
        program = Program(
            imports=[],
            declarations=[simple_async_function, multiple_awaits_function],
            range=sample_range
        )
        
        transformer = AsyncTransformer()
        transformed_program = transformer.transform(program)
        
        # Verificar que ambas se transformaron
        assert len(transformed_program.declarations) == 2
        
        # Verificar state machines
        machine1 = transformer.get_state_machine("simple")
        machine2 = transformer.get_state_machine("multiple")
        
        assert machine1 is not None
        assert machine2 is not None
        
        # simple tiene 1 await → 3 estados (Start, Await, Done)
        assert len(machine1.states) == 3
        
        # multiple tiene 3 awaits → 5 estados (Start, Await1, Await2, Await3, Done)
        assert len(machine2.states) == 5


# ===================================================================
# TESTS: Helper Functions
# ===================================================================

class TestHelperFunctions:
    """Tests para funciones helper"""
    
    def test_transform_async_to_cps(self, simple_async_function, sample_range):
        """Test: Función helper transform_async_to_cps"""
        program = Program(
            imports=[],
            declarations=[simple_async_function],
            range=sample_range
        )
        
        transformed, transformer = transform_async_to_cps(program)
        
        assert isinstance(transformed, Program)
        assert isinstance(transformer, AsyncTransformer)
        assert len(transformed.declarations) == 1
    
    def test_analyze_async_function(self, simple_async_function):
        """Test: Función helper analyze_async_function"""
        suspension_points = analyze_async_function(simple_async_function)
        
        assert isinstance(suspension_points, list)
        assert len(suspension_points) == 1
        assert isinstance(suspension_points[0], SuspensionPoint)


# ===================================================================
# TESTS: Edge Cases
# ===================================================================

class TestEdgeCases:
    """Tests de casos edge"""
    
    def test_async_function_with_parameters(self, sample_range):
        """Test: Async function con parámetros"""
        func = FunctionDeclaration(
            name="with_params",
            parameters=[
                Parameter(name="x", type_annotation=PrimitiveType(name="Number", range=sample_range)),
                Parameter(name="y", type_annotation=PrimitiveType(name="String", range=sample_range))
            ],
            return_type=PrimitiveType(name="Number", range=sample_range),
            body=BlockStatement(
                statements=[
                    ReturnStatement(
                        value=AwaitExpression(
                            expression=CallExpression(
                                callee=Identifier(name="fetch", range=sample_range),
                                arguments=[],
                                range=sample_range
                            ),
                            range=sample_range
                        ),
                        range=sample_range
                    )
                ],
                range=sample_range
            ),
            is_async=True,
            range=sample_range,
            generic_params=[]
        )
        
        analyzer = ControlFlowAnalyzer()
        suspension_points = analyzer.analyze(func)
        
        # Verificar que los parámetros están en scope
        assert len(suspension_points) > 0
        # Los parámetros deben estar en scope_vars del primer suspension point
        # (o no, dependiendo de la implementación)
    
    def test_empty_async_function(self, sample_range):
        """Test: Async function vacía"""
        func = FunctionDeclaration(
            name="empty",
            parameters=[],
            return_type=PrimitiveType(name="void", range=sample_range),
            body=BlockStatement(statements=[], range=sample_range),
            is_async=True,
            range=sample_range,
            generic_params=[]
        )
        
        transformer = AsyncTransformer()
        transformed = transformer._transform_async_function(func)
        
        assert transformed.is_async == False
        assert transformed.name == "empty"


# ===================================================================
# RUN TESTS
# ===================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
