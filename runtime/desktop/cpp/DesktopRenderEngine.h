#pragma once

#include <cstdint>
#include <string>
#include <memory>
#include <vector>

#ifdef WIN32
#include <windows.h>
#endif

#ifdef __APPLE__
#include <Cocoa/Cocoa.h>
#endif

#ifdef __linux__
#include <X11/Xlib.h>
#endif

// Forward declarations for Skia
class SkCanvas;
class SkSurface;
class GrDirectContext;

// Event types matching Rust enum
enum class DesktopEventType {
    WindowResized,
    KeyPressed,
    KeyReleased,
    MouseMoved,
    MousePressed,
    MouseReleased,
    Quit,
};

// Desktop event structure
struct DesktopEvent {
    DesktopEventType event_type;
    union EventData {
        struct {
            uint32_t width;
            uint32_t height;
        } window_resized;

        struct {
            uint32_t key_code;
            uint32_t modifiers;
        } key;

        struct {
            uint32_t button;
            float x;
            float y;
        } mouse;
    } data;
};

// Event buffer for FFI
struct EventBuffer {
    DesktopEvent* events;
    uint32_t count;
};

// File buffer for FFI
struct FileBuffer {
    uint8_t* data;
    uint64_t size;
};

// System info for FFI
struct SystemInfo {
    char* os_name;
    char* os_version;
    uint32_t cpu_count;
    uint64_t memory_mb;
    char* hostname;
};

// Main desktop render engine class
class DesktopRenderEngine {
public:
    DesktopRenderEngine(
        const char* title,
        uint32_t width,
        uint32_t height,
        bool resizable,
        bool fullscreen,
        bool vsync
    );

    ~DesktopRenderEngine();

    // Render operations
    bool render_frame();

    // Window management
    void set_window_title(const char* title, uint32_t len);
    void get_window_size(uint32_t* width, uint32_t* height);
    void set_window_size(uint32_t width, uint32_t height);

    // Event handling
    EventBuffer* poll_events();
    static void free_event_buffer(EventBuffer* buffer);

    // Platform-specific window handle
    void* get_native_window_handle();

private:
    // Platform-specific implementation
    class PlatformImpl;
    std::unique_ptr<PlatformImpl> platform_impl_;

    // Skia rendering context
    std::unique_ptr<GrDirectContext> skia_context_;
    std::unique_ptr<SkSurface> skia_surface_;
    SkCanvas* skia_canvas_;

    // Window properties
    std::string title_;
    uint32_t width_;
    uint32_t height_;
    bool resizable_;
    bool fullscreen_;
    bool vsync_;

    // Event queue
    std::vector<DesktopEvent> event_queue_;

    // Platform-specific initialization
    void initialize_platform();
    void create_skia_context();
    void setup_window();

    // Event processing
    void process_platform_events();
    void add_event(const DesktopEvent& event);
};

// FFI interface functions
extern "C" {
    // Render Engine lifecycle
    void* create_desktop_render_engine(
        const char* title,
        uint32_t title_len,
        uint32_t width,
        uint32_t height,
        bool resizable,
        bool fullscreen,
        bool vsync
    );

    void destroy_desktop_render_engine(void* handle);

    // Render operations
    bool render_frame(void* handle);

    // Window management
    void set_window_title(void* handle, const char* title, uint32_t len);
    void get_window_size(const void* handle, uint32_t* width, uint32_t* height);
    void set_window_size(void* handle, uint32_t width, uint32_t height);

    // Event handling
    EventBuffer* poll_events(void* handle);
    void free_event_buffer(EventBuffer* buffer);
}