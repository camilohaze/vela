"""
Tests de Integración del Lexer de Vela

Implementación de: VELA-567 (Sprint 5)
Subtask: TASK-007 (Tests Unitarios)
Fecha: 2025-11-30

Tests de código Vela real completo.
"""

import pytest
from src.lexer.lexer import Lexer
from src.lexer.token import TokenKind


class TestFunctionDefinitions:
    """Tests de funciones completas."""
    
    def test_simple_function(self):
        code = """fn add(a: Number, b: Number) -> Number {
    return a + b
}"""
        tokens = Lexer(code).tokenize()
        
        # Verificar tokens clave
        assert tokens[0].kind == TokenKind.FN
        assert tokens[1].kind == TokenKind.IDENTIFIER  # add
        assert tokens[2].kind == TokenKind.LEFT_PAREN
        
        # Verificar return existe
        return_tokens = [t for t in tokens if t.kind == TokenKind.RETURN]
        assert len(return_tokens) == 1
    
    def test_async_function(self):
        code = """async fn fetchData() -> Result<String> {
    data = await httpGet("api/users")
    return Ok(data)
}"""
        tokens = Lexer(code).tokenize()
        
        assert tokens[0].kind == TokenKind.ASYNC
        assert tokens[1].kind == TokenKind.FN
        
        await_tokens = [t for t in tokens if t.kind == TokenKind.AWAIT]
        assert len(await_tokens) == 1


class TestServiceDeclarations:
    """Tests de services (DDD)."""
    
    def test_service_class(self):
        code = """service UserService {
    repository: UserRepository = inject(UserRepository)
    
    fn createUser(dto: CreateUserDTO) -> Result<User> {
        return this.repository.save(dto)
    }
}"""
        tokens = Lexer(code).tokenize()
        
        # Verificar service keyword
        service_tokens = [t for t in tokens if t.kind == TokenKind.SERVICE]
        assert len(service_tokens) == 1
        
        # Verificar inject keyword
        inject_tokens = [t for t in tokens if t.kind == TokenKind.INJECT]
        assert len(inject_tokens) == 1


class TestComponentWithState:
    """Tests de componentes UI con state."""
    
    def test_stateful_widget(self):
        code = """component Counter {
    state count: Number = 0
    
    fn increment() -> void {
        this.count = this.count + 1
    }
    
    fn render() -> Widget {
        return Text("Count: ${this.count}")
    }
}"""
        tokens = Lexer(code).tokenize()
        
        # Verificar component
        component_tokens = [t for t in tokens if t.kind == TokenKind.COMPONENT]
        assert len(component_tokens) == 1
        
        # Verificar state
        state_tokens = [t for t in tokens if t.kind == TokenKind.STATE]
        assert len(state_tokens) == 1
        
        # Verificar string interpolation
        string_tokens = [t for t in tokens if t.kind == TokenKind.STRING_LITERAL]
        assert any("${" in t.value for t in string_tokens)


class TestMatchExpressions:
    """Tests de pattern matching."""
    
    def test_match_with_option(self):
        code = """result = findUser(123)
match result {
    Some(user) => print("Found: ${user.name}")
    None => print("Not found")
}"""
        tokens = Lexer(code).tokenize()
        
        # Verificar match
        match_tokens = [t for t in tokens if t.kind == TokenKind.MATCH]
        assert len(match_tokens) == 1
        
        # Verificar Some y None
        some_tokens = [t for t in tokens if t.kind == TokenKind.SOME]
        none_tokens = [t for t in tokens if t.kind == TokenKind.NONE]
        assert len(some_tokens) == 1
        assert len(none_tokens) == 1
        
        # Verificar arrow =>
        arrow_tokens = [t for t in tokens if t.kind == TokenKind.ARROW_THICK]
        assert len(arrow_tokens) == 2


class TestReactiveCode:
    """Tests de reactividad."""
    
    def test_signals_and_effects(self):
        code = """store AppStore {
    state count: Number = 0
    
    computed doubled: Number {
        return this.count * 2
    }
    
    effect {
        print("Count changed: ${this.count}")
    }
}"""
        tokens = Lexer(code).tokenize()
        
        # Verificar keywords reactivos
        store_tokens = [t for t in tokens if t.kind == TokenKind.STORE]
        computed_tokens = [t for t in tokens if t.kind == TokenKind.COMPUTED]
        effect_tokens = [t for t in tokens if t.kind == TokenKind.EFFECT]
        
        assert len(store_tokens) == 1
        assert len(computed_tokens) == 1
        assert len(effect_tokens) == 1


