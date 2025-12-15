//! # FFI Bridge para Vela
//!
//! Puente de interoperabilidad entre Vela y código C.
//!
//! Este módulo proporciona:
//! - Carga dinámica de librerías C
//! - Conversión automática de tipos
//! - Gestión segura de memoria
//! - Llamadas a funciones C con type safety

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::Arc;
use anyhow::{anyhow, Result};
use libloading::{Library, Symbol};
use libffi_sys::*;

/// Errores del sistema FFI
#[derive(Debug, thiserror::Error)]
pub enum FFIError {
    #[error("Error cargando librería: {0}")]
    LibraryLoadError(String),

    #[error("Símbolo no encontrado: {0}")]
    SymbolNotFound(String),

    #[error("Error de conversión de tipos: {0}")]
    TypeConversionError(String),

    #[error("Error de memoria: {0}")]
    MemoryError(String),

    #[error("Error de llamada FFI: {0}")]
    CallError(String),
}

/// Resultado de operaciones FFI
pub type FFIResult<T> = Result<T, FFIError>;

/// Tipos C primitivos
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CPrimitiveType {
    Void,
    Bool,
    Char,
    Int,
    Long,
    LongLong,
    UnsignedChar,
    UnsignedInt,
    UnsignedLong,
    UnsignedLongLong,
    Float,
    Double,
    Pointer,
}

/// Conversión de tipos entre Vela y C
pub trait FFIType {
    fn c_type() -> CPrimitiveType;
    fn ffi_type() -> *mut ffi_type;
    fn to_c_value(&self) -> *mut c_void;
    fn from_c_value(ptr: *mut c_void) -> Self;
}

/// Implementación de FFIType para tipos primitivos
macro_rules! impl_ffi_primitive {
    ($rust_type:ty, $c_type:expr, $ffi_type_ptr:expr) => {
        impl FFIType for $rust_type {
            fn c_type() -> CPrimitiveType {
                $c_type
            }

            fn ffi_type() -> *mut ffi_type {
                $ffi_type_ptr
            }

            fn to_c_value(&self) -> *mut c_void {
                Box::into_raw(Box::new(*self)) as *mut c_void
            }

            fn from_c_value(ptr: *mut c_void) -> Self {
                unsafe { *(ptr as *const $rust_type) }
            }
        }
    };
}

impl_ffi_primitive!(bool, CPrimitiveType::Bool, unsafe { &mut ffi_type_uint32 });
impl_ffi_primitive!(i32, CPrimitiveType::Int, unsafe { &mut ffi_type_sint32 });
impl_ffi_primitive!(i64, CPrimitiveType::LongLong, unsafe { &mut ffi_type_sint64 });
impl_ffi_primitive!(u32, CPrimitiveType::UnsignedInt, unsafe { &mut ffi_type_uint32 });
impl_ffi_primitive!(u64, CPrimitiveType::UnsignedLongLong, unsafe { &mut ffi_type_uint64 });
impl_ffi_primitive!(f32, CPrimitiveType::Float, unsafe { &mut ffi_type_float });
impl_ffi_primitive!(f64, CPrimitiveType::Double, unsafe { &mut ffi_type_double });

/// Implementación especial para String
impl FFIType for String {
    fn c_type() -> CPrimitiveType {
        CPrimitiveType::Pointer
    }

    fn ffi_type() -> *mut ffi_type {
        unsafe { &mut ffi_type_pointer }
    }

    fn to_c_value(&self) -> *mut c_void {
        let c_string = CString::new(self.as_str()).unwrap();
        c_string.into_raw() as *mut c_void
    }

    fn from_c_value(ptr: *mut c_void) -> Self {
        unsafe {
            let c_str = CStr::from_ptr(ptr as *const c_char);
            c_str.to_string_lossy().into_owned()
        }
    }
}

/// Librería cargada dinámicamente
pub struct FFILibrary {
    library: Library,
    symbols: HashMap<String, *mut c_void>,
}

impl FFILibrary {
    /// Cargar librería desde path
    pub fn load(path: &str) -> FFIResult<Self> {
        let library = unsafe { Library::new(path) }
            .map_err(|e| FFIError::LibraryLoadError(e.to_string()))?;

        Ok(Self {
            library,
            symbols: HashMap::new(),
        })
    }

    /// Obtener símbolo de la librería
    pub fn get_symbol<T>(&mut self, name: &str) -> FFIResult<Symbol<T>> {
        unsafe {
            self.library.get(name.as_bytes())
                .map_err(|e| FFIError::SymbolNotFound(e.to_string()))
        }
    }

