"""
Tests para Keywords del Lexer de Vela

Implementación de: VELA-567 (Sprint 5)
Subtask: TASK-007 (Tests Unitarios)
Fecha: 2025-11-30

Tests exhaustivos para todos los ~85 keywords del lenguaje Vela.
Verifica que cada keyword es reconocido correctamente y mapeado al TokenKind apropiado.
"""

import pytest
from src.lexer.lexer import Lexer
from src.lexer.token import TokenKind


class TestControlFlowKeywords:
    """Tests para keywords de control de flujo (funcional)."""
    
    def test_if_keyword(self):
        token = Lexer("if").next_token()
        assert token.kind == TokenKind.IF
        assert token.lexeme == "if"
    
    def test_else_keyword(self):
        token = Lexer("else").next_token()
        assert token.kind == TokenKind.ELSE
        assert token.lexeme == "else"
    
    def test_match_keyword(self):
        token = Lexer("match").next_token()
        assert token.kind == TokenKind.MATCH
        assert token.lexeme == "match"
    
    def test_return_keyword(self):
        token = Lexer("return").next_token()
        assert token.kind == TokenKind.RETURN
        assert token.lexeme == "return"
    
    def test_yield_keyword(self):
        token = Lexer("yield").next_token()
        assert token.kind == TokenKind.YIELD
        assert token.lexeme == "yield"


class TestDeclarationKeywords:
    """Tests para keywords de declaración."""
    
    def test_state_keyword(self):
        token = Lexer("state").next_token()
        assert token.kind == TokenKind.STATE
        assert token.lexeme == "state"
    
    def test_fn_keyword(self):
        token = Lexer("fn").next_token()
        assert token.kind == TokenKind.FN
        assert token.lexeme == "fn"
    
    def test_struct_keyword(self):
        token = Lexer("struct").next_token()
        assert token.kind == TokenKind.STRUCT
        assert token.lexeme == "struct"
    
    def test_enum_keyword(self):
        token = Lexer("enum").next_token()
        assert token.kind == TokenKind.ENUM
        assert token.lexeme == "enum"
    
    def test_trait_keyword(self):
        token = Lexer("trait").next_token()
        assert token.kind == TokenKind.TRAIT
        assert token.lexeme == "trait"
    
    def test_impl_keyword(self):
        token = Lexer("impl").next_token()
        assert token.kind == TokenKind.IMPL
        assert token.lexeme == "impl"
    
    def test_type_keyword(self):
        token = Lexer("type").next_token()
        assert token.kind == TokenKind.TYPE
        assert token.lexeme == "type"
    
    def test_interface_keyword(self):
        token = Lexer("interface").next_token()
        assert token.kind == TokenKind.INTERFACE
        assert token.lexeme == "interface"
    
    def test_class_keyword(self):
        token = Lexer("class").next_token()
        assert token.kind == TokenKind.CLASS
        assert token.lexeme == "class"
    
    def test_abstract_keyword(self):
        token = Lexer("abstract").next_token()
        assert token.kind == TokenKind.ABSTRACT
        assert token.lexeme == "abstract"
    
    def test_extends_keyword(self):
        token = Lexer("extends").next_token()
        assert token.kind == TokenKind.EXTENDS
        assert token.lexeme == "extends"
    
    def test_implements_keyword(self):
        token = Lexer("implements").next_token()
        assert token.kind == TokenKind.IMPLEMENTS
        assert token.lexeme == "implements"
    
    def test_override_keyword(self):
        token = Lexer("override").next_token()
        assert token.kind == TokenKind.OVERRIDE
        assert token.lexeme == "override"
    
    def test_overload_keyword(self):
        token = Lexer("overload").next_token()
        assert token.kind == TokenKind.OVERLOAD
        assert token.lexeme == "overload"
    
    def test_constructor_keyword(self):
        token = Lexer("constructor").next_token()
        assert token.kind == TokenKind.CONSTRUCTOR
        assert token.lexeme == "constructor"
    
    def test_this_keyword(self):
        token = Lexer("this").next_token()
        assert token.kind == TokenKind.THIS
        assert token.lexeme == "this"
    
    def test_super_keyword(self):
        token = Lexer("super").next_token()
        assert token.kind == TokenKind.SUPER
        assert token.lexeme == "super"


