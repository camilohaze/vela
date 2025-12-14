/*
Linking Pipeline para Vela Compiler

Implementación de: TASK-124 (Implementar linking pipeline)
Fecha: 2025-12-14

Este módulo implementa el pipeline completo de linking que toma LLVM IR
y genera ejecutables nativos linkeados con la runtime library en C.
*/

#[cfg(feature = "llvm_backend")]
use inkwell::targets::{TargetMachine, Target, RelocMode, CodeModel, FileType};
#[cfg(feature = "llvm_backend")]
use inkwell::OptimizationLevel;
#[cfg(feature = "llvm_backend")]
use std::path::{Path, PathBuf};
#[cfg(feature = "llvm_backend")]
use std::process::Command;
#[cfg(feature = "llvm_backend")]
use std::env;

#[cfg(feature = "llvm_backend")]
/// Pipeline completo de linking para generar ejecutables nativos
pub struct LinkingPipeline {
    target_machine: TargetMachine,
    runtime_build_dir: PathBuf,
}

#[cfg(feature = "llvm_backend")]
impl LinkingPipeline {
    /// Crear nuevo pipeline de linking
    pub fn new() -> Result<Self, String> {
        // Inicializar target
        Target::initialize_native(&Default::default())
            .map_err(|e| format!("Failed to initialize native target: {}", e))?;

        let target = Target::get_first()
            .ok_or("No native target available")?;

        // Configurar target machine con optimizaciones
        let target_machine = target.create_target_machine(
            &TargetMachine::get_default_triple(),
            "generic",  // CPU features
            "",         // CPU features string
            OptimizationLevel::Aggressive,
            RelocMode::Default,
            CodeModel::Default,
        ).ok_or("Failed to create target machine")?;

        // Directorio de build para runtime
        let runtime_build_dir = PathBuf::from("target/runtime-build");

        Ok(Self {
            target_machine,
            runtime_build_dir,
        })
    }

    /// Compilar módulo LLVM a código objeto
    pub fn compile_to_object(&self, module: &inkwell::module::Module, output_path: &Path) -> Result<(), String> {
        // Escribir código objeto directamente usando target machine
        self.target_machine.write_to_file(
            module,
            FileType::Object,
            output_path
        ).map_err(|e| format!("Failed to write object file: {}", e))?;

        println!("Generated object file: {}", output_path.display());
        Ok(())
    }

    /// Construir runtime library usando CMake
    pub fn build_runtime(&self) -> Result<PathBuf, String> {
        // Crear directorio de build si no existe
        if !self.runtime_build_dir.exists() {
            std::fs::create_dir_all(&self.runtime_build_dir)
                .map_err(|e| format!("Failed to create runtime build dir: {}", e))?;
        }

        // Ejecutar CMake configure
        let cmake_result = Command::new("cmake")
            .args(&[
                "-S", "../../runtime",  // Path to runtime CMakeLists.txt
                "-B", ".",
                "-DCMAKE_BUILD_TYPE=Release",
            ])
            .current_dir(&self.runtime_build_dir)
            .status()
            .map_err(|e| format!("Failed to run cmake configure: {}", e))?;

        if !cmake_result.success() {
            return Err("CMake configure failed".to_string());
        }

        // Ejecutar CMake build
        let build_result = Command::new("cmake")
            .args(&["--build", ".", "--config", "Release"])
            .current_dir(&self.runtime_build_dir)
            .status()
            .map_err(|e| format!("Failed to run cmake build: {}", e))?;

        if !build_result.success() {
            return Err("CMake build failed".to_string());
        }

        // Retornar path a la librería compilada
        let lib_path = if cfg!(target_os = "windows") {
            self.runtime_build_dir.join("Release").join("vela_runtime.lib")
        } else {
            self.runtime_build_dir.join("libvela_runtime.a")
        };

        if !lib_path.exists() {
            return Err(format!("Runtime library not found at: {}", lib_path.display()));
        }

        println!("Built runtime library: {}", lib_path.display());
        Ok(lib_path)
    }

