/*
Módulo de Debug Info para Vela Compiler

Implementación de: TASK-138 (Debug info generation)
Fecha: 2025-12-14

Este módulo define las estructuras para información de debug que permite
debugging efectivo del código Vela, incluyendo source maps, tabla de símbolos,
y información de variables.
*/

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Información completa de debug para un programa
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebugInfo {
    /// Source maps por archivo
    pub source_maps: HashMap<String, SourceMap>,
    /// Tabla de símbolos global
    pub symbol_table: SymbolTable,
    /// Información de línea para bytecode
    pub line_info: Vec<LineInfo>,
}

impl DebugInfo {
    pub fn new() -> Self {
        Self {
            source_maps: HashMap::new(),
            symbol_table: SymbolTable::new(),
            line_info: Vec::new(),
        }
    }
}

/// Source map para un archivo individual
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceMap {
    /// Ruta del archivo fuente
    pub file_path: String,
    /// Mapeos de línea
    pub line_mappings: Vec<LineMapping>,
}

impl SourceMap {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            line_mappings: Vec::new(),
        }
    }

    pub fn add_mapping(&mut self, source_line: usize, source_column: usize, bytecode_offset: usize) {
        self.line_mappings.push(LineMapping {
            source_line,
            source_column,
            bytecode_offset,
        });
    }
}

/// Mapeo entre posición fuente y bytecode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineMapping {
    /// Línea en el código fuente (1-based)
    pub source_line: usize,
    /// Columna en el código fuente (1-based)
    pub source_column: usize,
    /// Offset en el bytecode
    pub bytecode_offset: usize,
}

/// Información de línea para bytecode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineInfo {
    /// Offset en el bytecode
    pub bytecode_offset: usize,
    /// Línea fuente correspondiente
    pub source_line: usize,
    /// Columna fuente correspondiente
    pub source_column: usize,
}

/// Tabla de símbolos completa
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolTable {
    /// Símbolos de funciones
    pub functions: HashMap<String, FunctionSymbol>,
    /// Símbolos de variables
    pub variables: HashMap<String, VariableSymbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    pub fn add_function(&mut self, symbol: FunctionSymbol) {
        self.functions.insert(symbol.name.clone(), symbol);
    }

    pub fn add_variable(&mut self, symbol: VariableSymbol) {
        self.variables.insert(symbol.name.clone(), symbol);
    }
}

/// Símbolo de función
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionSymbol {
    /// Nombre de la función
    pub name: String,
    /// Offset de inicio en bytecode
    pub start_offset: usize,
    /// Offset de fin en bytecode
    pub end_offset: usize,
    /// Información de parámetros
    pub parameters: Vec<ParameterInfo>,
    /// Información de variables locales
    pub locals: Vec<LocalInfo>,
}

/// Información de parámetro de función
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterInfo {
    /// Nombre del parámetro
    pub name: String,
    /// Tipo del parámetro
    pub param_type: String,
    /// Posición del parámetro (0-based)
    pub position: usize,
}

/// Información de variable local
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalInfo {
    /// Nombre de la variable local
    pub name: String,
    /// Tipo de la variable
    pub var_type: String,
    /// Alcance de la variable
    pub scope: VariableScope,
    /// Ubicación de la variable
    pub location: VariableLocation,
}

/// Símbolo de variable
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableSymbol {
    /// Nombre de la variable
    pub name: String,
    /// Tipo de la variable
    pub var_type: String,
    /// Alcance de la variable
    pub scope: VariableScope,
    /// Ubicación de la variable
    pub location: VariableLocation,
}

/// Alcance de una variable
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VariableScope {
    /// Variable global
    Global,
    /// Variable local a una función
    Local,
    /// Parámetro de función
    Parameter,
    /// Variable de closure/capturada
    Captured,
}

/// Ubicación de una variable en memoria
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VariableLocation {
    /// Variable en el stack (offset desde base del frame)
    Stack(usize),
    /// Variable en registro de VM
    Register(usize),
    /// Variable global (por nombre)
    Global(String),
}