class TestVisibilityKeywords:
    """Tests para modificadores de visibilidad."""
    
    def test_public_keyword(self):
        token = Lexer("public").next_token()
        assert token.kind == TokenKind.PUBLIC
        assert token.lexeme == "public"
    
    def test_private_keyword(self):
        token = Lexer("private").next_token()
        assert token.kind == TokenKind.PRIVATE
        assert token.lexeme == "private"
    
    def test_protected_keyword(self):
        token = Lexer("protected").next_token()
        assert token.kind == TokenKind.PROTECTED
        assert token.lexeme == "protected"
    
    def test_async_keyword(self):
        token = Lexer("async").next_token()
        assert token.kind == TokenKind.ASYNC
        assert token.lexeme == "async"
    
    def test_static_keyword(self):
        token = Lexer("static").next_token()
        assert token.kind == TokenKind.STATIC
        assert token.lexeme == "static"
    
    def test_extern_keyword(self):
        token = Lexer("extern").next_token()
        assert token.kind == TokenKind.EXTERN
        assert token.lexeme == "extern"


class TestDomainSpecificKeywords:
    """Tests para keywords domain-specific (~30 keywords)."""
    
    # UI Keywords
    def test_widget_keyword(self):
        token = Lexer("widget").next_token()
        assert token.kind == TokenKind.WIDGET
    
    def test_component_keyword(self):
        token = Lexer("component").next_token()
        assert token.kind == TokenKind.COMPONENT
    
    # Architecture Keywords
    def test_service_keyword(self):
        token = Lexer("service").next_token()
        assert token.kind == TokenKind.SERVICE
    
    def test_repository_keyword(self):
        token = Lexer("repository").next_token()
        assert token.kind == TokenKind.REPOSITORY
    
    def test_controller_keyword(self):
        token = Lexer("controller").next_token()
        assert token.kind == TokenKind.CONTROLLER
    
    def test_usecase_keyword(self):
        token = Lexer("usecase").next_token()
        assert token.kind == TokenKind.USECASE
    
    # Model Keywords
    def test_dto_keyword(self):
        token = Lexer("dto").next_token()
        assert token.kind == TokenKind.DTO
    
    def test_entity_keyword(self):
        token = Lexer("entity").next_token()
        assert token.kind == TokenKind.ENTITY
    
    def test_valueObject_keyword(self):
        token = Lexer("valueObject").next_token()
        assert token.kind == TokenKind.VALUE_OBJECT
    
    def test_model_keyword(self):
        token = Lexer("model").next_token()
        assert token.kind == TokenKind.MODEL
    
    # Design Pattern Keywords
    def test_factory_keyword(self):
        token = Lexer("factory").next_token()
        assert token.kind == TokenKind.FACTORY
    
    def test_builder_keyword(self):
        token = Lexer("builder").next_token()
        assert token.kind == TokenKind.BUILDER
    
    def test_strategy_keyword(self):
        token = Lexer("strategy").next_token()
        assert token.kind == TokenKind.STRATEGY
    
    def test_observer_keyword(self):
        token = Lexer("observer").next_token()
        assert token.kind == TokenKind.OBSERVER
    
    def test_singleton_keyword(self):
        token = Lexer("singleton").next_token()
        assert token.kind == TokenKind.SINGLETON
    
    def test_adapter_keyword(self):
        token = Lexer("adapter").next_token()
        assert token.kind == TokenKind.ADAPTER
    
    def test_decorator_keyword(self):
        token = Lexer("decorator").next_token()
        assert token.kind == TokenKind.DECORATOR
    
    # Web/API Keywords
    def test_guard_keyword(self):
        token = Lexer("guard").next_token()
        assert token.kind == TokenKind.GUARD
    
    def test_middleware_keyword(self):
        token = Lexer("middleware").next_token()
        assert token.kind == TokenKind.MIDDLEWARE
    
    def test_interceptor_keyword(self):
        token = Lexer("interceptor").next_token()
        assert token.kind == TokenKind.INTERCEPTOR
    
    def test_validator_keyword(self):
        token = Lexer("validator").next_token()
        assert token.kind == TokenKind.VALIDATOR
    
    # Utility Keywords
    def test_pipe_keyword(self):
        token = Lexer("pipe").next_token()
        assert token.kind == TokenKind.PIPE_KEYWORD
    
    def test_task_keyword(self):
        token = Lexer("task").next_token()
        assert token.kind == TokenKind.TASK
    
    def test_helper_keyword(self):
        token = Lexer("helper").next_token()
        assert token.kind == TokenKind.HELPER
    
    def test_mapper_keyword(self):
        token = Lexer("mapper").next_token()
        assert token.kind == TokenKind.MAPPER
    
    def test_serializer_keyword(self):
        token = Lexer("serializer").next_token()
        assert token.kind == TokenKind.SERIALIZER
    
    def test_store_keyword(self):
        token = Lexer("store").next_token()
        assert token.kind == TokenKind.STORE
    
    def test_provider_keyword(self):
        token = Lexer("provider").next_token()
        assert token.kind == TokenKind.PROVIDER


