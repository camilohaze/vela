#include "PlatformWindows.h"
#include <iostream>
#include <windows.h>
#include <GL/gl.h>

PlatformWindows::PlatformWindows()
    : hwnd_(nullptr), hdc_(nullptr), hglrc_(nullptr), should_close_(false) {
}

PlatformWindows::~PlatformWindows() {
    destroy_window();
}

bool PlatformWindows::create_window(const char* title, uint32_t width, uint32_t height, bool resizable, bool fullscreen) {
    std::cout << "Creating Windows window: " << title << " (" << width << "x" << height << ")" << std::endl;

    // TODO: Implement actual Windows window creation
    // For now, just return true as stub
    return true;
}

void PlatformWindows::destroy_window() {
    if (hglrc_) {
        wglDeleteContext((HGLRC)hglrc_);
        hglrc_ = nullptr;
    }
    if (hdc_) {
        ReleaseDC((HWND)hwnd_, (HDC)hdc_);
        hdc_ = nullptr;
    }
    if (hwnd_) {
        DestroyWindow((HWND)hwnd_);
        hwnd_ = nullptr;
    }
}

void PlatformWindows::set_window_title(const char* title) {
    if (hwnd_) {
        SetWindowTextA((HWND)hwnd_, title);
    }
}

void PlatformWindows::get_window_size(uint32_t* width, uint32_t* height) {
    if (hwnd_) {
        RECT rect;
        GetClientRect((HWND)hwnd_, &rect);
        *width = rect.right - rect.left;
        *height = rect.bottom - rect.top;
    }
}

void PlatformWindows::set_window_size(uint32_t width, uint32_t height) {
    if (hwnd_) {
        RECT rect = {0, 0, (LONG)width, (LONG)height};
        AdjustWindowRect(&rect, WS_OVERLAPPEDWINDOW, FALSE);
        SetWindowPos((HWND)hwnd_, nullptr, 0, 0,
                    rect.right - rect.left, rect.bottom - rect.top,
                    SWP_NOMOVE | SWP_NOZORDER);
    }
}

void PlatformWindows::swap_buffers() {
    if (hdc_) {
        SwapBuffers((HDC)hdc_);
    }
}

bool PlatformWindows::should_close() const {
    return should_close_;
}

void* PlatformWindows::get_native_handle() const {
    return hwnd_;
}