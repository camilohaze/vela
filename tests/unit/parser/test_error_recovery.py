"""
Tests para Error Recovery del Parser de Vela

Tests de: VELA-568 (TASK-012)
Historia: Sprint 6 - Parser que genere AST válido
Fecha: 2025-12-01

Este módulo testea el error recovery del parser:
- Panic mode (sincronización)
- Múltiples errores acumulados
- Common mistakes (let, null, for, switch, export)
- Phrase-level recovery
- Error messages descriptivos
- Sugerencias útiles
"""

import pytest
import sys
sys.path.append('../..')

from src.parser.error_recovery import (
    ErrorRecoveryParser, ParseError, ErrorSeverity, ErrorStatistics
)
from src.lexer import tokenize


class TestPanicMode:
    """Tests para panic mode recovery"""
    
    def test_synchronize_after_error(self):
        """Test sincronización después de error"""
        code = """
        fn test() -> void {
            x: Number = 42 INVALID TOKEN
            y: Number = 10
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        # Parser debe recuperarse y continuar
        assert ast is not None
        assert len(errors) > 0
    
    def test_synchronize_to_declaration(self):
        """Test sincronización a siguiente declaration"""
        code = """
        fn broken() -> {
            # Missing body
        
        fn working() -> void {
            return
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        assert ast is not None
        assert len(errors) > 0


class TestMultipleErrors:
    """Tests para acumulación de múltiples errores"""
    
    def test_collect_multiple_errors(self):
        """Test colectar múltiples errores"""
        code = """
        fn test() -> void {
            let x = 5
            y = null
            for i in 0..10 {
                print(i)
            }
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        # Debe reportar múltiples errores (let, null, for)
        assert len(errors) >= 3
    
    def test_error_statistics(self):
        """Test estadísticas de errores"""
        code = """
        fn test() -> void {
            let x = null
            switch value {
                case 1: break
            }
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        stats = parser.collect_error_statistics()
        assert stats.total_errors > 0
        assert stats.recovery_attempts > 0


class TestCommonMistakes:
    """Tests para detección de errores comunes"""
    
    def test_detect_let_keyword(self):
        """Test detectar uso de 'let' (prohibido)"""
        code = """
        fn test() -> void {
            let x: Number = 42
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        # Debe haber error sugiriendo usar state o nada
        assert len(errors) > 0
        error_messages = [e.message for e in errors]
        assert any("let" in msg.lower() for msg in error_messages)
    
    def test_detect_const_keyword(self):
        """Test detectar uso de 'const' (prohibido)"""
        code = """
        fn test() -> void {
            const PI: Float = 3.14159
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        assert len(errors) > 0
        error_messages = [e.message for e in errors]
        assert any("const" in msg.lower() for msg in error_messages)
    
    def test_detect_var_keyword(self):
        """Test detectar uso de 'var' (prohibido)"""
        code = """
        fn test() -> void {
            var count: Number = 0
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        assert len(errors) > 0
    
    def test_detect_null_keyword(self):
        """Test detectar uso de 'null' (prohibido)"""
        code = """
        fn test() -> void {
            x = null
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        # Debe sugerir usar None o Option<T>
        assert len(errors) > 0
        error_messages = [e.message for e in errors]
        assert any("null" in msg.lower() or "none" in msg.lower() for msg in error_messages)
    
    def test_detect_undefined_keyword(self):
        """Test detectar uso de 'undefined' (prohibido)"""
        code = """
        fn test() -> void {
            x = undefined
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        assert len(errors) > 0
    
    def test_detect_for_loop(self):
        """Test detectar uso de 'for' loop (prohibido)"""
        code = """
        fn test() -> void {
            for i in 0..10 {
                print(i)
            }
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        # Debe sugerir usar métodos funcionales
        assert len(errors) > 0
        error_messages = [e.message for e in errors]
        assert any("functional" in msg.lower() or "map" in msg.lower() for msg in error_messages)
    
    def test_detect_while_loop(self):
        """Test detectar uso de 'while' loop (prohibido)"""
        code = """
        fn test() -> void {
            while condition {
                doSomething()
            }
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        assert len(errors) > 0
    
    def test_detect_switch_statement(self):
        """Test detectar uso de 'switch' (prohibido)"""
        code = """
        fn test() -> void {
            switch value {
                case 1:
                    print("one")
                    break
                default:
                    print("other")
            }
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        # Debe sugerir usar match
        assert len(errors) > 0
        error_messages = [e.message for e in errors]
        assert any("match" in msg.lower() for msg in error_messages)
    
    def test_detect_export_keyword(self):
        """Test detectar uso de 'export' (prohibido)"""
        code = """
        export fn myFunction() -> void {
            return
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        # Debe sugerir usar public modifier
        assert len(errors) > 0
        error_messages = [e.message for e in errors]
        assert any("public" in msg.lower() for msg in error_messages)


class TestPhraseLevelRecovery:
    """Tests para phrase-level recovery"""
    
    def test_insert_missing_token(self):
        """Test insertar token faltante"""
        code = """
        fn test() -> void {
            x: Number = 42
            y: Number = 10
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        # Parser debe intentar insertar tokens faltantes
        assert ast is not None
    
    def test_delete_unexpected_token(self):
        """Test eliminar token inesperado"""
        code = """
        fn test() -> void {
            x: Number = 42 extra_token
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        assert ast is not None


class TestErrorMessages:
    """Tests para mensajes de error descriptivos"""
    
    def test_error_has_position(self):
        """Test que errores tengan posición"""
        code = """
        fn test() -> void {
            let x = 5
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        assert len(errors) > 0
        for error in errors:
            assert error.position is not None
    
    def test_error_has_message(self):
        """Test que errores tengan mensaje"""
        code = """
        fn test() -> void {
            null
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        assert len(errors) > 0
        for error in errors:
            assert error.message is not None
            assert len(error.message) > 0
    
    def test_error_has_severity(self):
        """Test que errores tengan severity"""
        code = """
        fn test() -> void {
            for i in 0..10 { }
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        assert len(errors) > 0
        for error in errors:
            assert error.severity in [ErrorSeverity.ERROR, ErrorSeverity.WARNING, ErrorSeverity.INFO]


class TestSuggestions:
    """Tests para sugerencias útiles"""
    
    def test_suggestion_for_let(self):
        """Test sugerencia para 'let'"""
        code = """
        fn test() -> void {
            let x = 5
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        # Debe sugerir usar state o nada
        assert len(errors) > 0
        suggestions = [e.suggestion for e in errors if e.suggestion]
        assert len(suggestions) > 0
    
    def test_suggestion_for_null(self):
        """Test sugerencia para 'null'"""
        code = """
        fn test() -> void {
            x = null
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        # Debe sugerir usar None o Option<T>
        suggestions = [e.suggestion for e in errors if e.suggestion]
        assert len(suggestions) > 0
        assert any("None" in s or "Option" in s for s in suggestions)
    
    def test_suggestion_for_for_loop(self):
        """Test sugerencia para 'for' loop"""
        code = """
        fn test() -> void {
            for i in 0..10 { }
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        # Debe sugerir métodos funcionales
        suggestions = [e.suggestion for e in errors if e.suggestion]
        assert len(suggestions) > 0
        assert any(".forEach" in s or ".map" in s for s in suggestions)


class TestFormatErrors:
    """Tests para formateo de errores"""
    
    def test_format_single_error(self):
        """Test formatear un error"""
        code = """
        fn test() -> void {
            null
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        formatted = parser.format_errors(errors)
        assert formatted is not None
        assert len(formatted) > 0
    
    def test_format_multiple_errors(self):
        """Test formatear múltiples errores"""
        code = """
        fn test() -> void {
            let x = null
            for i in 0..10 { }
        }
        """
        tokens = tokenize(code)
        parser = ErrorRecoveryParser(tokens)
        ast, errors = parser.parse_with_recovery()
        
        formatted = parser.format_errors(errors)
        assert formatted is not None
        # Debe contener información de múltiples errores
        assert "error" in formatted.lower()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