class TestReactiveKeywords:
    """Tests para keywords del sistema reactivo."""
    
    def test_Signal_keyword(self):
        token = Lexer("Signal").next_token()
        assert token.kind == TokenKind.SIGNAL
        assert token.lexeme == "Signal"
    
    def test_Computed_keyword(self):
        token = Lexer("Computed").next_token()
        assert token.kind == TokenKind.COMPUTED
        assert token.lexeme == "Computed"
    
    def test_Effect_keyword(self):
        token = Lexer("Effect").next_token()
        assert token.kind == TokenKind.EFFECT
        assert token.lexeme == "Effect"
    
    def test_Watch_keyword(self):
        token = Lexer("Watch").next_token()
        assert token.kind == TokenKind.WATCH
        assert token.lexeme == "Watch"
    
    def test_dispatch_keyword(self):
        token = Lexer("dispatch").next_token()
        assert token.kind == TokenKind.DISPATCH
    
    def test_provide_keyword(self):
        token = Lexer("provide").next_token()
        assert token.kind == TokenKind.PROVIDE
    
    def test_inject_keyword(self):
        token = Lexer("inject").next_token()
        assert token.kind == TokenKind.INJECT


class TestLifecycleKeywords:
    """Tests para lifecycle hooks."""
    
    def test_mount_keyword(self):
        token = Lexer("mount").next_token()
        assert token.kind == TokenKind.MOUNT
    
    def test_update_keyword(self):
        token = Lexer("update").next_token()
        assert token.kind == TokenKind.UPDATE
    
    def test_destroy_keyword(self):
        token = Lexer("destroy").next_token()
        assert token.kind == TokenKind.DESTROY
    
    def test_beforeUpdate_keyword(self):
        token = Lexer("beforeUpdate").next_token()
        assert token.kind == TokenKind.BEFORE_UPDATE
    
    def test_afterUpdate_keyword(self):
        token = Lexer("afterUpdate").next_token()
        assert token.kind == TokenKind.AFTER_UPDATE


class TestTypeKeywords:
    """Tests para keywords de tipos."""
    
    def test_Number_type(self):
        token = Lexer("Number").next_token()
        assert token.kind == TokenKind.NUMBER
        assert token.lexeme == "Number"
    
    def test_Float_type(self):
        token = Lexer("Float").next_token()
        assert token.kind == TokenKind.FLOAT
        assert token.lexeme == "Float"
    
    def test_String_type(self):
        token = Lexer("String").next_token()
        assert token.kind == TokenKind.STRING
        assert token.lexeme == "String"
    
    def test_Bool_type(self):
        token = Lexer("Bool").next_token()
        assert token.kind == TokenKind.BOOL
        assert token.lexeme == "Bool"
    
    def test_Option_type(self):
        token = Lexer("Option").next_token()
        assert token.kind == TokenKind.OPTION
        assert token.lexeme == "Option"
    
    def test_Result_type(self):
        token = Lexer("Result").next_token()
        assert token.kind == TokenKind.RESULT
        assert token.lexeme == "Result"
    
    def test_void_type(self):
        token = Lexer("void").next_token()
        assert token.kind == TokenKind.VOID
    
    def test_never_type(self):
        token = Lexer("never").next_token()
        assert token.kind == TokenKind.NEVER


