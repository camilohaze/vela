#include "DesktopRenderEngine.h"
#include <iostream>
#include <cstring>
#include <chrono>
#include <thread>

// Platform-specific implementations
#ifdef WIN32
#include "PlatformWindows.h"
#endif

#ifdef __APPLE__
#include "PlatformMacOS.h"
#endif

#ifdef __linux__
#include "PlatformLinux.h"
#endif

// DesktopRenderEngine implementation
DesktopRenderEngine::DesktopRenderEngine(
    const char* title,
    uint32_t width,
    uint32_t height,
    bool resizable,
    bool fullscreen,
    bool vsync
) : title_(title, strnlen(title, 256)),
    width_(width),
    height_(height),
    resizable_(resizable),
    fullscreen_(fullscreen),
    vsync_(vsync),
    skia_canvas_(nullptr) {

    std::cout << "Initializing DesktopRenderEngine: " << title_ << " (" << width << "x" << height << ")" << std::endl;

    initialize_platform();
    setup_window();
}

DesktopRenderEngine::~DesktopRenderEngine() {
    std::cout << "Destroying DesktopRenderEngine" << std::endl;
    // Cleanup will be handled by platform implementations
}

void DesktopRenderEngine::initialize_platform() {
#ifdef WIN32
    platform_impl_ = std::make_unique<PlatformWindows>();
#endif

#ifdef __APPLE__
    platform_impl_ = std::make_unique<PlatformMacOS>();
#endif

#ifdef __linux__
    platform_impl_ = std::make_unique<PlatformLinux>();
#endif

    if (!platform_impl_) {
        throw std::runtime_error("Unsupported platform");
    }
}

void DesktopRenderEngine::setup_window() {
    platform_impl_->create_window(title_.c_str(), width_, height_, resizable_, fullscreen_);
    std::cout << "Window setup completed" << std::endl;
}

bool DesktopRenderEngine::render_frame() {
    // Stub implementation - no rendering yet
    std::cout << "Render frame called (no-op)" << std::endl;
    return true;
}

void DesktopRenderEngine::set_window_title(const char* title, uint32_t len) {
    title_ = std::string(title, len);
    platform_impl_->set_window_title(title_.c_str());
}

void DesktopRenderEngine::get_window_size(uint32_t* width, uint32_t* height) {
    platform_impl_->get_window_size(width, height);
}

void DesktopRenderEngine::set_window_size(uint32_t width, uint32_t height) {
    platform_impl_->set_window_size(width, height);
}

bool DesktopRenderEngine::should_close() const {
    return platform_impl_->should_close();
}

void* DesktopRenderEngine::get_native_handle() const {
    return platform_impl_->get_native_handle();
}

void DesktopRenderEngine::get_window_size(uint32_t* width, uint32_t* height) {
    *width = width_;
    *height = height_;
}

void DesktopRenderEngine::set_window_size(uint32_t width, uint32_t height) {
    width_ = width;
    height_ = height;
    platform_impl_->resize_window(width, height);

    // Recreate Skia surface with new size
    setup_window();
}

EventBuffer* DesktopRenderEngine::poll_events() {
    event_queue_.clear();
    process_platform_events();

    if (event_queue_.empty()) {
        return nullptr;
    }

    EventBuffer* buffer = new EventBuffer();
    buffer->count = static_cast<uint32_t>(event_queue_.size());
    buffer->events = new DesktopEvent[buffer->count];
    std::memcpy(buffer->events, event_queue_.data(), sizeof(DesktopEvent) * buffer->count);

    return buffer;
}

void DesktopRenderEngine::free_event_buffer(EventBuffer* buffer) {
    if (buffer) {
        delete[] buffer->events;
        delete buffer;
    }
}

void* DesktopRenderEngine::get_native_window_handle() {
    return platform_impl_->get_native_handle();
}

void DesktopRenderEngine::process_platform_events() {
    platform_impl_->process_events([this](const DesktopEvent& event) {
        event_queue_.push_back(event);
    });
}

void DesktopRenderEngine::add_event(const DesktopEvent& event) {
    event_queue_.push_back(event);
}

// FFI interface implementations
extern "C" {

void* create_desktop_render_engine(
    const char* title,
    uint32_t title_len,
    uint32_t width,
    uint32_t height,
    bool resizable,
    bool fullscreen,
    bool vsync
) {
    try {
        auto engine = new DesktopRenderEngine(title, width, height, resizable, fullscreen, vsync);
        return static_cast<void*>(engine);
    } catch (const std::exception& e) {
        std::cerr << "Failed to create desktop render engine: " << e.what() << std::endl;
        return nullptr;
    }
}

void destroy_desktop_render_engine(void* handle) {
    if (handle) {
        delete static_cast<DesktopRenderEngine*>(handle);
    }
}

bool render_frame(void* handle) {
    if (!handle) return false;
    auto engine = static_cast<DesktopRenderEngine*>(handle);
    return engine->render_frame();
}

void set_window_title(void* handle, const char* title, uint32_t len) {
    if (handle) {
        auto engine = static_cast<DesktopRenderEngine*>(handle);
        engine->set_window_title(title, len);
    }
}

void get_window_size(const void* handle, uint32_t* width, uint32_t* height) {
    if (handle) {
        auto engine = static_cast<DesktopRenderEngine*>(handle);
        engine->get_window_size(width, height);
    }
}

void set_window_size(void* handle, uint32_t width, uint32_t height) {
    if (handle) {
        auto engine = static_cast<DesktopRenderEngine*>(handle);
        engine->set_window_size(width, height);
    }
}

EventBuffer* poll_events(void* handle) {
    if (!handle) return nullptr;
    auto engine = static_cast<DesktopRenderEngine*>(handle);
    return engine->poll_events();
}

void free_event_buffer(EventBuffer* buffer) {
    DesktopRenderEngine::free_event_buffer(buffer);
}

}