    /// Llamar función con argumentos tipados usando libffi-sys
    pub unsafe fn call_function<T: FFIType, Args: FFIArgs>(
        &mut self,
        name: &str,
        args: Args,
    ) -> FFIResult<T> {
        // Obtener el puntero a la función
        let func_ptr = self.library.get::<*mut c_void>(name.as_bytes())
            .map_err(|e| FFIError::SymbolNotFound(e.to_string()))?;

        // Preparar tipos de argumentos y valores
        let arg_types = args.ffi_types();
        let mut arg_values = args.to_c_args();

        // Preparar arrays para libffi
        let mut arg_types_array: Vec<*mut ffi_type> = arg_types.into_iter().map(|t| t as *mut ffi_type).collect();
        let return_type = T::ffi_type();

        // Crear CIF (Call Interface)
        let mut cif: ffi_cif = unsafe { std::mem::zeroed() };
        let status = unsafe {
            ffi_prep_cif(
                &mut cif,
                ffi_abi_FFI_DEFAULT_ABI,
                arg_types_array.len() as u32,
                return_type,
                arg_types_array.as_mut_ptr(),
            )
        };

        if status != ffi_status_FFI_OK {
            return Err(FFIError::CallError("Failed to prepare CIF".to_string()));
        }

        // Preparar valor de retorno
        let mut result: T = unsafe { std::mem::zeroed() };
        let result_ptr = &mut result as *mut T as *mut c_void;

        // Llamar función usando libffi
        unsafe {
            ffi_call(
                &mut cif,
                Some(std::mem::transmute::<*mut c_void, unsafe extern "C" fn()>(*func_ptr)),
                result_ptr,
                arg_values.as_mut_ptr(),
            );
        }

        Ok(result)
    }
}

/// Trait para argumentos de funciones FFI
pub trait FFIArgs {
    fn to_c_args(&self) -> Vec<*mut c_void>;
    fn ffi_types(&self) -> Vec<*mut ffi_type>;
}

/// Implementación de FFIArgs para tuplas
macro_rules! impl_ffi_args {
    ($( $t:ident ),*) => {
        impl<$( $t: FFIType ),*> FFIArgs for ($( $t, )*) {
            fn to_c_args(&self) -> Vec<*mut c_void> {
                let ($( $t, )*) = self;
                vec![$( $t.to_c_value() ),*]
            }

            fn ffi_types(&self) -> Vec<*mut ffi_type> {
                vec![$( $t::ffi_type() ),*]
            }
        }
    };
}

impl_ffi_args!();
impl_ffi_args!(A);
impl_ffi_args!(A, B);
impl_ffi_args!(A, B, C);
impl_ffi_args!(A, B, C, D);
impl_ffi_args!(A, B, C, D, E);

/// Bridge principal FFI
pub struct FFIBridge {
    libraries: HashMap<String, Arc<FFILibrary>>,
}

impl FFIBridge {
    /// Crear nuevo bridge FFI
    pub fn new() -> Self {
        Self {
            libraries: HashMap::new(),
        }
    }

    /// Cargar librería
    pub fn load_library(&mut self, name: &str, path: &str) -> FFIResult<()> {
        let library = FFILibrary::load(path)?;
        self.libraries.insert(name.to_string(), Arc::new(library));
        Ok(())
    }

    /// Obtener librería cargada
    pub fn get_library(&self, name: &str) -> FFIResult<Arc<FFILibrary>> {
        self.libraries.get(name)
            .cloned()
            .ok_or_else(|| FFIError::LibraryLoadError(format!("Library '{}' not loaded", name)))
    }

    /// Llamar función externa
    pub unsafe fn call_extern<T: FFIType, Args: FFIArgs>(
        &mut self,
        library: &str,
        function: &str,
        args: Args,
    ) -> FFIResult<T> {
        let lib = self.get_library(library)?;
        let mut lib_ref = Arc::try_unwrap(lib)
            .map_err(|_| FFIError::CallError("Library in use".to_string()))?;

        lib_ref.call_function(function, args)
    }
}

impl Default for FFIBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Función helper para crear bridge FFI
pub fn create_ffi_bridge() -> FFIBridge {
    FFIBridge::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = create_ffi_bridge();
        assert!(bridge.libraries.is_empty());
    }

    #[test]
    fn test_type_conversion_bool() {
        let value = true;
        let c_value = value.to_c_value();
        let recovered = bool::from_c_value(c_value);
        assert_eq!(value, recovered);
    }

    #[test]
    fn test_type_conversion_i32() {
        let value = 42i32;
        let c_value = value.to_c_value();
        let recovered = i32::from_c_value(c_value);
        assert_eq!(value, recovered);
    }

    #[test]
    fn test_type_conversion_string() {
        let value = "Hello FFI".to_string();
        let c_value = value.to_c_value();
        let recovered = String::from_c_value(c_value);
        assert_eq!(value, recovered);
    }
}