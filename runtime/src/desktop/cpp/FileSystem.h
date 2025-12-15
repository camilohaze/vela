#pragma once

#include <cstdint>
#include <string>
#include <vector>

// File buffer for FFI
struct FileBuffer {
    uint8_t* data;
    uint64_t size;
};

// File system operations
class FileSystem {
public:
    static FileBuffer* read_file(const char* path, uint32_t path_len);
    static bool write_file(const char* path, uint32_t path_len, const uint8_t* data, uint32_t data_len);
    static void free_file_buffer(FileBuffer* buffer);

private:
    static std::vector<uint8_t> read_file_contents(const std::string& path);
    static bool write_file_contents(const std::string& path, const std::vector<uint8_t>& data);
};

// FFI interface
extern "C" {
    FileBuffer* read_file(const char* path, uint32_t len);
    bool write_file(const char* path, uint32_t path_len, const uint8_t* data, uint32_t data_len);
    void free_file_buffer(FileBuffer* buffer);
}