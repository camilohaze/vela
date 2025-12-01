"""
Vela Lexer - State Machine Implementation

Implementación de: VELA-567 (Sprint 5)
Subtask: TASK-004, TASK-006
Fecha: 2025-11-30

Lexer basado en state machine para tokenizar código Vela.
- Reconoce ~100 keywords (paradigma funcional puro)
- Tokeniza 40+ operadores
- Tracking preciso de posiciones (line, column, offset)
- Performance O(n) en número de caracteres
"""

from typing import List, Optional
from .token import Token, TokenKind, Position, KEYWORDS


class Lexer:
    """
    Lexer de Vela con state machine manual.
    
    El lexer procesa el código fuente carácter por carácter,
    generando tokens que serán consumidos por el parser.
    """
    
    def __init__(self, source: str):
        """
        Inicializa el lexer con el código fuente.
        
        Args:
            source: Código fuente completo como string
        """
        self.source = source
        self.tokens: List[Token] = []
        
        # Posición actual en el source
        self.current = 0
        self.start = 0
        
        # Posición para tracking (line, column, offset)
        self.position = Position(line=1, column=1, offset=0)
        self.token_start_position = Position(line=1, column=1, offset=0)
    
    def is_at_end(self) -> bool:
        """Verifica si llegamos al final del source."""
        return self.current >= len(self.source)
    
    def peek(self) -> str:
        """
        Retorna el carácter actual sin consumirlo.
        
        Returns:
            Carácter actual, o '\0' si estamos al final
        """
        if self.is_at_end():
            return '\0'
        return self.source[self.current]
    
    def peek_next(self) -> str:
        """
        Retorna el siguiente carácter sin consumirlo.
        
        Returns:
            Siguiente carácter, o '\0' si no hay más
        """
        if self.current + 1 >= len(self.source):
            return '\0'
        return self.source[self.current + 1]
    
    def advance(self) -> str:
        """
        Consume y retorna el carácter actual.
        Actualiza position tracking.
        
        Returns:
            Carácter consumido
        """
        char = self.source[self.current]
        self.current += 1
        self.position.advance(char)
        return char
    
    def matches(self, expected: str) -> bool:
        """
        Verifica si el carácter actual es el esperado y lo consume.
        
        Args:
            expected: Carácter esperado
            
        Returns:
            True si coincide y fue consumido, False si no
        """
        if self.is_at_end():
            return False
        if self.source[self.current] != expected:
            return False
        
        self.advance()
        return True
    
    def current_lexeme(self) -> str:
        """Retorna el texto del token actual (desde start hasta current)."""
        return self.source[self.start:self.current]
    
    def make_token(self, kind: TokenKind, value=None) -> Token:
        """
        Crea un token con el lexeme actual.
        
        Args:
            kind: Tipo de token
            value: Valor asociado (para literales)
            
        Returns:
            Token creado
        """
        lexeme = self.current_lexeme()
        token = Token(kind, lexeme, self.token_start_position, value)
        return token
    
    def error_token(self, message: str) -> Token:
        """
        Crea un token de error.
        
        Args:
            message: Mensaje de error
            
        Returns:
            Token de tipo ERROR
        """
        return Token(TokenKind.ERROR, message, self.token_start_position)
    
    def skip_whitespace(self) -> None:
        """Salta espacios en blanco, tabs y saltos de línea."""
        while not self.is_at_end():
            c = self.peek()
            if c in ' \t\r\n':
                self.advance()
            else:
                break
    
    def comment_line(self) -> Token:
        """
        Consume un comentario de línea // hasta el final.
        
        Returns:
            Siguiente token después del comentario
        """
        # Consumir hasta newline
        while not self.is_at_end() and self.peek() != '\n':
            self.advance()
        
        # Saltar el newline y continuar
        if not self.is_at_end():
            self.advance()
        
        return self.next_token()
    
    def comment_block(self) -> Token:
        """
        Consume un comentario de bloque /* ... */.
        
        Returns:
            Siguiente token después del comentario
        """
        # Consumir hasta encontrar */
        while not self.is_at_end():
            if self.peek() == '*' and self.peek_next() == '/':
                self.advance()  # *
                self.advance()  # /
                break
            self.advance()
        
        return self.next_token()
    
    def identifier(self) -> Token:
        """
        Tokeniza un identificador o keyword.
        
        Identificadores: [a-zA-Z_][a-zA-Z0-9_]*
        
        Returns:
            Token IDENTIFIER o keyword específico
        """
        while self.peek().isalnum() or self.peek() == '_':
            self.advance()
        
        text = self.current_lexeme()
        
        # Verificar si es un keyword
        if text in KEYWORDS:
            kind = KEYWORDS[text]
            
            # Para booleanos, agregar el valor
            if kind == TokenKind.TRUE:
                return self.make_token(kind, value=True)
            elif kind == TokenKind.FALSE:
                return self.make_token(kind, value=False)
            
            return self.make_token(kind)
        
        # Es un identificador común
        return self.make_token(TokenKind.IDENTIFIER)
    
    def number(self) -> Token:
        """
        Tokeniza un número (entero o float).
        
        Formatos soportados:
        - Enteros: 123, 42
        - Floats: 3.14, 0.5
        
        Returns:
            Token NUMBER_LITERAL o FLOAT_LITERAL
        """
        # Consumir dígitos
        while self.peek().isdigit():
            self.advance()
        
        # Verificar si es float
        if self.peek() == '.' and self.peek_next().isdigit():
            self.advance()  # Consumir '.'
            
            while self.peek().isdigit():
                self.advance()
            
            # Es un float
            lexeme = self.current_lexeme()
            try:
                value = float(lexeme)
                return self.make_token(TokenKind.FLOAT_LITERAL, value)
            except ValueError:
                return self.error_token(f"Invalid float literal: {lexeme}")
        
        # Es un entero
        lexeme = self.current_lexeme()
        try:
            value = int(lexeme)
            return self.make_token(TokenKind.NUMBER_LITERAL, value)
        except ValueError:
            return self.error_token(f"Invalid number literal: {lexeme}")
    
    def string(self) -> Token:
        """
        Tokeniza un string literal.
        
        Formatos:
        - String simple: "Hello, World!"
        - String con escape sequences: "Hello\nWorld"
        - String interpolation: "Hello, ${name}!" (ver TASK-005)
        
        Returns:
            Token STRING_LITERAL
        """
        # Consumir hasta el cierre de "
        value_chars = []
        
        while not self.is_at_end() and self.peek() != '"':
            char = self.peek()
            
            # TODO: String interpolation ${} (TASK-005)
            # Por ahora, string simple
            
            # Manejar escape sequences
            if char == '\\':
                self.advance()  # Consumir \
                if not self.is_at_end():
                    escape_char = self.advance()
                    # Mapeo de escape sequences
                    escape_map = {
                        'n': '\n',
                        't': '\t',
                        'r': '\r',
                        '\\': '\\',
                        '"': '"',
                        '0': '\0',
                    }
                    value_chars.append(escape_map.get(escape_char, escape_char))
            else:
                value_chars.append(self.advance())
        
        if self.is_at_end():
            return self.error_token("Unterminated string")
        
        # Consumir el cierre "
        self.advance()
        
        value = ''.join(value_chars)
        return self.make_token(TokenKind.STRING_LITERAL, value)
    
    def next_token(self) -> Token:
        """
        Genera el siguiente token del source.
        
        Esta es la función principal del state machine.
        
        Returns:
            Próximo token reconocido
        """
        self.skip_whitespace()
        
        # Guardar posición de inicio del token
        self.start = self.current
        self.token_start_position = Position(
            self.position.line,
            self.position.column,
            self.position.offset
        )
        
        if self.is_at_end():
            return self.make_token(TokenKind.EOF)
        
        c = self.advance()
        
        # Identificadores y keywords
        if c.isalpha() or c == '_':
            # Retroceder para procesar desde el inicio
            self.current -= 1
            self.position.offset -= 1
            self.position.column -= 1
            return self.identifier()
        
        # Números
        if c.isdigit():
            self.current -= 1
            self.position.offset -= 1
            self.position.column -= 1
            return self.number()
        
        # Strings
        if c == '"':
            return self.string()
        
        # Operadores y delimitadores
        match c:
            # Operadores de dos caracteres
            case '=':
                if self.matches('='):
                    return self.make_token(TokenKind.EQUAL_EQUAL)
                elif self.matches('>'):
                    return self.make_token(TokenKind.FAT_ARROW)
                else:
                    return self.make_token(TokenKind.EQUAL)
            
            case '!':
                if self.matches('='):
                    return self.make_token(TokenKind.BANG_EQUAL)
                else:
                    return self.make_token(TokenKind.BANG)
            
            case '<':
                if self.matches('='):
                    return self.make_token(TokenKind.LESS_EQUAL)
                elif self.matches('<'):
                    return self.make_token(TokenKind.LESS_LESS)
                else:
                    return self.make_token(TokenKind.LESS)
            
            case '>':
                if self.matches('='):
                    return self.make_token(TokenKind.GREATER_EQUAL)
                elif self.matches('>'):
                    return self.make_token(TokenKind.GREATER_GREATER)
                else:
                    return self.make_token(TokenKind.GREATER)
            
            case '&':
                if self.matches('&'):
                    return self.make_token(TokenKind.AMPERSAND_AMPERSAND)
                else:
                    return self.make_token(TokenKind.AMPERSAND)
            
            case '|':
                if self.matches('|'):
                    return self.make_token(TokenKind.PIPE_PIPE)
                else:
                    return self.make_token(TokenKind.PIPE)
            
            case '?':
                if self.matches('?'):
                    return self.make_token(TokenKind.QUESTION_QUESTION)
                elif self.matches('.'):
                    return self.make_token(TokenKind.QUESTION_DOT)
                else:
                    return self.make_token(TokenKind.QUESTION)
            
            case '+':
                if self.matches('='):
                    return self.make_token(TokenKind.PLUS_EQUAL)
                else:
                    return self.make_token(TokenKind.PLUS)
            
            case '-':
                if self.matches('='):
                    return self.make_token(TokenKind.MINUS_EQUAL)
                elif self.matches('>'):
                    return self.make_token(TokenKind.ARROW)
                else:
                    return self.make_token(TokenKind.MINUS)
            
            case '*':
                if self.matches('*'):
                    return self.make_token(TokenKind.STAR_STAR)
                elif self.matches('='):
                    return self.make_token(TokenKind.STAR_EQUAL)
                else:
                    return self.make_token(TokenKind.STAR)
            
            case '/':
                if self.matches('/'):
                    return self.comment_line()
                elif self.matches('*'):
                    return self.comment_block()
                elif self.matches('='):
                    return self.make_token(TokenKind.SLASH_EQUAL)
                else:
                    return self.make_token(TokenKind.SLASH)
            
            case '%':
                if self.matches('='):
                    return self.make_token(TokenKind.PERCENT_EQUAL)
                else:
                    return self.make_token(TokenKind.PERCENT)
            
            case ':':
                if self.matches(':'):
                    return self.make_token(TokenKind.DOUBLE_COLON)
                else:
                    return self.make_token(TokenKind.COLON)
            
            # Single-character tokens
            case '(': return self.make_token(TokenKind.LEFT_PAREN)
            case ')': return self.make_token(TokenKind.RIGHT_PAREN)
            case '{': return self.make_token(TokenKind.LEFT_BRACE)
            case '}': return self.make_token(TokenKind.RIGHT_BRACE)
            case '[': return self.make_token(TokenKind.LEFT_BRACKET)
            case ']': return self.make_token(TokenKind.RIGHT_BRACKET)
            case ',': return self.make_token(TokenKind.COMMA)
            case ';': return self.make_token(TokenKind.SEMICOLON)
            case '.': return self.make_token(TokenKind.DOT)
            case '^': return self.make_token(TokenKind.CARET)
            case '~': return self.make_token(TokenKind.TILDE)
            
            case _:
                return self.error_token(f"Unexpected character: '{c}'")
    
    def tokenize(self) -> List[Token]:
        """
        Tokeniza todo el source y retorna lista de tokens.
        
        Returns:
            Lista completa de tokens hasta EOF
        """
        tokens = []
        
        while True:
            token = self.next_token()
            tokens.append(token)
            
            if token.kind == TokenKind.EOF or token.kind == TokenKind.ERROR:
                break
        
        return tokens


if __name__ == "__main__":
    # Demo: tokenizar código simple
    code = """
    service UserService {
        fn getUser(id: Number) -> Option<User> {
            user = repository.findById(id)
            if user.exists() {
                return Some(user)
            }
            return None
        }
    }
    """
    
    lexer = Lexer(code)
    tokens = lexer.tokenize()
    
    print("TOKENS GENERADOS:")
    print("=" * 60)
    for token in tokens:
        print(token)
    
    print(f"\nTotal tokens: {len(tokens)}")
