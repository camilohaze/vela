/*!
 * Module System Tests
 *
 * Tests exhaustivos para el sistema de carga de módulos de VelaVM.
 * Incluye tests unitarios, de integración y de performance.
 *
 * Jira: VELA-588
 * Task: TASK-081
 */

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use vela_vm::error::Error;
use vela_vm::loader::{BytecodeLoader, LoadedModule};
use vela_vm::module_resolver::ModuleResolver;
use vela_vm::{Bytecode, CodeObject};

/// Tests de integración completa del sistema de módulos
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_module_loading_workflow() {
        // Crear directorio temporal para el proyecto
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Crear estructura de módulos
        let modules_dir = project_root.join("modules");
        fs::create_dir(&modules_dir).unwrap();

        // Crear módulo principal
        let main_bytecode = create_test_bytecode("main", vec!["utils", "math"]);
        let main_path = modules_dir.join("main.velac");
        save_bytecode(&main_bytecode, &main_path);

        // Crear módulo de utilidades
        let utils_bytecode = create_test_bytecode("utils", vec![]);
        let utils_path = modules_dir.join("utils.velac");
        save_bytecode(&utils_bytecode, &utils_path);

        // Crear módulo de matemáticas
        let math_bytecode = create_test_bytecode("math", vec![]);
        let math_path = modules_dir.join("math.velac");
        save_bytecode(&math_bytecode, &math_path);

        // Configurar resolver y loader
        let mut resolver = ModuleResolver::new(project_root.to_path_buf());
        resolver.add_search_path("module", modules_dir);

        let mut loader = BytecodeLoader::new();
        loader.set_resolver(resolver);

        // Cargar módulo principal
        let main_module = loader.load_module("module:main").unwrap();

        assert_eq!(main_module.name, "module:main");
        assert_eq!(main_module.exports.len(), 2); // function_a, function_b
        assert!(main_module.exports.contains_key("function_a"));
        assert!(main_module.exports.contains_key("function_b"));

        // Verificar que las dependencias se cargaron automáticamente
        assert!(loader.is_module_loaded("module:utils"));
        assert!(loader.is_module_loaded("module:math"));

        // Verificar contenido de dependencias
        let utils_module = loader.get_loaded_module("module:utils").unwrap();
        assert_eq!(utils_module.name, "module:utils");

        let math_module = loader.get_loaded_module("module:math").unwrap();
        assert_eq!(math_module.name, "module:math");
    }

    #[test]
    fn test_circular_dependency_detection() {
        let temp_dir = TempDir::new().unwrap();
        let modules_dir = temp_dir.path().join("modules");
        fs::create_dir(&modules_dir).unwrap();

        // Crear módulos con dependencia circular: A -> B -> A
        let bytecode_a = create_test_bytecode_with_deps("a", vec!["b"]);
        let bytecode_b = create_test_bytecode_with_deps("b", vec!["a"]);

        save_bytecode(&bytecode_a, &modules_dir.join("a.velac"));
        save_bytecode(&bytecode_b, &modules_dir.join("b.velac"));

        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        resolver.add_search_path("module", modules_dir);

        let mut loader = BytecodeLoader::new();
        loader.set_resolver(resolver);

        // Intentar cargar módulo A debería detectar el ciclo
        let result = loader.load_module("module:a");
        assert!(result.is_err());

        match result.err().unwrap() {
            Error::ImportError { message, .. } => {
                assert!(message.contains("circular") || message.contains("cycle"));
            }
            _ => panic!("Expected ImportError for circular dependency"),
        }
    }

    #[test]
    fn test_module_not_found_error() {
        let temp_dir = TempDir::new().unwrap();
        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        let mut loader = BytecodeLoader::new();
        loader.set_resolver(resolver);

        let result = loader.load_module("nonexistent:module");
        assert!(result.is_err());

        match result.err().unwrap() {
            Error::ImportError { message, .. } => {
                assert!(message.contains("not found") || message.contains("resolve"));
            }
            _ => panic!("Expected ImportError for module not found"),
        }
    }

    #[test]
    fn test_corrupted_bytecode_handling() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join("corrupted.velac");

        // Crear archivo con datos corruptos
        fs::write(&module_path, b"This is not valid bytecode").unwrap();

        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        resolver.add_search_path("module", temp_dir.path().to_path_buf());

        let mut loader = BytecodeLoader::new();
        loader.set_resolver(resolver);

        let result = loader.load_module("module:corrupted");
        assert!(result.is_err());

        match result.err().unwrap() {
            Error::ImportError { message, .. } => {
                assert!(message.contains("deserialize") || message.contains("corrupt"));
            }
            _ => panic!("Expected ImportError for corrupted bytecode"),
        }
    }

    #[test]
    fn test_multiple_prefix_resolution() {
        let temp_dir = TempDir::new().unwrap();

        // Crear diferentes directorios para diferentes prefijos
        let std_dir = temp_dir.path().join("std");
        let user_dir = temp_dir.path().join("user");
        let ext_dir = temp_dir.path().join("ext");

        fs::create_dir(&std_dir).unwrap();
        fs::create_dir(&user_dir).unwrap();
        fs::create_dir(&ext_dir).unwrap();

        // Crear módulos en diferentes ubicaciones
        let std_bytecode = create_test_bytecode("io", vec![]);
        let user_bytecode = create_test_bytecode("utils", vec![]);
        let ext_bytecode = create_test_bytecode("charts", vec![]);

        save_bytecode(&std_bytecode, &std_dir.join("io.velac"));
        save_bytecode(&user_bytecode, &user_dir.join("utils.velac"));
        save_bytecode(&ext_bytecode, &ext_dir.join("charts.velac"));

        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        resolver.add_search_path("std", std_dir);
        resolver.add_search_path("user", user_dir);
        resolver.add_search_path("ext", ext_dir);

        let mut loader = BytecodeLoader::new();
        loader.set_resolver(resolver);

        // Cargar módulos con diferentes prefijos
        let std_module = loader.load_module("std:io").unwrap();
        assert_eq!(std_module.name, "std:io");

        let user_module = loader.load_module("user:utils").unwrap();
        assert_eq!(user_module.name, "user:utils");

        let ext_module = loader.load_module("ext:charts").unwrap();
        assert_eq!(ext_module.name, "ext:charts");

        // Verificar que todos están en cache
        assert_eq!(loader.get_loaded_modules().len(), 3);
    }

    #[test]
    fn test_module_reloading() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join("test.velac");

        // Crear y guardar bytecode inicial
        let mut bytecode = create_test_bytecode("test", vec![]);
        save_bytecode(&bytecode, &module_path);

        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        resolver.add_search_path("module", temp_dir.path().to_path_buf());

        let mut loader = BytecodeLoader::new();
        loader.set_resolver(resolver);

        // Cargar módulo
        let module1 = loader.load_module("module:test").unwrap();
        assert_eq!(module1.exports.len(), 2);

        // Modificar bytecode y volver a guardar
        bytecode.strings.push("new_function".to_string());
        let mut new_code_obj = CodeObject::new(0, 0);
        new_code_obj.names.push(2); // índice de "new_function"
        bytecode.code_objects.push(new_code_obj);
        save_bytecode(&bytecode, &module_path);

        // Recargar módulo (debería usar cache, no recargar)
        let module2 = loader.load_module("module:test").unwrap();
        assert_eq!(module2.exports.len(), 2); // Sigue siendo el mismo (cache)

        // Limpiar cache y recargar
        loader.clear_cache();
        let module3 = loader.load_module("module:test").unwrap();
        assert_eq!(module3.exports.len(), 3); // Ahora incluye la nueva función
    }

    #[test]
    fn test_export_extraction_priority() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join("test.velac");

        // Crear bytecode con exports tanto en metadata como en code objects
        let mut bytecode = Bytecode::new();
        bytecode.strings.push("meta_export".to_string());
        bytecode.strings.push("code_export".to_string());

        // Exports en metadata
        let metadata_exports = HashMap::from([
            ("meta_function".to_string(), 0),
            ("meta_variable".to_string(), 1),
        ]);
        let export_data = bincode::serialize(&metadata_exports).unwrap();
        bytecode.metadata.insert("exports".to_string(), export_data);

        // Exports en code object (fallback)
        let mut code_obj = CodeObject::new(0, 0);
        code_obj.names = vec![0, 1]; // índices a strings
        bytecode.code_objects.push(code_obj);

        save_bytecode(&bytecode, &module_path);

        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        resolver.add_search_path("module", temp_dir.path().to_path_buf());

        let mut loader = BytecodeLoader::new();
        loader.set_resolver(resolver);

        let module = loader.load_module("module:test").unwrap();

        // Debería usar exports de metadata (prioridad)
        assert!(module.exports.contains_key("meta_function"));
        assert!(module.exports.contains_key("meta_variable"));
        assert!(!module.exports.contains_key("meta_export")); // no en metadata
        assert!(!module.exports.contains_key("code_export")); // no en metadata
    }

    #[test]
    fn test_performance_large_module_set() {
        let temp_dir = TempDir::new().unwrap();
        let modules_dir = temp_dir.path().join("modules");
        fs::create_dir(&modules_dir).unwrap();

        // Crear 50 módulos pequeños
        for i in 0..50 {
            let bytecode = create_test_bytecode(&format!("module_{}", i), vec![]);
            let path = modules_dir.join(format!("module_{}.velac", i));
            save_bytecode(&bytecode, &path);
        }

        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        resolver.add_search_path("module", modules_dir);

        let mut loader = BytecodeLoader::new();
        loader.set_resolver(resolver);

        // Medir tiempo de carga de todos los módulos
        let start = std::time::Instant::now();

        for i in 0..50 {
            let module_name = format!("module:module_{}", i);
            let _module = loader.load_module(&module_name).unwrap();
        }

        let duration = start.elapsed();

        // Verificar que todos se cargaron
        assert_eq!(loader.get_loaded_modules().len(), 50);

        // Performance: menos de 1 segundo para 50 módulos pequeños
        assert!(duration.as_millis() < 1000);

        println!("Loaded 50 modules in {:?}", duration);
    }
}

