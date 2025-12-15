//
//  VelaBridge.swift
//  Vela iOS Runtime
//
//  Swift bridging layer for Vela runtime FFI functions.
//  This file provides a safe, Swift-friendly API to interact with the Vela runtime.
//

import Foundation

/// Configuration for Vela iOS runtime
public struct VelaRuntimeConfig {
    /// Enable debug logging
    public var debugLogging: Bool = false
    /// Maximum UIView pool size for performance
    public var maxViewPoolSize: UInt32 = 100
    /// Enable gesture recognition
    public var enableGestures: Bool = true

    /// Convert to FFI-compatible structure
    func toFFI() -> IOSRuntimeConfig {
        IOSRuntimeConfig(
            debug_logging: debugLogging,
            max_view_pool_size: maxViewPoolSize,
            enable_gestures: enableGestures
        )
    }
}

/// Main bridge class for Vela iOS integration
public class VelaBridge {
    /// Opaque pointer to Vela runtime
    private var runtime: OpaquePointer?

    /// Initialize Vela bridge with configuration
    /// - Parameter config: Runtime configuration
    public init(config: VelaRuntimeConfig = VelaRuntimeConfig()) {
        self.runtime = vela_ios_create_runtime(config.toFFI())
        guard self.runtime != nil else {
            fatalError("Failed to create Vela iOS runtime")
        }
    }

    /// Render a Vela widget from JSON description
    /// - Parameters:
    ///   - json: JSON string describing the widget
    ///   - parent: Parent UIView for layout
    /// - Returns: Rendered UIView or nil if failed
    public func renderWidget(json: String, parent: UIView) -> UIView? {
        guard let runtime = self.runtime else { return nil }

        // Convert Swift string to C string
        guard let cJson = json.cString(using: .utf8) else { return nil }

        // Call FFI function
        let viewPointer = vela_ios_render_widget(
            runtime,
            cJson,
            Unmanaged.passUnretained(parent).toOpaque()
        )

        // Convert opaque pointer back to UIView
        guard let viewPtr = viewPointer else { return nil }
        return Unmanaged<UIView>.fromOpaque(viewPtr).takeRetainedValue()
    }

    /// Update an existing widget with new properties
    /// - Parameters:
    ///   - widgetId: Unique identifier of the widget
    ///   - updatesJson: JSON string with property updates
    /// - Returns: Success status
    public func updateWidget(widgetId: UInt64, updatesJson: String) -> Bool {
        guard let runtime = self.runtime else { return false }

        guard let cUpdates = updatesJson.cString(using: .utf8) else { return false }

        let result = vela_ios_update_widget(runtime, widgetId, cUpdates)
        return result == 0 // 0 = success
    }

    /// Destroy a widget and free its resources
    /// - Parameter widgetId: Unique identifier of the widget
    /// - Returns: Success status
    public func destroyWidget(widgetId: UInt64) -> Bool {
        guard let runtime = self.runtime else { return false }

        let result = vela_ios_destroy_widget(runtime, widgetId)
        return result == 0 // 0 = success
    }

    /// Handle touch event from iOS
    /// - Parameters:
    ///   - widgetId: Target widget identifier
    ///   - event: Touch event data
    /// - Returns: Whether the event was handled
    public func handleTouchEvent(widgetId: UInt64, event: VelaTouchEvent) -> Bool {
        guard let runtime = self.runtime else { return false }

        var ffiEvent = event.toFFI()
        return vela_ios_handle_touch_event(runtime, widgetId, &ffiEvent)
    }

    /// Handle gesture event from iOS
    /// - Parameters:
    ///   - widgetId: Target widget identifier
    ///   - event: Gesture event data
    /// - Returns: Whether the event was handled
    public func handleGestureEvent(widgetId: UInt64, event: VelaGestureEvent) -> Bool {
        guard let runtime = self.runtime else { return false }

        var ffiEvent = event.toFFI()
        return vela_ios_handle_gesture_event(runtime, widgetId, &ffiEvent)
    }

    /// Get widget bounds
    /// - Parameter widgetId: Widget identifier
    /// - Returns: Rectangle bounds or nil if widget not found
    public func getWidgetBounds(widgetId: UInt64) -> CGRect? {
        guard let runtime = self.runtime else { return nil }

        var bounds = IOSRect(x: 0, y: 0, width: 0, height: 0)
        let success = vela_ios_get_widget_bounds(runtime, widgetId, &bounds)

        if success {
            return CGRect(x: CGFloat(bounds.x),
                         y: CGFloat(bounds.y),
                         width: CGFloat(bounds.width),
                         height: CGFloat(bounds.height))
        }

        return nil
    }

    /// Clean up resources
    deinit {
        if let runtime = self.runtime {
            vela_ios_destroy_runtime(runtime)
        }
    }
}

/// Touch event types
public enum VelaTouchEvent {
    case began(x: Float, y: Float, pressure: Float = 1.0)
    case moved(x: Float, y: Float, pressure: Float = 1.0)
    case ended(x: Float, y: Float, pressure: Float = 0.0)

    func toFFI() -> IOSTouchEvent {
        let timestamp = UInt64(Date().timeIntervalSince1970 * 1_000_000) // microseconds

        switch self {
        case .began(let x, let y, let pressure):
            return IOSTouchEvent(
                event_type: 0, // TOUCH_BEGAN
                x: x, y: y, pressure: pressure, timestamp: timestamp
            )
        case .moved(let x, let y, let pressure):
            return IOSTouchEvent(
                event_type: 1, // TOUCH_MOVED
                x: x, y: y, pressure: pressure, timestamp: timestamp
            )
        case .ended(let x, let y, let pressure):
            return IOSTouchEvent(
                event_type: 2, // TOUCH_ENDED
                x: x, y: y, pressure: pressure, timestamp: timestamp
            )
        }
    }
}

/// Gesture event types
public enum VelaGestureEvent {
    case pinch(scale: Float)
    case rotate(rotation: Float)
    case pan(velocityX: Float, velocityY: Float)
    case longPress

    func toFFI() -> IOSGestureEvent {
        switch self {
        case .pinch(let scale):
            return IOSGestureEvent(
                gesture_type: 0, // PINCH
                scale: scale, rotation: 0, velocity_x: 0, velocity_y: 0
            )
        case .rotate(let rotation):
            return IOSGestureEvent(
                gesture_type: 1, // ROTATE
                scale: 1.0, rotation: rotation, velocity_x: 0, velocity_y: 0
            )
        case .pan(let velocityX, let velocityY):
            return IOSGestureEvent(
                gesture_type: 2, // PAN
                scale: 1.0, rotation: 0, velocity_x: velocityX, velocity_y: velocityY
            )
        case .longPress:
            return IOSGestureEvent(
                gesture_type: 3, // LONG_PRESS
                scale: 1.0, rotation: 0, velocity_x: 0, velocity_y: 0
            )
        }
    }
}