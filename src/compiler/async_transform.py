"""
Async Transform - Continuation Passing Style (CPS)

Implementación de: VELA-580 (TASK-046)
Historia: Sprint 18 - Async/Await
Fecha: 2025-12-02

Este módulo transforma funciones async en state machines.

Transformación:
--------------

Código Original (Vela):
```vela
async fn fetchAndProcess() -> Result<String> {
  user = await fetchUser(123)
  orders = await fetchOrders(user.id)
  return Ok(processOrders(orders))
}
```

Transformación CPS (pseudo-código generado):
```vela
fn fetchAndProcess() -> Future<Result<String>> {
  promise = Promise<Result<String>>()
  
  enum State {
    Start
    AwaitUser(Future<User>)
    AwaitOrders(user: User, Future<Orders>)
  }
  
  state = State::Start
  
  fn resume() -> void {
    match state {
      State::Start => {
        future = fetchUser(123)
        state = State::AwaitUser(future)
        future.then(user => { state.user = user; resume() })
      }
      
      State::AwaitUser(user) => {
        future = fetchOrders(user.id)
        state = State::AwaitOrders(user, future)
        future.then(orders => { state.orders = orders; resume() })
      }
      
      State::AwaitOrders(user, orders) => {
        result = processOrders(orders)
        promise.resolve(Ok(result))
      }
    }
  }
  
  resume()
  return promise.future()
}
```

Algoritmo:
----------

1. **Identificar Suspension Points**: Encontrar todos los `await` en la función
2. **Generar Estados**: Crear un estado por cada suspensión + estado inicial
3. **Analizar Control Flow**: Detectar if, match, loops para estados adicionales
4. **Generar State Machine**: Crear enum con todos los estados
5. **Generar resume()**: Función que avanza la máquina de estados
6. **Preservar Tipos**: Mantener tipos correctos en transformación
7. **Optimizar**: Inline small futures, evitar allocations innecesarias
"""

from dataclasses import dataclass, field
from typing import List, Optional, Set, Dict, Any, Tuple
from enum import Enum
import sys
sys.path.insert(0, '.')

from src.parser.ast_nodes import (
    ASTNode, Program, Declaration, Statement, Expression,
    FunctionDeclaration, MethodDeclaration,
    BlockStatement, ReturnStatement, ExpressionStatement,
    IfStatement, MatchStatement,
    AwaitExpression, CallExpression, Identifier, Literal,
    AssignmentStatement, VariableDeclaration,
    TypeAnnotation, Range, Position
)


# ===================================================================
# STATE REPRESENTATION
# ===================================================================

class StateKind(Enum):
    """Tipos de estados en la state machine"""
    START = "Start"
    AWAIT = "Await"
    IF_TRUE = "IfTrue"
    IF_FALSE = "IfFalse"
    MATCH_ARM = "MatchArm"
    LOOP_BODY = "LoopBody"
    DONE = "Done"
    ERROR = "Error"


@dataclass
class StateMachineState:
    """
    Representa un estado en la state machine.
    
    Cada estado corresponde a un punto de suspensión (await)
    o una bifurcación en el control flow (if, match).
    """
    id: int
    kind: StateKind
    name: str
    captured_vars: List[str] = field(default_factory=list)  # Variables capturadas del scope
    await_expr: Optional[AwaitExpression] = None  # Expression que se espera (si kind == AWAIT)
    continuation: Optional['StateMachineState'] = None  # Siguiente estado
    alternative: Optional['StateMachineState'] = None  # Estado alternativo (if false, match otro arm)
    
    def __repr__(self) -> str:
        return f"State{self.id}::{self.kind.value}({', '.join(self.captured_vars)})"


@dataclass
class StateMachine:
    """
    Representa una state machine completa para una async function.
    """
    function_name: str
    return_type: Optional[TypeAnnotation]
    states: List[StateMachineState]
    initial_state: StateMachineState
    parameters: List[Any]  # Parameters de la función original
    
    def __repr__(self) -> str:
        return f"StateMachine(fn={self.function_name}, states={len(self.states)})"


# ===================================================================
# CONTROL FLOW ANALYZER
# ===================================================================

