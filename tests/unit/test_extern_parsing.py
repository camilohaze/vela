"""
Tests unitarios para el parsing de extern declarations (FFI)

Jira: VELA-1179
Historia: VELA-1179
"""

import pytest
from vela_compiler.parser import Parser
from vela_compiler.lexer import Lexer
from vela_compiler.ast import Declaration, ExternDeclaration, ExternStructDeclaration


class TestExternDeclarations:
    """Suite de tests para extern declarations."""

    def test_extern_function_declaration_basic(self):
        """Test parsing de función externa básica."""
        source = 'extern "C" fn strlen(s: *const u8) -> usize;'
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        parser = Parser(tokens)

        declarations = parser.parse()

        assert len(declarations) == 1
        assert isinstance(declarations[0], Declaration.Extern)

        extern_decl = declarations[0].value
        assert extern_decl.abi == "C"
        assert extern_decl.library is None
        assert extern_decl.function_name == "strlen"
        assert len(extern_decl.parameters) == 1
        assert extern_decl.parameters[0].name == "s"
        assert extern_decl.return_type is not None

    def test_extern_function_with_library(self):
        """Test parsing de función externa con especificación de librería."""
        source = 'extern "C" from "libc.so" fn printf(format: *const u8, ...) -> i32;'
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        parser = Parser(tokens)

        declarations = parser.parse()

        assert len(declarations) == 1
        extern_decl = declarations[0].value
        assert extern_decl.abi == "C"
        assert extern_decl.library == "libc.so"
        assert extern_decl.function_name == "printf"

    def test_extern_struct_declaration(self):
        """Test parsing de struct externa."""
        source = '''
        extern "C" struct tm {
            tm_sec: i32,
            tm_min: i32,
            tm_hour: i32,
        };
        '''
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        parser = Parser(tokens)

        declarations = parser.parse()

        assert len(declarations) == 1
        assert isinstance(declarations[0], Declaration.ExternStruct)

        extern_struct = declarations[0].value
        assert extern_struct.abi == "C"
        assert extern_struct.struct_name == "tm"
        assert len(extern_struct.fields) == 3

    def test_extern_multiple_declarations(self):
        """Test parsing de múltiples declaraciones extern."""
        source = '''
        extern "C" fn malloc(size: usize) -> *mut u8;
        extern "C" fn free(ptr: *mut u8) -> void;
        extern "C" struct FILE {
            _markers: *mut u8,
        };
        '''
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        parser = Parser(tokens)

        declarations = parser.parse()

        assert len(declarations) == 3
        assert isinstance(declarations[0], Declaration.Extern)
        assert isinstance(declarations[1], Declaration.Extern)
        assert isinstance(declarations[2], Declaration.ExternStruct)

    def test_extern_different_abis(self):
        """Test parsing de diferentes ABIs."""
        sources = [
            'extern "C" fn c_function() -> void;',
            'extern "C++" fn cpp_function() -> void;',
            'extern "Rust" fn rust_function() -> void;',
        ]

        for source in sources:
            lexer = Lexer(source)
            tokens = lexer.tokenize()
            parser = Parser(tokens)

            declarations = parser.parse()
            assert len(declarations) == 1
            extern_decl = declarations[0].value
            assert extern_decl.abi in ["C", "C++", "Rust"]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])