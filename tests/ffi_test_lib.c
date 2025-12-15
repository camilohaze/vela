// tests/ffi_test_lib.c - Librería de prueba C para validar FFI bridge
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <stdlib.h>

// Funciones de prueba simples para validar conversión de tipos

// Primitivos enteros
int32_t add_int32(int32_t a, int32_t b) {
    return a + b;
}

int64_t add_int64(int64_t a, int64_t b) {
    return a + b;
}

uint32_t add_uint32(uint32_t a, uint32_t b) {
    return a + b;
}

uint64_t add_uint64(uint64_t a, uint64_t b) {
    return a + b;
}

// Primitivos flotantes
float add_float(float a, float b) {
    return a + b;
}

double add_double(double a, double b) {
    return a * b;  // Multiplicación para variar
}

// Booleanos
bool is_even(int32_t n) {
    return n % 2 == 0;
}

bool both_true(bool a, bool b) {
    return a && b;
}

// Strings - gestión de memoria por caller (buffer estático)
const char* greet(const char* name) {
    static char buffer[256];
    snprintf(buffer, sizeof(buffer), "Hello, %s!", name);
    return buffer;
}

// Strings - gestión de memoria por callee (debe ser liberado por caller)
char* create_greeting(const char* name) {
    char* result = (char*)malloc(256);
    if (result) {
        snprintf(result, 256, "Greetings, %s!", name);
    }
    return result;
}

// Función con múltiples argumentos
int32_t sum_four(int32_t a, int32_t b, int32_t c, int32_t d) {
    return a + b + c + d;
}

// Función con diferentes tipos de argumentos
double mixed_calculation(int32_t a, double b, bool c) {
    double result = (double)a + b;
    if (c) {
        result *= 2.0;
    }
    return result;
}

// Función que retorna void (sin retorno)
void log_message(const char* message) {
    // En un entorno real, esto podría escribir a un log
    // Para testing, solo validamos que se llame sin crash
    (void)message;  // Suprimir warning de unused parameter
}

// Función que toma y retorna struct simple (simulado con punteros)
// Nota: Para mantener simplicidad, usamos arrays en lugar de structs
void process_array(int32_t* arr, size_t len) {
    for (size_t i = 0; i < len; i++) {
        arr[i] *= 2;  // Duplicar cada elemento
    }
}

// Función que retorna código de error
int32_t divide_safe(int32_t a, int32_t b, int32_t* result) {
    if (b == 0) {
        return -1;  // Error: división por cero
    }
    *result = a / b;
    return 0;  // Success
}

// Función con callback (simulado)
// typedef void (*callback_t)(int32_t);
void call_callback(void (*callback)(int32_t), int32_t value) {
    if (callback) {
        callback(value * 2);
    }
}