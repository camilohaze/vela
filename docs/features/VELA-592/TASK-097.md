# TASK-097: Implementar vela build

## 📋 Información General
- **Historia:** US-22
- **Estado:** En curso ⏳
- **Fecha:** 2025-01-07

## 🎯 Objetivo
Implementar el comando `vela build` para compilar código fuente Vela (.vela) a bytecode (.velac) ejecutable por la VM.

## 🔨 Implementación

### Comando vela build
```bash
vela build [options] [input-files...]
```

**Opciones:**
- `-o, --output <FILE>` - Archivo de salida (.velac)
- `-O, --opt-level <LEVEL>` - Nivel de optimización (none, basic, aggressive, maximum)
- `--target <TARGET>` - Target de compilación (por defecto: bytecode)

### Funcionalidad
1. **Análisis sintáctico** - Parsear archivos .vela
2. **Análisis semántico** - Resolver tipos y símbolos
3. **Generación de bytecode** - Compilar a instrucciones VM
4. **Optimizaciones** - Aplicar optimizaciones según nivel
5. **Output** - Generar archivo .velac

## ✅ Criterios de Aceptación
- [ ] Comando `vela build` implementado
- [ ] Compilación básica funcionando
- [ ] Soporte para múltiples archivos de entrada
- [ ] Niveles de optimización
- [ ] Generación correcta de bytecode .velac
- [ ] Tests unitarios para el comando
- [ ] Documentación generada

## 🔗 Referencias
- **Jira:** [TASK-097](https://velalang.atlassian.net/browse/TASK-097)
- **Historia:** [US-22](https://velalang.atlassian.net/browse/US-22)