/// Ubicación en código fuente
#[derive(Debug, Clone, Copy)]
pub struct SourceLocation {
    /// Línea (1-based)
    pub line: usize,
    /// Columna (1-based)
    pub column: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// Generador de información de debug
pub struct DebugInfoGenerator {
    debug_info: DebugInfo,
    current_file: Option<String>,
    current_function: Option<String>,
    bytecode_offset: usize,
}

impl DebugInfoGenerator {
    pub fn new() -> Self {
        Self {
            debug_info: DebugInfo::new(),
            current_file: None,
            current_function: None,
            bytecode_offset: 0,
        }
    }

    /// Establecer archivo fuente actual
    pub fn set_current_file(&mut self, file_path: String) {
        self.current_file = Some(file_path.clone());
        self.debug_info.source_maps.insert(file_path.clone(), SourceMap::new(file_path));
    }

    /// Iniciar nueva función
    pub fn start_function(&mut self, name: String, start_offset: usize) {
        self.current_function = Some(name.clone());
        let function_symbol = FunctionSymbol {
            name: name.clone(),
            start_offset,
            end_offset: 0, // Se actualizará al finalizar
            parameters: Vec::new(),
            locals: Vec::new(),
        };
        self.debug_info.symbol_table.add_function(function_symbol);
    }

    /// Finalizar función actual
    pub fn end_function(&mut self, end_offset: usize) {
        if let Some(ref func_name) = self.current_function {
            if let Some(func_symbol) = self.debug_info.symbol_table.functions.get_mut(func_name) {
                func_symbol.end_offset = end_offset;
            }
        }
        self.current_function = None;
    }

    /// Agregar parámetro a función actual
    pub fn add_parameter(&mut self, name: String, param_type: String, position: usize) {
        if let Some(ref func_name) = self.current_function {
            if let Some(func_symbol) = self.debug_info.symbol_table.functions.get_mut(func_name) {
                func_symbol.parameters.push(ParameterInfo {
                    name: name.clone(),
                    param_type: param_type.clone(),
                    position,
                });
            }
        }

        // Agregar como variable global con scope Parameter
        let var_symbol = VariableSymbol {
            name,
            var_type: param_type,
            scope: VariableScope::Parameter,
            location: VariableLocation::Stack(position), // Parámetros en stack
        };
        self.debug_info.symbol_table.add_variable(var_symbol);
    }

    /// Agregar variable local a función actual
    pub fn add_local(&mut self, name: String, var_type: String, location: VariableLocation) {
        if let Some(ref func_name) = self.current_function {
            if let Some(func_symbol) = self.debug_info.symbol_table.functions.get_mut(func_name) {
                func_symbol.locals.push(LocalInfo {
                    name: name.clone(),
                    var_type: var_type.clone(),
                    scope: VariableScope::Local,
                    location: location.clone(),
                });
            }
        }

        // Agregar a tabla global de variables
        let var_symbol = VariableSymbol {
            name,
            var_type,
            scope: VariableScope::Local,
            location,
        };
        self.debug_info.symbol_table.add_variable(var_symbol);
    }

    /// Registrar mapeo de línea fuente a bytecode
    pub fn record_line_mapping(&mut self, source_line: usize, source_column: usize, bytecode_offset: usize) {
        if let Some(ref file_path) = self.current_file {
            if let Some(source_map) = self.debug_info.source_maps.get_mut(file_path) {
                source_map.add_mapping(source_line, source_column, bytecode_offset);
            }
        }

        // Agregar a line_info global
        self.debug_info.line_info.push(LineInfo {
            bytecode_offset,
            source_line,
            source_column,
        });
    }

    /// Obtener información de debug completa
    pub fn get_debug_info(&self) -> &DebugInfo {
        &self.debug_info
    }

    /// Actualizar offset actual de bytecode
    pub fn set_bytecode_offset(&mut self, offset: usize) {
        self.bytecode_offset = offset;
    }
}