#!/bin/bash
# build_ffi_test_lib.sh - Script para compilar la librería de prueba C

set -e  # Salir en caso de error

echo "Building FFI test library..."

# Detectar sistema operativo
if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    # Windows con MSYS2/MinGW
    echo "Building for Windows..."
    gcc -shared -o libtestffi.dll tests/ffi_test_lib.c -Wl,--out-implib,libtestffi.a
    echo "Created: libtestffi.dll"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    echo "Building for macOS..."
    gcc -shared -o libtestffi.dylib tests/ffi_test_lib.c
    echo "Created: libtestffi.dylib"
else
    # Linux y otros Unix
    echo "Building for Linux/Unix..."
    gcc -shared -o libtestffi.so tests/ffi_test_lib.c -fPIC
    echo "Created: libtestffi.so"
fi

echo "FFI test library built successfully!"