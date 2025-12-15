/*!
Desktop Runtime for Vela Applications

This module provides native desktop application support for Vela,
enabling high-performance GUI applications that run directly on
Windows, macOS, and Linux without web technologies.

The runtime consists of:
- DesktopRenderEngine (C++): Hardware-accelerated rendering with Skia
- VelaDesktopBridge: Safe FFI bridge between Rust and C++
- Platform APIs: Native OS integration (file system, network, etc.)
*/

use std::sync::{Arc, Mutex};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{info, error, warn};

pub mod bridge;
pub mod platform;
pub mod renderer;
pub mod system_apis;

/// Configuration for desktop applications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub fullscreen: bool,
    pub vsync: bool,
    pub target_fps: u32,
}

/// Main desktop runtime coordinator
pub struct VelaDesktopRuntime {
    config: DesktopConfig,
    render_engine: Arc<Mutex<DesktopRenderEngine>>,
    event_sender: mpsc::UnboundedSender<DesktopEvent>,
    event_receiver: mpsc::UnboundedReceiver<DesktopEvent>,
}

impl VelaDesktopRuntime {
    /// Create a new desktop runtime instance
    pub fn new(config: DesktopConfig) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let render_engine = Arc::new(Mutex::new(
            DesktopRenderEngine::new(&config)
                .context("Failed to create desktop render engine")?
        ));

        Ok(Self {
            config,
            render_engine,
            event_sender,
            event_receiver,
        })
    }

    /// Start the desktop application main loop
    pub async fn run(mut self) -> Result<()> {
        info!("Starting Vela Desktop Runtime v{}", env!("CARGO_PKG_VERSION"));
        info!("Application: {}", self.config.title);
        info!("Resolution: {}x{}", self.config.width, self.config.height);

        // Initialize platform-specific components
        platform::init()?;

        // Start render loop
        let render_handle = {
            let render_engine = Arc::clone(&self.render_engine);
            tokio::spawn(async move {
                Self::render_loop(render_engine).await;
            })
        };

        // Main event loop
        self.event_loop().await?;

        // Wait for render loop to finish
        render_handle.await?;
        Ok(())
    }

    /// Main event processing loop
    async fn event_loop(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                event = self.event_receiver.recv() => {
                    match event {
                        Some(DesktopEvent::Quit) => {
                            info!("Quit event received, shutting down");
                            break;
                        }
                        Some(event) => {
                            self.handle_event(event).await?;
                        }
                        None => break,
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle desktop-specific events
    async fn handle_event(&mut self, event: DesktopEvent) -> Result<()> {
        match event {
            DesktopEvent::WindowResized { width, height } => {
                info!("Window resized to {}x{}", width, height);
                // Update render engine viewport
            }
            DesktopEvent::KeyPressed { key, modifiers } => {
                // Handle keyboard input
                info!("Key pressed: {:?} with modifiers {:?}", key, modifiers);
            }
            DesktopEvent::MouseMoved { x, y } => {
                // Handle mouse movement
            }
            DesktopEvent::MouseClicked { button, x, y } => {
                // Handle mouse clicks
            }
            DesktopEvent::Quit => {
                // Already handled in event_loop
            }
        }
        Ok(())
    }

    /// Render loop running at target FPS
    async fn render_loop(render_engine: Arc<Mutex<DesktopRenderEngine>>) {
        let mut interval = tokio::time::interval(
            std::time::Duration::from_millis(16) // ~60 FPS
        );

        loop {
            interval.tick().await;

            let mut engine = render_engine.lock().unwrap();
            if let Err(e) = engine.render_frame() {
                error!("Render error: {}", e);
                break;
            }
        }
    }
}

/// Desktop-specific events
#[derive(Debug, Clone)]
pub enum DesktopEvent {
    WindowResized { width: u32, height: u32 },
    KeyPressed { key: KeyCode, modifiers: KeyModifiers },
    MouseMoved { x: f32, y: f32 },
    MouseClicked { button: MouseButton, x: f32, y: f32 },
    Quit,
}

/// Keyboard key codes
#[derive(Debug, Clone, Copy)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    Escape, Enter, Space, Backspace, Tab,
    Left, Right, Up, Down,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool, // Cmd on macOS, Win on Windows
}

/// Mouse buttons
#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Opaque handle to the C++ DesktopRenderEngine
pub struct DesktopRenderEngine {
    handle: bridge::DesktopRenderEngineHandle,
}

// DesktopRenderEngine needs to be Send for tokio::spawn
unsafe impl Send for DesktopRenderEngine {}

impl DesktopRenderEngine {
    /// Create a new render engine instance
    pub fn new(config: &DesktopConfig) -> Result<Self> {
        // This will call into C++ via FFI
        let handle = bridge::safe::create_desktop_render_engine(
            &config.title,
            config.width,
            config.height,
            config.resizable,
            config.fullscreen,
            config.vsync,
        ).map_err(|e| anyhow::anyhow!(e))?;

        if handle.0.is_null() {
            return Err(anyhow::anyhow!("Failed to create desktop render engine"));
        }

        Ok(Self { handle })
    }

    /// Render a single frame
    pub fn render_frame(&mut self) -> Result<()> {
        let success = bridge::safe::render_frame(&self.handle).map_err(|e| anyhow::anyhow!(e))?;
        if !success {
            return Err(anyhow::anyhow!("Render frame failed"));
        }
        Ok(())
    }
}

impl Drop for DesktopRenderEngine {
    fn drop(&mut self) {
        // Handle is automatically dropped by DesktopRenderEngineHandle
    }
}

/// Main entry point for desktop applications
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    // TODO: Initialize tracing properly when tracing-subscriber is added
    // tracing_subscriber::init();

    // Default desktop configuration
    let config = DesktopConfig {
        title: "Vela Desktop App".to_string(),
        width: 1280,
        height: 720,
        resizable: true,
        fullscreen: false,
        vsync: true,
        target_fps: 60,
    };

    // Create and run the desktop runtime
    let runtime = VelaDesktopRuntime::new(config)?;
    runtime.run().await
}