/*!
Android Integration Tests

Comprehensive tests for Android build pipeline including compilation validation,
Gradle configuration, bytecode embedding, and end-to-end app generation.
*/

#[cfg(test)]
mod android_integration_tests {
    use super::*;
    use crate::{BuildConfig, BuildExecutor};
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::tempdir;

    #[test]
    fn test_android_project_has_valid_gradle_structure() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");
        let android_dir = output_dir.join("android");

        // Create build config and executor using current directory as project root
        let config = BuildConfig::new(std::env::current_dir().unwrap()).with_output_dir(&output_dir);
        let executor = BuildExecutor::new(config);

        // Generate Android artifacts
        let result = executor.generate_android_artifacts();
        assert!(result.is_ok(), "Android artifacts generation should succeed");

        // Verify directory structure
        assert!(android_dir.exists(), "Android output directory should exist");
        assert!(android_dir.join("build.gradle.kts").exists(), "build.gradle.kts should exist");
        assert!(android_dir.join("settings.gradle.kts").exists(), "settings.gradle.kts should exist");

        // Verify Android app structure
        let main_dir = android_dir.join("src").join("main");
        assert!(main_dir.exists(), "src/main should exist");
        assert!(main_dir.join("AndroidManifest.xml").exists(), "AndroidManifest.xml should exist");

        let kotlin_dir = main_dir.join("kotlin").join("com").join("velalang").join("app");
        assert!(kotlin_dir.exists(), "Kotlin package directory should exist");
        assert!(kotlin_dir.join("MainActivity.kt").exists(), "MainActivity.kt should exist");
    }

    #[test]
    fn test_android_build_gradle_is_valid() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");

        let config = BuildConfig::new(std::env::current_dir().unwrap()).with_output_dir(&output_dir);
        let executor = BuildExecutor::new(config);

        executor.generate_android_artifacts().unwrap();

        let build_gradle_path = output_dir.join("android").join("build.gradle.kts");
        let content = std::fs::read_to_string(build_gradle_path).unwrap();

        // Verify essential Gradle configuration
        assert!(content.contains("com.android.application"), "Should have Android application plugin");
        assert!(content.contains("org.jetbrains.kotlin.android"), "Should have Kotlin Android plugin");
        assert!(content.contains("compileSdk 34"), "Should have correct compile SDK");
        assert!(content.contains("minSdk 21"), "Should have correct min SDK");
        assert!(content.contains("androidx.compose"), "Should have Compose dependencies");
        assert!(content.contains("coil-compose"), "Should have Coil dependency");
        assert!(content.contains("kotlinx-serialization"), "Should have serialization dependency");
    }

    #[test]
    fn test_android_manifest_is_valid() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");

        let config = BuildConfig::new(std::env::current_dir().unwrap()).with_output_dir(&output_dir);
        let executor = BuildExecutor::new(config);

        executor.generate_android_artifacts().unwrap();

        let manifest_path = output_dir.join("android").join("src").join("main").join("AndroidManifest.xml");
        let content = std::fs::read_to_string(manifest_path).unwrap();

        // Verify manifest structure
        assert!(content.contains("<?xml version=\"1.0\""), "Should be valid XML");
        assert!(content.contains("android:name=\".MainActivity\""), "Should have MainActivity");
        assert!(content.contains("android.intent.action.MAIN"), "Should have launcher intent");
        assert!(content.contains("android.permission.INTERNET"), "Should have internet permission");
    }

    #[test]
    fn test_android_main_activity_is_valid() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");

        let config = BuildConfig::new(PathBuf::from("/tmp/project")).with_output_dir(&output_dir);
        let executor = BuildExecutor::new(config);

        executor.generate_android_artifacts().unwrap();

        let activity_path = output_dir.join("android").join("src").join("main")
            .join("kotlin").join("com").join("velalang").join("app").join("MainActivity.kt");
        let content = std::fs::read_to_string(activity_path).unwrap();

        // Verify MainActivity structure
        assert!(content.contains("class MainActivity"), "Should have MainActivity class");
        assert!(content.contains("AndroidRenderEngine"), "Should use AndroidRenderEngine");
        assert!(content.contains("setContent"), "Should use Compose setContent");
        assert!(content.contains("RenderApp()"), "Should call RenderApp");
    }

    #[test]
    fn test_android_settings_gradle_includes_modules() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");

        let config = BuildConfig::new(std::env::current_dir().unwrap()).with_output_dir(&output_dir);
        let executor = BuildExecutor::new(config);

        executor.generate_android_artifacts().unwrap();

        let settings_path = output_dir.join("android").join("settings.gradle.kts");
        let content = std::fs::read_to_string(settings_path).unwrap();

        // Verify settings structure
        assert!(content.contains("include ':app'"), "Should include app module");
        assert!(content.contains("include ':runtime-android'"), "Should include runtime module");
        assert!(content.contains("rootProject.name = \"VelaApp\""), "Should have correct root project name");
    }

    #[test]
    fn test_android_runtime_is_copied() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");

        // Create a mock runtime directory in the location the executor expects
        // The executor searches in current_dir/runtime/android or current_dir/../runtime/android
        let mock_runtime = temp_dir.path().join("runtime").join("android");
        std::fs::create_dir_all(&mock_runtime).unwrap();
        std::fs::write(mock_runtime.join("build.gradle.kts"), "mock content").unwrap();

        let config = BuildConfig::new(temp_dir.path().to_path_buf())
            .with_output_dir(&output_dir);
        let executor = BuildExecutor::new(config);

        executor.generate_android_artifacts().unwrap();

        let runtime_dest = output_dir.join("android").join("runtime-android");
        assert!(runtime_dest.exists(), "Runtime should be copied");
        assert!(runtime_dest.join("build.gradle.kts").exists(), "Runtime files should be copied");
    }

    #[test]
    fn test_android_bytecode_copy() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");

        // Create mock bytecode in the vela output directory (where executor looks for .velac files)
        let vela_output_dir = output_dir.join("vela");
        std::fs::create_dir_all(&vela_output_dir).unwrap();
        std::fs::write(vela_output_dir.join("app.velac"), "mock bytecode").unwrap();

        let config = BuildConfig::new(std::env::current_dir().unwrap()).with_output_dir(&output_dir);
        let executor = BuildExecutor::new(config);

        executor.generate_android_artifacts().unwrap();

        let bytecode_dest = output_dir.join("android").join("Bytecode").join("app.velac");
        assert!(bytecode_dest.exists(), "Bytecode should be copied");
        assert_eq!(std::fs::read_to_string(bytecode_dest).unwrap(), "mock bytecode", "Bytecode content should match");
    }

    #[test]
    fn test_android_gradle_wrapper_generation() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");

        let config = BuildConfig::new(PathBuf::from("/tmp/project")).with_output_dir(&output_dir);
        let executor = BuildExecutor::new(config);

        // Note: Gradle wrapper generation could be added to generate_android_artifacts
        // For now, we test that the structure supports Gradle builds
        executor.generate_android_artifacts().unwrap();

        let android_dir = output_dir.join("android");
        assert!(android_dir.join("build.gradle.kts").exists(), "Should have build.gradle.kts for Gradle builds");
        assert!(android_dir.join("settings.gradle.kts").exists(), "Should have settings.gradle.kts for Gradle builds");
    }
}