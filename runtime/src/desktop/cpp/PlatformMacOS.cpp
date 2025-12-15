#include "PlatformMacOS.h"
#include <iostream>

PlatformMacOS::PlatformMacOS()
    : ns_window_(nullptr), ns_view_(nullptr), gl_context_(nullptr), should_close_(false) {
}

PlatformMacOS::~PlatformMacOS() {
    destroy_window();
}

bool PlatformMacOS::create_window(const char* title, uint32_t width, uint32_t height, bool resizable, bool fullscreen) {
    std::cout << "Creating macOS window: " << title << " (" << width << "x" << height << ")" << std::endl;

    // TODO: Implement actual macOS window creation with NSWindow
    // For now, just return true as stub
    return true;
}

void PlatformMacOS::destroy_window() {
    // TODO: Clean up NSWindow, NSView, NSOpenGLContext
    ns_window_ = nullptr;
    ns_view_ = nullptr;
    gl_context_ = nullptr;
}

void PlatformMacOS::set_window_title(const char* title) {
    // TODO: Set NSWindow title
    std::cout << "Setting window title to: " << title << std::endl;
}

void PlatformMacOS::get_window_size(uint32_t* width, uint32_t* height) {
    // TODO: Get NSWindow frame size
    *width = 800;  // stub
    *height = 600; // stub
}

void PlatformMacOS::set_window_size(uint32_t width, uint32_t height) {
    // TODO: Set NSWindow frame size
    std::cout << "Setting window size to: " << width << "x" << height << std::endl;
}

void PlatformMacOS::swap_buffers() {
    // TODO: Flush OpenGL context
    std::cout << "Swapping buffers" << std::endl;
}

bool PlatformMacOS::should_close() const {
    return should_close_;
}

void* PlatformMacOS::get_native_handle() const {
    return ns_window_;
}