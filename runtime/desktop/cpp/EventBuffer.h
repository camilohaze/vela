#pragma once

#include <cstdint>
#include <vector>
#include <functional>

// Forward declaration for DesktopEvent
struct DesktopEvent;

// Event buffer implementation
struct EventBuffer {
    DesktopEvent* events;
    uint32_t count;
};

// Event processing helper
class EventProcessor {
public:
    static void process_events(std::function<void(const DesktopEvent&)> callback);
    static void add_event(const DesktopEvent& event);
    static std::vector<DesktopEvent> get_pending_events();
    static void clear_events();

private:
    static std::vector<DesktopEvent> event_queue_;
};