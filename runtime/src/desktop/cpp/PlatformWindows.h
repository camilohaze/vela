#pragma once

#include <string>
#include <cstdint>

/**
 * Windows platform implementation for desktop rendering
 */
class PlatformWindows {
public:
    PlatformWindows();
    ~PlatformWindows();

    bool create_window(const char* title, uint32_t width, uint32_t height, bool resizable, bool fullscreen);
    void destroy_window();
    void set_window_title(const char* title);
    void get_window_size(uint32_t* width, uint32_t* height);
    void set_window_size(uint32_t width, uint32_t height);
    void swap_buffers();
    bool should_close() const;
    void* get_native_handle() const;

private:
    void* hwnd_;        // HWND
    void* hdc_;         // HDC
    void* hglrc_;       // HGLRC
    bool should_close_;
};