class TestValueKeywords:
    """Tests para keywords de valores."""
    
    def test_true_value(self):
        token = Lexer("true").next_token()
        assert token.kind == TokenKind.TRUE
        assert token.lexeme == "true"
    
    def test_false_value(self):
        token = Lexer("false").next_token()
        assert token.kind == TokenKind.FALSE
        assert token.lexeme == "false"
    
    def test_None_value(self):
        """None (NO null, undefined, nil)."""
        token = Lexer("None").next_token()
        assert token.kind == TokenKind.NONE
        assert token.lexeme == "None"
    
    def test_Some_constructor(self):
        token = Lexer("Some").next_token()
        assert token.kind == TokenKind.SOME
        assert token.lexeme == "Some"
    
    def test_Ok_constructor(self):
        token = Lexer("Ok").next_token()
        assert token.kind == TokenKind.OK
        assert token.lexeme == "Ok"
    
    def test_Err_constructor(self):
        token = Lexer("Err").next_token()
        assert token.kind == TokenKind.ERR
        assert token.lexeme == "Err"


class TestErrorHandlingKeywords:
    """Tests para keywords de manejo de errores."""
    
    def test_try_keyword(self):
        token = Lexer("try").next_token()
        assert token.kind == TokenKind.TRY
    
    def test_catch_keyword(self):
        token = Lexer("catch").next_token()
        assert token.kind == TokenKind.CATCH
    
    def test_throw_keyword(self):
        token = Lexer("throw").next_token()
        assert token.kind == TokenKind.THROW
    
    def test_finally_keyword(self):
        token = Lexer("finally").next_token()
        assert token.kind == TokenKind.FINALLY


class TestAsyncKeywords:
    """Tests para keywords asíncronos."""
    
    def test_await_keyword(self):
        token = Lexer("await").next_token()
        assert token.kind == TokenKind.AWAIT


class TestModuleKeywords:
    """Tests para keywords de módulos."""
    
    def test_import_keyword(self):
        token = Lexer("import").next_token()
        assert token.kind == TokenKind.IMPORT
    
    def test_from_keyword(self):
        token = Lexer("from").next_token()
        assert token.kind == TokenKind.FROM
    
    def test_as_keyword(self):
        token = Lexer("as").next_token()
        assert token.kind == TokenKind.AS
    
    def test_show_keyword(self):
        token = Lexer("show").next_token()
        assert token.kind == TokenKind.SHOW
    
    def test_hide_keyword(self):
        token = Lexer("hide").next_token()
        assert token.kind == TokenKind.HIDE


class TestKeywordCaseSensitivity:
    """Tests para verificar case sensitivity."""
    
    def test_IF_uppercase_is_identifier(self):
        """IF en mayúsculas NO es keyword."""
        token = Lexer("IF").next_token()
        assert token.kind == TokenKind.IDENTIFIER
        assert token.lexeme == "IF"
    
    def test_True_PascalCase_is_identifier(self):
        """True con T mayúscula NO es keyword."""
        token = Lexer("True").next_token()
        assert token.kind == TokenKind.IDENTIFIER
        assert token.lexeme == "True"
    
    def test_number_lowercase_is_identifier(self):
        """number en minúsculas NO es el tipo Number."""
        token = Lexer("number").next_token()
        assert token.kind == TokenKind.IDENTIFIER
        assert token.lexeme == "number"


class TestKeywordsInContext:
    """Tests de keywords en contextos reales."""
    
    def test_multiple_keywords_in_sequence(self):
        code = "public fn getUser"
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        assert tokens[0].kind == TokenKind.PUBLIC
        assert tokens[1].kind == TokenKind.FN
        assert tokens[2].kind == TokenKind.IDENTIFIER
        assert tokens[2].lexeme == "getUser"
    
    def test_keywords_with_whitespace(self):
        code = "if   else   match"
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        assert tokens[0].kind == TokenKind.IF
        assert tokens[1].kind == TokenKind.ELSE
        assert tokens[2].kind == TokenKind.MATCH
    
    def test_keywords_adjacent_to_delimiters(self):
        code = "fn(state){"
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        assert tokens[0].kind == TokenKind.FN
        assert tokens[1].kind == TokenKind.LEFT_PAREN
        assert tokens[2].kind == TokenKind.STATE
        assert tokens[3].kind == TokenKind.RIGHT_PAREN
        assert tokens[4].kind == TokenKind.LEFT_BRACE


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
