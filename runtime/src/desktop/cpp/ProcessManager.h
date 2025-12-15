#pragma once

#include <cstdint>
#include <string>
#include <vector>

// Process management operations
class ProcessManager {
public:
    static uint32_t spawn_process(const char* cmd, uint32_t cmd_len, const char** args, uint32_t arg_count);
    static bool kill_process(uint32_t pid);
    static int32_t wait_process(uint32_t pid);

private:
    static std::string join_command(const char* cmd, uint32_t cmd_len, const char** args, uint32_t arg_count);
};

// FFI interface
extern "C" {
    uint32_t spawn_process(const char* cmd, uint32_t cmd_len, const char** args, uint32_t arg_count);
    bool kill_process(uint32_t pid);
    int32_t wait_process(uint32_t pid);
}