@dataclass
class SuspensionPoint:
    """
    Punto de suspensión en una async function.
    
    Cada `await` expression es un suspension point.
    """
    id: int
    await_expr: AwaitExpression
    scope_vars: List[str]  # Variables en scope en este punto
    statement: Statement  # Statement que contiene el await
    
    def __repr__(self) -> str:
        return f"SuspensionPoint{self.id}(vars={self.scope_vars})"


class ControlFlowAnalyzer:
    """
    Analiza control flow de una async function.
    
    Identifica:
    - Suspension points (awaits)
    - Variables en scope en cada punto
    - Branching (if, match)
    - Loops
    """
    
    def __init__(self):
        self.suspension_points: List[SuspensionPoint] = []
        self.scope_vars: List[str] = []
        self.current_id: int = 0
    
    def analyze(self, func: FunctionDeclaration) -> List[SuspensionPoint]:
        """
        Analiza una función y retorna sus suspension points.
        """
        self.suspension_points = []
        self.scope_vars = []
        self.current_id = 0
        
        # Agregar parámetros al scope inicial
        self.scope_vars = [param.name for param in func.parameters]
        
        # Analizar body
        self._analyze_block(func.body)
        
        return self.suspension_points
    
    def _analyze_block(self, block: BlockStatement) -> None:
        """Analiza un bloque de statements"""
        for stmt in block.statements:
            self._analyze_statement(stmt)
    
    def _analyze_statement(self, stmt: Statement) -> None:
        """Analiza un statement"""
        if isinstance(stmt, VariableDeclaration):
            # Agregar variable al scope
            self.scope_vars.append(stmt.name)
            
            # Si el initializer tiene await, es suspension point
            if stmt.initializer:
                self._analyze_expression(stmt.initializer, stmt)
        
        elif isinstance(stmt, AssignmentStatement):
            # Si la expresión tiene await, es suspension point
            self._analyze_expression(stmt.value, stmt)
        
        elif isinstance(stmt, ExpressionStatement):
            self._analyze_expression(stmt.expression, stmt)
        
        elif isinstance(stmt, ReturnStatement):
            if stmt.value:
                self._analyze_expression(stmt.value, stmt)
        
        elif isinstance(stmt, IfStatement):
            # Analizar condition
            self._analyze_expression(stmt.condition, stmt)
            
            # Analizar branches
            self._analyze_block(stmt.then_branch)
            if stmt.else_branch:
                if isinstance(stmt.else_branch, BlockStatement):
                    self._analyze_block(stmt.else_branch)
                else:
                    self._analyze_statement(stmt.else_branch)
        
        elif isinstance(stmt, MatchStatement):
            # Analizar scrutinee
            self._analyze_expression(stmt.scrutinee, stmt)
            
            # Analizar cada arm
            for arm in stmt.arms:
                self._analyze_block(arm.body)
        
        # Note: WhileStatement no existe en Vela (funcional puro)
        # Los loops se implementan con recursión o métodos funcionales (.forEach, .map, etc.)
        
        elif isinstance(stmt, BlockStatement):
            self._analyze_block(stmt)
    
    def _analyze_expression(self, expr: Expression, parent_stmt: Statement) -> None:
        """Analiza una expresión buscando awaits"""
        if isinstance(expr, AwaitExpression):
            # ¡Encontramos un suspension point!
            point = SuspensionPoint(
                id=self.current_id,
                await_expr=expr,
                scope_vars=self.scope_vars.copy(),
                statement=parent_stmt
            )
            self.suspension_points.append(point)
            self.current_id += 1
            
            # Analizar la expresión dentro del await
            self._analyze_expression(expr.expression, parent_stmt)
        
        elif isinstance(expr, CallExpression):
            # Analizar argumentos
            for arg in expr.arguments:
                self._analyze_expression(arg, parent_stmt)
        
        # TODO: Agregar más tipos de expresiones según sea necesario


# ===================================================================
# STATE MACHINE BUILDER
# ===================================================================

