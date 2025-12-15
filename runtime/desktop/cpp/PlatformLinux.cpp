#include "PlatformLinux.h"
#include <iostream>

PlatformLinux::PlatformLinux()
    : display_(nullptr), window_(nullptr), gl_context_(nullptr), should_close_(false) {
}

PlatformLinux::~PlatformLinux() {
    destroy_window();
}

bool PlatformLinux::create_window(const char* title, uint32_t width, uint32_t height, bool resizable, bool fullscreen) {
    std::cout << "Creating Linux window: " << title << " (" << width << "x" << height << ")" << std::endl;

    // TODO: Implement actual X11 window creation
    // For now, just return true as stub
    return true;
}

void PlatformLinux::destroy_window() {
    // TODO: Clean up X11 window and GLX context
    display_ = nullptr;
    window_ = nullptr;
    gl_context_ = nullptr;
}

void PlatformLinux::set_window_title(const char* title) {
    // TODO: Set X11 window title
    std::cout << "Setting window title to: " << title << std::endl;
}

void PlatformLinux::get_window_size(uint32_t* width, uint32_t* height) {
    // TODO: Get X11 window size
    *width = 800;  // stub
    *height = 600; // stub
}

void PlatformLinux::set_window_size(uint32_t width, uint32_t height) {
    // TODO: Set X11 window size
    std::cout << "Setting window size to: " << width << "x" << height << std::endl;
}

void PlatformLinux::swap_buffers() {
    // TODO: Swap GLX buffers
    std::cout << "Swapping buffers" << std::endl;
}

bool PlatformLinux::should_close() const {
    return should_close_;
}

void* PlatformLinux::get_native_handle() const {
    return window_;
}