/// Tests unitarios específicos del ModuleResolver
#[cfg(test)]
mod resolver_unit_tests {
    use super::*;

    #[test]
    fn test_prefix_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());

        // Configurar rutas de búsqueda
        let std_path = temp_dir.path().join("std");
        let user_path = temp_dir.path().join("user");
        fs::create_dir(&std_path).unwrap();
        fs::create_dir(&user_path).unwrap();

        resolver.add_search_path("std", std_path.clone());
        resolver.add_search_path("user", user_path.clone());

        // Crear archivos de prueba
        fs::write(std_path.join("math.velac"), b"fake bytecode").unwrap();
        fs::write(user_path.join("utils.velac"), b"fake bytecode").unwrap();

        // Resolver módulos con prefijos
        let std_result = resolver.resolve_module("std:math");
        assert!(std_result.is_ok());
        assert_eq!(std_result.unwrap(), std_path.join("math.velac"));

        let user_result = resolver.resolve_module("user:utils");
        assert!(user_result.is_ok());
        assert_eq!(user_result.unwrap(), user_path.join("utils.velac"));
    }

    #[test]
    fn test_resolution_caching() {
        let temp_dir = TempDir::new().unwrap();
        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());

        let test_path = temp_dir.path().join("test.velac");
        fs::write(&test_path, b"fake bytecode").unwrap();
        resolver.add_search_path("module", temp_dir.path().to_path_buf());

        // Primera resolución
        let result1 = resolver.resolve_module("module:test").unwrap();
        assert_eq!(result1, test_path);

        // Modificar archivo (pero cache debería mantener el resultado anterior)
        fs::remove_file(&test_path).unwrap();
        let result2 = resolver.resolve_module("module:test").unwrap();
        assert_eq!(result2, test_path); // Cache hit

        // Limpiar cache y resolver nuevamente
        resolver.clear_cache();
        let result3 = resolver.resolve_module("module:test");
        assert!(result3.is_err()); // Ahora debería fallar
    }

    #[test]
    fn test_invalid_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());

        let result = resolver.resolve_module("invalid:module");
        assert!(result.is_err());
    }

    #[test]
    fn test_relative_path_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());

        // Crear archivo en subdirectorio
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        let file_path = sub_dir.join("module.velac");
        fs::write(&file_path, b"fake bytecode").unwrap();

        // Resolver con path relativo
        let result = resolver.resolve_module("./subdir/module");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), file_path);
    }
}