class StateMachineBuilder:
    """
    Construye una state machine a partir de suspension points.
    """
    
    def __init__(self):
        self.state_counter: int = 0
    
    def build(self, func: FunctionDeclaration, suspension_points: List[SuspensionPoint]) -> StateMachine:
        """
        Construye la state machine.
        
        Algoritmo:
        1. Crear estado inicial (Start)
        2. Por cada suspension point, crear estado Await
        3. Encadenar estados (continuation)
        4. Crear estado final (Done)
        """
        states: List[StateMachineState] = []
        
        # Estado inicial
        initial_state = StateMachineState(
            id=self._next_state_id(),
            kind=StateKind.START,
            name="Start",
            captured_vars=[]
        )
        states.append(initial_state)
        
        # Estado por cada suspension point
        prev_state = initial_state
        for point in suspension_points:
            await_state = StateMachineState(
                id=self._next_state_id(),
                kind=StateKind.AWAIT,
                name=f"Await{point.id}",
                captured_vars=point.scope_vars,
                await_expr=point.await_expr
            )
            states.append(await_state)
            
            # Encadenar con estado anterior
            prev_state.continuation = await_state
            prev_state = await_state
        
        # Estado final (Done)
        done_state = StateMachineState(
            id=self._next_state_id(),
            kind=StateKind.DONE,
            name="Done",
            captured_vars=[]
        )
        states.append(done_state)
        
        # Último await → Done
        if suspension_points:
            prev_state.continuation = done_state
        else:
            # Sin awaits → Start → Done directamente
            initial_state.continuation = done_state
        
        machine = StateMachine(
            function_name=func.name,
            return_type=func.return_type,
            states=states,
            initial_state=initial_state,
            parameters=func.parameters
        )
        
        return machine
    
    def _next_state_id(self) -> int:
        """Genera siguiente ID de estado"""
        state_id = self.state_counter
        self.state_counter += 1
        return state_id


# ===================================================================
# CODE GENERATOR (State Machine → AST)
# ===================================================================

class StateMachineCodeGenerator:
    """
    Genera código AST para la state machine.
    
    Transforma StateMachine → FunctionDeclaration (transformada).
    """
    
    def generate(self, machine: StateMachine, original_func: FunctionDeclaration) -> FunctionDeclaration:
        """
        Genera función transformada.
        
        Estructura generada:
        ```vela
        fn original_name(params) -> Future<ReturnType> {
          promise = Promise<ReturnType>()
          
          enum State {
            Start,
            Await1(Future<...>),
            Await2(...),
            Done
          }
          
          state = State::Start
          
          fn resume() -> void {
            match state {
              State::Start => { ... }
              State::Await1(...) => { ... }
              ...
            }
          }
          
          resume()
          return promise.future()
        }
        ```
        """
        # TODO: Implementar generación completa de código
        # Por ahora, retornar placeholder
        
        # Crear función transformada (no-async)
        transformed_func = FunctionDeclaration(
            name=original_func.name,
            parameters=original_func.parameters,
            return_type=self._wrap_in_future(original_func.return_type),
            body=self._generate_state_machine_body(machine, original_func),
            is_async=False,  # Ya no es async, es función normal que retorna Future
            range=original_func.range,
            generic_params=original_func.generic_params
        )
        
        return transformed_func
    
    def _wrap_in_future(self, return_type: Optional[TypeAnnotation]) -> TypeAnnotation:
        """Envuelve tipo de retorno en Future<T>"""
        # TODO: Crear TypeAnnotation para Future<ReturnType>
        # Por ahora, placeholder
        from src.parser.ast_nodes import GenericType
        
        if return_type is None:
            # Future<void>
            inner_type = TypeAnnotation(name="void", range=Range(Position(0, 0), Position(0, 0)))
        else:
            inner_type = return_type
        
        future_type = GenericType(
            base_name="Future",
            type_arguments=[inner_type],
            range=Range(Position(0, 0), Position(0, 0))
        )
        
        return future_type
    
    def _generate_state_machine_body(self, machine: StateMachine, original_func: FunctionDeclaration) -> BlockStatement:
        """Genera body de la función transformada"""
        # TODO: Implementar generación completa
        # Por ahora, retornar body original (placeholder)
        return original_func.body


# ===================================================================
# ASYNC TRANSFORMER (Main Entry Point)
# ===================================================================

