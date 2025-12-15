//
//  VelaBridge.h
//  Vela iOS Runtime
//
//  C header file for FFI declarations used by Swift.
//  This file defines the interface between Rust and Swift.
//

#ifndef VelaBridge_h
#define VelaBridge_h

#include <stdbool.h>
#include <stdint.h>

// Opaque pointer to Vela iOS runtime
typedef void* VelaIOSRuntime;

// iOS runtime configuration
typedef struct {
    bool debug_logging;
    uint32_t max_view_pool_size;
    bool enable_gestures;
} IOSRuntimeConfig;

// iOS touch event data
typedef struct {
    uint32_t event_type;  // 0=touch_began, 1=touch_moved, 2=touch_ended
    float x;
    float y;
    float pressure;       // 0.0-1.0
    uint64_t timestamp;   // microseconds since epoch
} IOSTouchEvent;

// iOS gesture event data
typedef struct {
    uint32_t gesture_type;  // 0=pinch, 1=rotate, 2=pan, 3=long_press
    float scale;            // for pinch gestures
    float rotation;         // radians, for rotate gestures
    float velocity_x;       // for pan gestures
    float velocity_y;       // for pan gestures
} IOSGestureEvent;

// iOS rectangle structure
typedef struct {
    float x;
    float y;
    float width;
    float height;
} IOSRect;

// FFI function declarations
#ifdef __cplusplus
extern "C" {
#endif

// Runtime lifecycle
VelaIOSRuntime* vela_ios_create_runtime(const IOSRuntimeConfig* config);
void vela_ios_destroy_runtime(VelaIOSRuntime* runtime);

// Widget management
void* vela_ios_render_widget(VelaIOSRuntime* runtime, const char* widget_json, void* parent_view);
int32_t vela_ios_update_widget(VelaIOSRuntime* runtime, uint64_t widget_id, const char* updates_json);
int32_t vela_ios_destroy_widget(VelaIOSRuntime* runtime, uint64_t widget_id);

// Event handling
bool vela_ios_handle_touch_event(VelaIOSRuntime* runtime, uint64_t widget_id, const IOSTouchEvent* event);
bool vela_ios_handle_gesture_event(VelaIOSRuntime* runtime, uint64_t widget_id, const IOSGestureEvent* event);

// Widget queries
bool vela_ios_get_widget_bounds(VelaIOSRuntime* runtime, uint64_t widget_id, IOSRect* bounds);

#ifdef __cplusplus
}
#endif

#endif /* VelaBridge_h */