/// Tests unitarios específicos del BytecodeLoader
#[cfg(test)]
mod loader_unit_tests {
    use super::*;

    #[test]
    fn test_bytecode_validation_comprehensive() {
        let loader = BytecodeLoader::new();

        // Bytecode válido
        let mut valid = Bytecode::new();
        valid.code_objects.push(CodeObject::new(0, 0));
        assert!(loader.validate_bytecode(&valid).is_ok());

        // Tests de invalidación
        let test_cases = vec![
            ("magic number", {
                let mut bc = valid.clone();
                bc.magic = 0x12345678;
                bc
            }),
            ("unsupported version", {
                let mut bc = valid.clone();
                bc.version = (2, 0, 0); // versión futura
                bc
            }),
            ("no code objects", {
                let mut bc = valid.clone();
                bc.code_objects.clear();
                bc
            }),
        ];

        for (description, invalid_bytecode) in test_cases {
            let result = loader.validate_bytecode(&invalid_bytecode);
            assert!(result.is_err(), "Should fail for: {}", description);
        }
    }

    #[test]
    fn test_export_extraction_comprehensive() {
        let loader = BytecodeLoader::new();

        // Test con metadata
        let mut bytecode_with_metadata = Bytecode::new();
        let exports = HashMap::from([
            ("func1".to_string(), 0),
            ("var1".to_string(), 1),
        ]);
        let export_data = bincode::serialize(&exports).unwrap();
        bytecode_with_metadata.metadata.insert("exports".to_string(), export_data);

        let extracted = loader.extract_exports(&bytecode_with_metadata).unwrap();
        assert_eq!(extracted, exports);

        // Test con fallback a code objects
        let mut bytecode_fallback = Bytecode::new();
        bytecode_fallback.strings.push("func_a".to_string());
        bytecode_fallback.strings.push("var_b".to_string());

        let mut code_obj = CodeObject::new(0, 0);
        code_obj.names = vec![0, 1]; // índices a strings
        bytecode_fallback.code_objects.push(code_obj);

        let extracted_fallback = loader.extract_exports(&bytecode_fallback).unwrap();
        assert_eq!(extracted_fallback.get("func_a"), Some(&0));
        assert_eq!(extracted_fallback.get("var_b"), Some(&1));
    }

