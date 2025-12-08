# TASK-100: Implementar comando vela fmt

## 📋 Información General
- **Historia:** VELA-561
- **Estado:** Completada ✅
- **Fecha:** 2024-01-15

## 🎯 Objetivo
Implementar el comando `vela fmt` para formatear código Vela con reglas de estilo consistentes, incluyendo modo check para validación en CI/CD.

## 🔨 Implementación

### Funcionalidades Implementadas
1. **Formateo básico de sintaxis** con regex:
   - Espacios alrededor de operadores (`+`, `-`, `*`, `/`, `=`, `==`, etc.)
   - Espacios después de comas en parámetros
   - Espacios alrededor de `->` en tipos de retorno
   - Espacios después de `:` en anotaciones de tipo
   - Eliminación de espacios en blanco al final de línea
   - Reducción de líneas en blanco excesivas

2. **Indentación automática**:
   - Indentación consistente de 4 espacios
   - Manejo correcto de bloques `if`/`else`
   - Indentación de funciones y estructuras de control

3. **Modo check** (`--check`):
   - Verifica si los archivos necesitan formateo
   - Retorna código de error si hay archivos mal formateados
   - Útil para CI/CD pipelines

4. **Procesamiento de archivos**:
   - Soporte para múltiples archivos
   - Filtrado automático de archivos `.vela`
   - Ignora archivos que no son `.vela`
   - Manejo de errores de archivo no encontrado

### Archivos generados
- `cli/src/main.rs` - Implementación de `handle_fmt` y `fix_indentation`
- `cli/Cargo.toml` - Dependencia `regex = "1.10"`
- `tests/unit/test_cli_fmt.rs` - Tests unitarios completos

### Código Principal

```rust
fn handle_fmt(files: Vec<PathBuf>, check: bool) -> Result<()> {
    // Regex rules for formatting
    let rules = vec![
        // Remove trailing whitespace
        (Regex::new(r"[ \t]+$").unwrap(), ""),
        // Ensure space around return type arrow
        (Regex::new(r"([^\s])\->([^\s])").unwrap(), "$1 -> $2"),
        // ... more rules
    ];

    // Process each file
    for file_path in files {
        // Read, format, and write back
        let content = fs::read_to_string(&file_path)?;
        let mut formatted_content = content.clone();
        
        // Apply regex rules
        for (pattern, replacement) in &rules {
            formatted_content = pattern.replace_all(&formatted_content, *replacement).to_string();
        }
        
        // Fix indentation
        formatted_content = fix_indentation(&formatted_content);
        
        // Check mode vs format mode
        if check {
            if formatted_content != content {
                // File needs formatting
                return Err(...);
            }
        } else {
            fs::write(&file_path, &formatted_content)?;
        }
    }
    
    Ok(())
}
```

## ✅ Criterios de Aceptación
- [x] Comando `vela fmt <files...>` formatea archivos in-place
- [x] Comando `vela fmt --check <files...>` valida formato sin modificar
- [x] Reglas de formateo aplicadas correctamente (espacios, indentación)
- [x] Manejo de múltiples archivos
- [x] Filtrado de archivos no-.vela
- [x] Tests unitarios con cobertura >= 80%
- [x] Código compila sin errores
- [x] Integración con CLI existente

## 🧪 Tests Implementados

### Casos de Test
1. **test_format_basic_syntax** - Formateo de sintaxis básica
2. **test_format_check_mode** - Modo check retorna error para archivos mal formateados
3. **test_format_already_formatted** - Archivos ya formateados no cambian
4. **test_format_non_vela_file** - Ignora archivos que no son .vela
5. **test_format_file_not_found** - Manejo de archivos inexistentes

### Ejemplos de Formateo

**Antes:**
```vela
fn add(a: Number,b: Number)->Number{
if a>0{
return a+b
}else{
return 0
}
}
```

**Después:**
```vela
fn add(a: Number, b: Number) -> Number{
    if a > 0{
        return a + b
    }else{
        return 0
    }
}
```

## 🔗 Referencias
- **Jira:** [VELA-592](https://velalang.atlassian.net/browse/VELA-592)
- **Historia:** [VELA-561](https://velalang.atlassian.net/browse/VELA-561)
- **Dependencias:** `regex = "1.10"` para patrones de formateo