#include "FileSystem.h"
#include <fstream>
#include <iostream>
#include <cstring>

// FileSystem implementation
FileBuffer* FileSystem::read_file(const char* path, uint32_t path_len) {
    try {
        std::string path_str(path, path_len);
        auto data = read_file_contents(path_str);

        FileBuffer* buffer = new FileBuffer();
        buffer->size = data.size();
        buffer->data = new uint8_t[buffer->size];
        std::memcpy(buffer->data, data.data(), buffer->size);

        return buffer;
    } catch (const std::exception& e) {
        std::cerr << "File read error: " << e.what() << std::endl;
        return nullptr;
    }
}

bool FileSystem::write_file(const char* path, uint32_t path_len, const uint8_t* data, uint32_t data_len) {
    try {
        std::string path_str(path, path_len);
        std::vector<uint8_t> data_vec(data, data + data_len);
        return write_file_contents(path_str, data_vec);
    } catch (const std::exception& e) {
        std::cerr << "File write error: " << e.what() << std::endl;
        return false;
    }
}

void FileSystem::free_file_buffer(FileBuffer* buffer) {
    if (buffer) {
        delete[] buffer->data;
        delete buffer;
    }
}

std::vector<uint8_t> FileSystem::read_file_contents(const std::string& path) {
    std::ifstream file(path, std::ios::binary | std::ios::ate);
    if (!file) {
        throw std::runtime_error("Cannot open file: " + path);
    }

    std::streamsize size = file.tellg();
    file.seekg(0, std::ios::beg);

    std::vector<uint8_t> buffer(size);
    if (!file.read(reinterpret_cast<char*>(buffer.data()), size)) {
        throw std::runtime_error("Failed to read file: " + path);
    }

    return buffer;
}

bool FileSystem::write_file_contents(const std::string& path, const std::vector<uint8_t>& data) {
    std::ofstream file(path, std::ios::binary);
    if (!file) {
        return false;
    }

    file.write(reinterpret_cast<const char*>(data.data()), data.size());
    return file.good();
}

// FFI interface implementations
extern "C" {

FileBuffer* read_file(const char* path, uint32_t len) {
    return FileSystem::read_file(path, len);
}

bool write_file(const char* path, uint32_t path_len, const uint8_t* data, uint32_t data_len) {
    return FileSystem::write_file(path, path_len, data, data_len);
}

void free_file_buffer(FileBuffer* buffer) {
    FileSystem::free_file_buffer(buffer);
}

}