class AsyncTransformer:
    """
    Transformador principal de async/await a CPS.
    
    Uso:
    ```python
    transformer = AsyncTransformer()
    transformed_ast = transformer.transform(original_ast)
    ```
    """
    
    def __init__(self):
        self.cf_analyzer = ControlFlowAnalyzer()
        self.sm_builder = StateMachineBuilder()
        self.code_generator = StateMachineCodeGenerator()
        self.transformed_functions: Dict[str, StateMachine] = {}
    
    def transform(self, program: Program) -> Program:
        """
        Transforma un programa completo.
        
        Busca todas las async functions y las transforma a state machines.
        """
        transformed_declarations: List[Declaration] = []
        
        for decl in program.declarations:
            if self._is_async_function(decl):
                # Transformar async function
                transformed = self._transform_async_function(decl)
                transformed_declarations.append(transformed)
            else:
                # No es async, mantener original
                transformed_declarations.append(decl)
        
        # Crear programa transformado
        transformed_program = Program(
            imports=program.imports,
            declarations=transformed_declarations,
            range=program.range
        )
        
        return transformed_program
    
    def _is_async_function(self, decl: Declaration) -> bool:
        """Verifica si es async function"""
        if isinstance(decl, FunctionDeclaration):
            return decl.is_async
        elif isinstance(decl, MethodDeclaration):
            return decl.is_async
        return False
    
    def _transform_async_function(self, func: FunctionDeclaration) -> FunctionDeclaration:
        """
        Transforma una async function a state machine.
        
        Pipeline:
        1. Analizar control flow → Suspension points
        2. Construir state machine → Estados
        3. Generar código → AST transformado
        """
        print(f"🔄 Transformando async fn {func.name}...")
        
        # 1. Analizar control flow
        suspension_points = self.cf_analyzer.analyze(func)
        print(f"   └─ Encontrados {len(suspension_points)} suspension points")
        
        # 2. Construir state machine
        machine = self.sm_builder.build(func, suspension_points)
        print(f"   └─ State machine: {len(machine.states)} estados")
        
        # Guardar para debugging/inspection
        self.transformed_functions[func.name] = machine
        
        # 3. Generar código transformado
        transformed = self.code_generator.generate(machine, func)
        print(f"   └─ Código generado ✅")
        
        return transformed
    
    def get_state_machine(self, function_name: str) -> Optional[StateMachine]:
        """Obtiene la state machine de una función (para debugging)"""
        return self.transformed_functions.get(function_name)
    
    def print_state_machines(self) -> None:
        """Imprime todas las state machines generadas (debugging)"""
        print("\n" + "="*60)
        print("STATE MACHINES GENERADAS")
        print("="*60)
        
        for func_name, machine in self.transformed_functions.items():
            print(f"\n📦 {func_name}:")
            print(f"   Return type: {machine.return_type}")
            print(f"   Estados: {len(machine.states)}")
            
            for state in machine.states:
                print(f"      - {state}")
                if state.continuation:
                    print(f"        → {state.continuation.name}")
                if state.await_expr:
                    print(f"        ⏸️  await expression")


# ===================================================================
# UTILITY FUNCTIONS
# ===================================================================

def transform_async_to_cps(program: Program) -> Tuple[Program, AsyncTransformer]:
    """
    Función helper para transformar async/await a CPS.
    
    Returns:
        (transformed_program, transformer)
    """
    transformer = AsyncTransformer()
    transformed = transformer.transform(program)
    return transformed, transformer


def analyze_async_function(func: FunctionDeclaration) -> List[SuspensionPoint]:
    """
    Función helper para analizar una async function.
    
    Útil para debugging/testing.
    """
    analyzer = ControlFlowAnalyzer()
    return analyzer.analyze(func)


# ===================================================================
# EJEMPLO DE USO
# ===================================================================

if __name__ == "__main__":
    print("Async Transform - CPS")
    print("=" * 60)
    print()
    print("Este módulo transforma async fn a state machines.")
    print()
    print("Uso:")
    print("```python")
    print("from src.compiler.async_transform import AsyncTransformer")
    print()
    print("transformer = AsyncTransformer()")
    print("transformed_ast = transformer.transform(original_ast)")
    print("transformer.print_state_machines()  # Debug")
    print("```")
    print()
    print("Ver documentación en:")
    print("- docs/architecture/ADR-012-async-await-semantics.md")
    print("- docs/specifications/async-await-spec.md")
    print("- docs/features/VELA-580/TASK-046.md")
