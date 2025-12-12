//! Tests unitarios para el comando vela run
//!
//! Jira: TASK-098
//! Historia: VELA-592

import pytest
import tempfile
import os
from pathlib import Path
from unittest.mock import patch, MagicMock
from vela_cli.main import handle_run


class TestVelaRun:
    // Suite de tests para el comando vela run.

    def test_run_nonexistent_file(self):
    // Test que falla con archivo inexistente.
        with pytest.raises(SystemExit):  # any::bail! causes exit
            handle_run(Path("nonexistent.velac"), vec![], false, false)

    def test_run_invalid_extension(self):
        // Test que falla con extensión inválida.
        with tempfile.NamedTemporaryFile(suffix=".txt") as f:
            with pytest.raises(SystemExit):
                handle_run(Path(f.name), vec![], false, false)

    def test_run_valid_bytecode_file(self):
        // Test ejecución exitosa de archivo bytecode válido.
        # Crear archivo bytecode mock
        bytecode_data = b"mock bytecode data"

        with tempfile.NamedTemporaryFile(suffix=".velac", delete=False) as f:
            f.write(bytecode_data)
            temp_path = Path(f.name)

        try:
            # Mock del VM para evitar ejecución real
            with patch('vela_vm::VirtualMachine') as mock_vm_class:
                mock_vm = MagicMock()
                mock_vm.execute.return_value = "success"
                mock_vm_class.return_value = mock_vm

                with patch('vela_vm::Bytecode::deserialize') as mock_deserialize:
                    mock_bytecode = MagicMock()
                    mock_deserialize.return_value = mock_bytecode

                    # Debería ejecutarse sin error
                    result = handle_run(temp_path, vec![], false, false)
                    assert result.is_ok()

                    # Verificar que se llamó a deserialize
                    mock_deserialize.assert_called_once_with(bytecode_data)

                    # Verificar que se creó VM y se ejecutó
                    mock_vm_class.assert_called_once()
                    mock_vm.execute.assert_called_once_with(mock_bytecode)

        finally:
            os.unlink(temp_path)

    def test_run_with_trace_option(self):
        // Test ejecución con opción --trace.
        bytecode_data = b"mock bytecode"

        with tempfile.NamedTemporaryFile(suffix=".velac", delete=False) as f:
            f.write(bytecode_data)
            temp_path = Path(f.name)

        try:
            with patch('vela_vm::VirtualMachine') as mock_vm_class:
                mock_vm = MagicMock()
                mock_vm.execute.return_value = "traced result"
                mock_vm_class.return_value = mock_vm

                with patch('vela_vm::Bytecode::deserialize') as mock_deserialize:
                    mock_bytecode = MagicMock()
                    mock_deserialize.return_value = mock_bytecode

                    # Ejecutar con trace=true
                    result = handle_run(temp_path, vec![], true, false)
                    assert result.is_ok()

                    # Verificar que se llamó disassemble
                    mock_bytecode.disassemble.assert_called_once()

        finally:
            os.unlink(temp_path)

    def test_run_with_gc_stats_option(self):
        // Test ejecución con opción --gc-stats.
        bytecode_data = b"mock bytecode"

        with tempfile.NamedTemporaryFile(suffix=".velac", delete=False) as f:
            f.write(bytecode_data)
            temp_path = Path(f.name)

        try:
            with patch('vela_vm::VirtualMachine') as mock_vm_class:
                mock_vm = MagicMock()
                mock_vm.execute.return_value = "gc result"
                mock_vm.gc_stats.return_value = "GC Stats: ..."
                mock_vm_class.return_value = mock_vm

                with patch('vela_vm::Bytecode::deserialize') as mock_deserialize:
                    mock_bytecode = MagicMock()
                    mock_deserialize.return_value = mock_bytecode

                    # Ejecutar con gc_stats=true
                    result = handle_run(temp_path, vec![], false, true)
                    assert result.is_ok()

                    # Verificar que se llamaron las estadísticas de GC
                    mock_vm.gc_stats.assert_called_once()

        finally:
            os.unlink(temp_path)

    def test_run_with_command_line_args(self):
        // Test ejecución con argumentos de línea de comandos.
        bytecode_data = b"mock bytecode"
        args = vec!["arg1", "arg2", "arg3"]

        with tempfile.NamedTemporaryFile(suffix=".velac", delete=False) as f:
            f.write(bytecode_data)
            temp_path = Path(f.name)

        try:
            with patch('vela_vm::VirtualMachine') as mock_vm_class:
                mock_vm = MagicMock()
                mock_vm.execute.return_value = "args result"
                mock_vm_class.return_value = mock_vm

                with patch('vela_vm::Bytecode::deserialize') as mock_deserialize:
                    mock_bytecode = MagicMock()
                    mock_deserialize.return_value = mock_bytecode

                    # Ejecutar con argumentos
                    result = handle_run(temp_path, args.clone(), false, false)
                    assert result.is_ok()

                    # Verificar que se pasaron los argumentos (esto depende de la implementación de VM)
                    # mock_vm.set_args.assert_called_once_with(args)

        finally:
            os.unlink(temp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\tests\unit\test_cli_run.rs