class TestComplexExpressions:
    """Tests de expresiones complejas."""
    
    def test_arithmetic_with_precedence(self):
        code = "result = x + y * z / 2 - (a + b) ** 2"
        tokens = Lexer(code).tokenize()
        
        # Verificar todos los operadores
        operators = [t.kind for t in tokens if t.kind in (
            TokenKind.PLUS, TokenKind.MINUS, TokenKind.MULTIPLY,
            TokenKind.DIVIDE, TokenKind.POWER
        )]
        assert len(operators) == 6  # +, *, /, -, +, **
    
    def test_logical_expression(self):
        code = "isValid = age >= 18 && hasPermission || isAdmin"
        tokens = Lexer(code).tokenize()
        
        and_tokens = [t for t in tokens if t.kind == TokenKind.AND]
        or_tokens = [t for t in tokens if t.kind == TokenKind.OR]
        gte_tokens = [t for t in tokens if t.kind == TokenKind.GREATER_EQUAL]
        
        assert len(and_tokens) == 1
        assert len(or_tokens) == 1
        assert len(gte_tokens) == 1
    
    def test_null_safety_operators(self):
        code = "name = user?.profile?.name ?? 'Unknown'"
        tokens = Lexer(code).tokenize()
        
        # Verificar optional chaining ?.
        safe_nav_tokens = [t for t in tokens if t.kind == TokenKind.QUESTION_DOT]
        assert len(safe_nav_tokens) == 2
        
        # Verificar null coalescing ??
        null_coal_tokens = [t for t in tokens if t.kind == TokenKind.QUESTION_QUESTION]
        assert len(null_coal_tokens) == 1


class TestArraysAndMaps:
    """Tests de estructuras de datos."""
    
    def test_array_literal(self):
        code = 'items = [1, "two", 3.0, true, None]'
        tokens = Lexer(code).tokenize()
        
        assert tokens[2].kind == TokenKind.LEFT_BRACKET
        
        # Verificar diferentes tipos de literales
        number_tokens = [t for t in tokens if t.kind == TokenKind.NUMBER_LITERAL]
        string_tokens = [t for t in tokens if t.kind == TokenKind.STRING_LITERAL]
        float_tokens = [t for t in tokens if t.kind == TokenKind.FLOAT_LITERAL]
        bool_tokens = [t for t in tokens if t.kind == TokenKind.TRUE]
        none_tokens = [t for t in tokens if t.kind == TokenKind.NONE]
        
        assert len(number_tokens) == 1
        assert len(string_tokens) == 1
        assert len(float_tokens) == 1
        assert len(bool_tokens) == 1
        assert len(none_tokens) == 1
    
    def test_functional_methods(self):
        code = "doubled = items.map(x => x * 2).filter(x => x > 5)"
        tokens = Lexer(code).tokenize()
        
        # Verificar arrow functions
        arrow_tokens = [t for t in tokens if t.kind == TokenKind.ARROW_THICK]
        assert len(arrow_tokens) == 2


class TestErrorHandling:
    """Tests de try-catch."""
    
    def test_try_catch_finally(self):
        code = """try {
    result = riskyOperation()
    return Ok(result)
} catch (e: Error) {
    return Err(e)
} finally {
    cleanup()
}"""
        tokens = Lexer(code).tokenize()
        
        try_tokens = [t for t in tokens if t.kind == TokenKind.TRY]
        catch_tokens = [t for t in tokens if t.kind == TokenKind.CATCH]
        finally_tokens = [t for t in tokens if t.kind == TokenKind.FINALLY]
        
        assert len(try_tokens) == 1
        assert len(catch_tokens) == 1
        assert len(finally_tokens) == 1


class TestImportsAndModules:
    """Tests de imports."""
    
    def test_import_statement(self):
        code = "import 'package:http' show { get, post }"
        tokens = Lexer(code).tokenize()
        
        import_tokens = [t for t in tokens if t.kind == TokenKind.IMPORT]
        show_tokens = [t for t in tokens if t.kind == TokenKind.SHOW]
        
        assert len(import_tokens) == 1
        assert len(show_tokens) == 1
    
    def test_import_with_alias(self):
        code = "import 'lib:utils' as util"
        tokens = Lexer(code).tokenize()
        
        as_tokens = [t for t in tokens if t.kind == TokenKind.AS]
        assert len(as_tokens) == 1


class TestRealWorldCode:
    """Tests de código real Vela."""
    
    def test_complete_service_impl(self):
        """Service completo con múltiples features."""
        code = """service OrderService {
    repository: OrderRepository = inject(OrderRepository)
    paymentService: PaymentService = inject(PaymentService)
    
    async fn createOrder(items: List<OrderItem>) -> Result<Order, Error> {
        // Validate items
        if items.length == 0 {
            return Err(Error("Empty order"))
        }
        
        // Calculate total
        total = items.map(i => i.price * i.quantity)
                     .reduce((acc, val) => acc + val, 0.0)
        
        // Process payment
        paymentResult = await this.paymentService.charge(total)
        
        match paymentResult {
            Ok(payment) => {
                order = Order { items, total, paymentId: payment.id }
                return this.repository.save(order)
            }
            Err(error) => return Err(error)
        }
    }
}"""
        tokens = Lexer(code).tokenize()
        
        # Verificar no hay errores
        error_tokens = [t for t in tokens if t.kind == TokenKind.ERROR]
        assert len(error_tokens) == 0
        
        # Verificar keywords clave están presentes
        service_present = any(t.kind == TokenKind.SERVICE for t in tokens)
        async_present = any(t.kind == TokenKind.ASYNC for t in tokens)
        await_present = any(t.kind == TokenKind.AWAIT for t in tokens)
        match_present = any(t.kind == TokenKind.MATCH for t in tokens)
        
        assert service_present
        assert async_present
        assert await_present
        assert match_present


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
