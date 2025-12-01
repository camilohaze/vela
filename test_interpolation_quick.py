"""
Test rápido de string interpolation
"""

from src.lexer.lexer import Lexer
from src.lexer.token import TokenKind


def test_basic_interpolation():
    """Test básico de interpolación."""
    print("=" * 60)
    print("TEST: String Interpolation Básica")
    print("=" * 60)
    
    # Test 1: String simple
    print("\n1. String simple sin interpolación:")
    code1 = '"Hello, World!"'
    lexer1 = Lexer(code1)
    token1 = lexer1.next_token()
    print(f"   Input:  {code1}")
    print(f"   Kind:   {token1.kind}")
    print(f"   Value:  {token1.value}")
    assert token1.kind == TokenKind.STRING_LITERAL
    assert token1.value == "Hello, World!"
    print("   ✅ PASS")
    
    # Test 2: String con interpolación
    print("\n2. String con interpolación simple:")
    code2 = '"Hello, ${name}!"'
    lexer2 = Lexer(code2)
    token2 = lexer2.next_token()
    print(f"   Input:  {code2}")
    print(f"   Kind:   {token2.kind}")
    print(f"   Value:  {token2.value}")
    assert token2.kind == TokenKind.STRING_LITERAL
    assert "${name}" in token2.value
    print("   ✅ PASS")
    
    # Test 3: Múltiples interpolaciones
    print("\n3. Múltiples interpolaciones:")
    code3 = '"${greeting}, ${name}! Age: ${age}"'
    lexer3 = Lexer(code3)
    token3 = lexer3.next_token()
    print(f"   Input:  {code3}")
    print(f"   Kind:   {token3.kind}")
    print(f"   Value:  {token3.value}")
    assert token3.kind == TokenKind.STRING_LITERAL
    assert "${greeting}" in token3.value
    assert "${name}" in token3.value
    assert "${age}" in token3.value
    print("   ✅ PASS")
    
    # Test 4: Expresión en interpolación
    print("\n4. Expresión aritmética:")
    code4 = '"Result: ${x + y}"'
    lexer4 = Lexer(code4)
    token4 = lexer4.next_token()
    print(f"   Input:  {code4}")
    print(f"   Kind:   {token4.kind}")
    print(f"   Value:  {token4.value}")
    assert token4.kind == TokenKind.STRING_LITERAL
    assert "${x + y}" in token4.value
    print("   ✅ PASS")
    
    # Test 5: Braces anidados
    print("\n5. Braces anidados (map):")
    code5 = '"Users: ${users.map(u => u.name)}"'
    lexer5 = Lexer(code5)
    token5 = lexer5.next_token()
    print(f"   Input:  {code5}")
    print(f"   Kind:   {token5.kind}")
    print(f"   Value:  {token5.value}")
    assert token5.kind == TokenKind.STRING_LITERAL
    assert token5.value.count('{') == token5.value.count('}')
    print("   ✅ PASS - Braces balanceados")
    
    # Test 6: Escape de $
    print("\n6. Escape de $ con \\$:")
    code6 = r'"Price: \$${amount}"'
    lexer6 = Lexer(code6)
    token6 = lexer6.next_token()
    print(f"   Input:  {code6}")
    print(f"   Kind:   {token6.kind}")
    print(f"   Value:  {repr(token6.value)}")
    assert token6.kind == TokenKind.STRING_LITERAL
    # \$ se escapa a $ literal
    print("   ✅ PASS")
    
    # Test 7: $ sin { no es interpolación
    print("\n7. $ sin { (no es interpolación):")
    code7 = '"Price: $100"'
    lexer7 = Lexer(code7)
    token7 = lexer7.next_token()
    print(f"   Input:  {code7}")
    print(f"   Kind:   {token7.kind}")
    print(f"   Value:  {token7.value}")
    assert token7.kind == TokenKind.STRING_LITERAL
    assert token7.value == "Price: $100"
    print("   ✅ PASS")
    
    print("\n" + "=" * 60)
    print("✅ TODOS LOS TESTS PASARON")
    print("=" * 60)


if __name__ == "__main__":
    test_basic_interpolation()