    /// Linkear ejecutable final
    pub fn link_executable(&self, object_files: &[PathBuf], output_path: &Path) -> Result<(), String> {
        // Build runtime library first
        let runtime_lib = self.build_runtime()?;

        // Configurar comando de linking según plataforma
        let (linker_cmd, linker_args) = self.get_linker_command(object_files, &runtime_lib, output_path)?;

        println!("Linking executable: {}", output_path.display());
        println!("Command: {} {:?}", linker_cmd, linker_args);

        let link_result = Command::new(linker_cmd)
            .args(&linker_args)
            .status()
            .map_err(|e| format!("Failed to run linker: {}", e))?;

        if !link_result.success() {
            return Err("Linking failed".to_string());
        }

        println!("Successfully linked executable: {}", output_path.display());
        Ok(())
    }

    /// Obtener comando de linker apropiado para la plataforma
    fn get_linker_command(&self, object_files: &[PathBuf], runtime_lib: &Path, output_path: &Path)
        -> Result<(String, Vec<String>), String>
    {
        let mut args = Vec::new();

        // Agregar archivos objeto
        for obj_file in object_files {
            args.push(obj_file.to_string_lossy().to_string());
        }

        if cfg!(target_os = "windows") {
            // Windows: usar link.exe
            args.push(format!("/LIBPATH:{}", runtime_lib.parent().unwrap().to_string_lossy()));
            args.push("vela_runtime.lib".to_string());
            args.push(format!("/OUT:{}", output_path.to_string_lossy()));

            Ok(("link".to_string(), args))
        } else {
            // Unix-like: usar clang/gcc
            args.push(format!("-L{}", runtime_lib.parent().unwrap().to_string_lossy()));
            args.push("-lvela_runtime".to_string());
            args.push("-lpthread".to_string()); // Para actores
            args.push("-o".to_string());
            args.push(output_path.to_string_lossy().to_string());

            // Preferir clang si está disponible, sino gcc
            if Command::new("clang").arg("--version").status().is_ok() {
                Ok(("clang".to_string(), args))
            } else if Command::new("gcc").arg("--version").status().is_ok() {
                Ok(("gcc".to_string(), args))
            } else {
                Err("No suitable linker found (clang or gcc)".to_string())
            }
        }
    }

    /// Pipeline completo: IR -> Ejecutable
    pub fn build_executable(&self, ir_module: &crate::ir::IRModule, output_path: &Path) -> Result<(), String> {
        use super::ir_to_llvm::LLVMGenerator;

        // 1. Generar LLVM IR
        let mut llvm_generator = LLVMGenerator::new()?;
        llvm_generator.generate(ir_module)?;

        // 2. Crear archivo temporal para objeto
        let temp_dir = env::temp_dir();
        let object_file = temp_dir.join("vela_program.o");

        // 3. Compilar a objeto
        self.compile_to_object(&llvm_generator.module, &object_file)?;

        // 4. Linkear ejecutable
        self.link_executable(&[object_file.clone()], output_path)?;

        // 5. Limpiar archivo temporal
        if object_file.exists() {
            let _ = std::fs::remove_file(&object_file);
        }

        Ok(())
    }

    /// Generar código ensamblador (para debugging)
    pub fn generate_assembly(&self, module: &inkwell::module::Module, output_path: &Path) -> Result<(), String> {
        self.target_machine.write_to_file(
            module,
            FileType::Assembly,
            output_path
        ).map_err(|e| format!("Failed to write assembly file: {}", e))?;

        println!("Generated assembly file: {}", output_path.display());
        Ok(())
    }

    /// Generar LLVM IR textual (para debugging)
    pub fn generate_llvm_ir(&self, module: &inkwell::module::Module, output_path: &Path) -> Result<(), String> {
        module.print_to_file(output_path)
            .map_err(|e| format!("Failed to write LLVM IR file: {}", e))?;

        println!("Generated LLVM IR file: {}", output_path.display());
        Ok(())
    }
}

#[cfg(feature = "llvm_backend")]
impl Default for LinkingPipeline {
    fn default() -> Self {
        Self::new().expect("Failed to create linking pipeline")
    }
}

#[cfg(not(feature = "llvm_backend"))]
/// Stub implementation when LLVM backend is not available
pub struct LinkingPipeline;

#[cfg(not(feature = "llvm_backend")]
impl LinkingPipeline {
    pub fn new() -> Result<Self, String> {
        Err("LLVM backend not available".to_string())
    }

    pub fn build_executable(&self, _ir_module: &crate::ir::IRModule, _output_path: &Path) -> Result<(), String> {
        Err("LLVM backend not available".to_string())
    }
}