#include "SystemInfo.h"
#include <iostream>
#include <cstring>
#include <thread>

#ifdef WIN32
#include <windows.h>
#include <sysinfoapi.h>
#endif

#ifdef __APPLE__
#include <sys/sysctl.h>
#include <unistd.h>
#endif

#ifdef __linux__
#include <sys/sysinfo.h>
#include <unistd.h>
#endif

// SystemInfoGatherer implementation
SystemInfo* SystemInfoGatherer::get_system_info() {
    try {
        SystemInfo* info = new SystemInfo();

        std::string os_name = get_os_name();
        std::string os_version = get_os_version();
        std::string hostname = get_hostname();

        info->os_name = strdup(os_name.c_str());
        info->os_version = strdup(os_version.c_str());
        info->cpu_count = get_cpu_count();
        info->memory_mb = get_memory_mb();
        info->hostname = strdup(hostname.c_str());

        return info;
    } catch (const std::exception& e) {
        std::cerr << "System info error: " << e.what() << std::endl;
        return nullptr;
    }
}

void SystemInfoGatherer::free_system_info(SystemInfo* info) {
    if (info) {
        free(info->os_name);
        free(info->os_version);
        free(info->hostname);
        delete info;
    }
}

std::string SystemInfoGatherer::get_os_name() {
#ifdef WIN32
    return "Windows";
#endif

#ifdef __APPLE__
    return "macOS";
#endif

#ifdef __linux__
    return "Linux";
#endif

    return "Unknown";
}

std::string SystemInfoGatherer::get_os_version() {
#ifdef WIN32
    // TODO: Get Windows version
    return "10.0";
#endif

#ifdef __APPLE__
    // TODO: Get macOS version
    return "12.0";
#endif

#ifdef __linux__
    // TODO: Get Linux version
    return "5.0";
#endif

    return "Unknown";
}

uint32_t SystemInfoGatherer::get_cpu_count() {
    return std::thread::hardware_concurrency();
}

uint64_t SystemInfoGatherer::get_memory_mb() {
#ifdef WIN32
    MEMORYSTATUSEX status;
    status.dwLength = sizeof(status);
    GlobalMemoryStatusEx(&status);
    return status.ullTotalPhys / (1024 * 1024);
#endif

#ifdef __APPLE__
    int mib[2] = { CTL_HW, HW_MEMSIZE };
    uint64_t memsize;
    size_t len = sizeof(memsize);
    sysctl(mib, 2, &memsize, &len, NULL, 0);
    return memsize / (1024 * 1024);
#endif

#ifdef __linux__
    struct sysinfo info;
    sysinfo(&info);
    return (info.totalram * info.mem_unit) / (1024 * 1024);
#endif

    return 0;
}

std::string SystemInfoGatherer::get_hostname() {
    char hostname[256];
    if (gethostname(hostname, sizeof(hostname)) == 0) {
        return std::string(hostname);
    }
    return "localhost";
}

// FFI interface implementations
extern "C" {

SystemInfo* get_system_info() {
    return SystemInfoGatherer::get_system_info();
}

void free_system_info(SystemInfo* info) {
    SystemInfoGatherer::free_system_info(info);
}

}