    #[test]
    fn test_cache_operations_comprehensive() {
        let mut loader = BytecodeLoader::new();

        // Inicialmente vacío
        assert!(!loader.is_module_loaded("test"));
        assert_eq!(loader.get_loaded_modules().len(), 0);

        // Crear módulo dummy para testing
        let dummy_module = LoadedModule {
            name: "test".to_string(),
            path: PathBuf::from("dummy.velac"),
            bytecode: Bytecode::new(),
            exports: HashMap::from([("dummy".to_string(), 0)]),
        };

        // Insertar manualmente (normalmente se hace en load_module)
        loader.insert_module_into_cache("test".to_string(), dummy_module);

        // Verificar operaciones de cache
        assert!(loader.is_module_loaded("test"));
        assert_eq!(loader.get_loaded_modules().len(), 1);
        assert!(loader.get_loaded_module("test").is_some());

        // Limpiar cache
        loader.clear_cache();
        assert!(!loader.is_module_loaded("test"));
        assert_eq!(loader.get_loaded_modules().len(), 0);
    }
}

/// Funciones auxiliares para tests
fn create_test_bytecode(name: &str, dependencies: Vec<&str>) -> Bytecode {
    let mut bytecode = Bytecode::new();
    bytecode.strings.push(name.to_string());
    bytecode.strings.push("function_a".to_string());
    bytecode.strings.push("function_b".to_string());

    // Agregar dependencias como metadata
    if !dependencies.is_empty() {
        let deps_data = bincode::serialize(&dependencies).unwrap();
        bytecode.metadata.insert("dependencies".to_string(), deps_data);
    }

    // Agregar exports
    let exports = HashMap::from([
        ("function_a".to_string(), 0),
        ("function_b".to_string(), 1),
    ]);
    let export_data = bincode::serialize(&exports).unwrap();
    bytecode.metadata.insert("exports".to_string(), export_data);

    // Agregar code object básico
    let mut code_obj = CodeObject::new(0, 0);
    code_obj.names = vec![1, 2]; // índices de function_a, function_b
    bytecode.code_objects.push(code_obj);

    bytecode
}

fn create_test_bytecode_with_deps(name: &str, dependencies: Vec<&str>) -> Bytecode {
    let mut bytecode = create_test_bytecode(name, dependencies.clone());

    // Agregar dependencias como metadata estructurada
    let deps_data = bincode::serialize(&dependencies).unwrap();
    bytecode.metadata.insert("dependencies".to_string(), deps_data);

    bytecode
}

fn save_bytecode(bytecode: &Bytecode, path: &Path) {
    use std::fs::File;
    use std::io::Write;

    let data = bincode::serialize(bytecode).unwrap();
    let mut file = File::create(path).unwrap();
    file.write_all(&data).unwrap();
}