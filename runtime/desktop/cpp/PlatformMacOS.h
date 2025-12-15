#pragma once

#include <string>
#include <cstdint>

/**
 * macOS platform implementation for desktop rendering
 */
class PlatformMacOS {
public:
    PlatformMacOS();
    ~PlatformMacOS();

    bool create_window(const char* title, uint32_t width, uint32_t height, bool resizable, bool fullscreen);
    void destroy_window();
    void set_window_title(const char* title);
    void get_window_size(uint32_t* width, uint32_t* height);
    void set_window_size(uint32_t width, uint32_t height);
    void swap_buffers();
    bool should_close() const;
    void* get_native_handle() const;

private:
    void* ns_window_;   // NSWindow*
    void* ns_view_;     // NSView*
    void* gl_context_;  // NSOpenGLContext*
    bool should_close_;
};