#pragma once

#include <cstdint>
#include <string>

// System info structure
struct SystemInfo {
    char* os_name;
    char* os_version;
    uint32_t cpu_count;
    uint64_t memory_mb;
    char* hostname;
};

// System information gathering
class SystemInfoGatherer {
public:
    static SystemInfo* get_system_info();
    static void free_system_info(SystemInfo* info);

private:
    static std::string get_os_name();
    static std::string get_os_version();
    static uint32_t get_cpu_count();
    static uint64_t get_memory_mb();
    static std::string get_hostname();
};

// FFI interface
extern "C" {
    SystemInfo* get_system_info();
    void free_system_info(SystemInfo* info);
}