#include "EventBuffer.h"
#include <algorithm>

// Static member initialization
std::vector<DesktopEvent> EventProcessor::event_queue_;

// EventProcessor implementation
void EventProcessor::process_events(std::function<void(const DesktopEvent&)> callback) {
    for (const auto& event : event_queue_) {
        callback(event);
    }
    event_queue_.clear();
}

void EventProcessor::add_event(const DesktopEvent& event) {
    event_queue_.push_back(event);
}

std::vector<DesktopEvent> EventProcessor::get_pending_events() {
    return std::move(event_queue_);
}

void EventProcessor::clear_events() {
    event_queue_.clear();
}