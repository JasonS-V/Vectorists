//! Low-latency Windows Raw Input background listener and hardware telemetry.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Discrete hardware report from the optical mouse sensor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MousePacket {
    pub dx: i32,
    pub dy: i32,
    pub button_flags: u16,
    pub timestamp: Instant,
}

/// Real-time hardware telemetry and polling rate measurement.
#[derive(Debug, Clone, Default)]
pub struct SensorTelemetry {
    pub polling_rate_hz: u32,
    pub packet_count: u64,
    pub total_dx: i64,
    pub total_dy: i64,
    pub counts_per_sec: f32,
}

/// Thread-safe shared metrics.
pub struct SharedTelemetry {
    pub total_packets: AtomicU64,
    pub polling_hz: AtomicU32,
    pub counts_sec: AtomicU32, // Stored as scaled integer (cps * 10)
}

impl SharedTelemetry {
    pub fn new() -> Self {
        Self {
            total_packets: AtomicU64::new(0),
            polling_hz: AtomicU32::new(0),
            counts_sec: AtomicU32::new(0),
        }
    }

    pub fn snapshot(&self) -> SensorTelemetry {
        SensorTelemetry {
            polling_rate_hz: self.polling_hz.load(Ordering::Relaxed),
            packet_count: self.total_packets.load(Ordering::Relaxed),
            total_dx: 0,
            total_dy: 0,
            counts_per_sec: (self.counts_sec.load(Ordering::Relaxed) as f32) / 10.0,
        }
    }
}

pub struct RawInputListener {
    running: Arc<AtomicBool>,
    telemetry: Arc<SharedTelemetry>,
    receiver: Receiver<MousePacket>,
    thread_handle: Option<JoinHandle<()>>,
}

impl RawInputListener {
    /// Starts the background Raw Input listener thread.
    pub fn start() -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let telemetry = Arc::new(SharedTelemetry::new());
        let (tx, rx) = channel();

        let running_clone = running.clone();
        let telemetry_clone = telemetry.clone();

        let thread_handle = thread::spawn(move || {
            run_listener_loop(running_clone, telemetry_clone, tx);
        });

        Self {
            running,
            telemetry,
            receiver: rx,
            thread_handle: Some(thread_handle),
        }
    }

    /// Reads all pending mouse packets received since the last update.
    pub fn drain_packets(&self) -> Vec<MousePacket> {
        let mut packets = Vec::new();
        while let Ok(pkt) = self.receiver.try_recv() {
            packets.push(pkt);
        }
        packets
    }

    /// Returns the current real-time telemetry snapshot (Polling rate, PPS, etc.).
    pub fn telemetry(&self) -> SensorTelemetry {
        self.telemetry.snapshot()
    }

    /// Stops the background listener.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for RawInputListener {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(target_os = "windows")]
fn run_listener_loop(
    running: Arc<AtomicBool>,
    telemetry: Arc<SharedTelemetry>,
    tx: Sender<MousePacket>,
) {
    use std::mem::size_of;
    use windows_sys::Win32::UI::Input::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    unsafe {
        // Register window class for message-only window
        let class_name = b"VectoristsRawInputClass\0";
        let wnd_class = WNDCLASSA {
            style: 0,
            lpfnWndProc: Some(DefWindowProcA),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0 as _,
            hIcon: 0 as _,
            hCursor: 0 as _,
            hbrBackground: 0 as _,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };

        RegisterClassA(&wnd_class);

        // Create message-only window
        let hwnd = CreateWindowExA(
            0,
            class_name.as_ptr(),
            b"VectoristsHiddenWindow\0".as_ptr(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            0 as _,
            0 as _,
            std::ptr::null(),
        );

        // Register for Raw Input mouse device
        let rid = RAWINPUTDEVICE {
            usUsagePage: 0x01, // Generic Desktop Controls
            usUsage: 0x02,     // Mouse
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        };

        let reg_result = RegisterRawInputDevices(&rid, 1, size_of::<RAWINPUTDEVICE>() as u32);
        if reg_result == 0 {
            eprintln!("Vectorists: Failed to register Raw Input device.");
            return;
        }

        // Telemetry tracking variables
        let mut second_timer = Instant::now();
        let mut packets_this_second = 0u32;
        let mut counts_this_second = 0f32;

        let mut msg: MSG = std::mem::zeroed();

        while running.load(Ordering::Relaxed) {
            // Non-blocking message peek / pump with micro-sleep to prevent 100% spin
            while PeekMessageA(&mut msg, hwnd, 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_INPUT {
                    let mut raw: RAWINPUT = std::mem::zeroed();
                    let mut size = size_of::<RAWINPUT>() as u32;

                    let header_size = size_of::<RAWINPUTHEADER>() as u32;
                    let bytes = GetRawInputData(
                        msg.lParam as _,
                        RID_INPUT,
                        &mut raw as *mut _ as *mut _,
                        &mut size,
                        header_size,
                    );

                    if bytes != u32::MAX && raw.header.dwType == RIM_TYPEMOUSE {
                        let mouse = raw.data.mouse;
                        let dx = mouse.lLastX;
                        let dy = mouse.lLastY;
                        let button_flags = mouse.Anonymous.Anonymous.usButtonFlags;

                        let pkt = MousePacket {
                            dx,
                            dy,
                            button_flags,
                            timestamp: Instant::now(),
                        };

                        packets_this_second += 1;
                        let dist = ((dx * dx + dy * dy) as f32).sqrt();
                        counts_this_second += dist;
                        telemetry.total_packets.fetch_add(1, Ordering::Relaxed);

                        let _ = tx.send(pkt);
                    }
                }
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }

            // Update 1-second rolling telemetry window
            if second_timer.elapsed() >= Duration::from_millis(500) {
                let elapsed_secs = second_timer.elapsed().as_secs_f32();
                let hz = (packets_this_second as f32 / elapsed_secs).round() as u32;
                let cps = counts_this_second / elapsed_secs;

                telemetry.polling_hz.store(hz, Ordering::Relaxed);
                telemetry.counts_sec.store((cps * 10.0) as u32, Ordering::Relaxed);

                packets_this_second = 0;
                counts_this_second = 0.0;
                second_timer = Instant::now();
            }

            // Small sleep (100 microseconds) to yield CPU while staying responsive to 1-8kHz mouse interrupts
            thread::sleep(Duration::from_micros(100));
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn run_listener_loop(
    running: Arc<AtomicBool>,
    _telemetry: Arc<SharedTelemetry>,
    _tx: Sender<MousePacket>,
) {
    // Non-Windows fallback
    while running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_snapshot() {
        let shared = SharedTelemetry::new();
        shared.total_packets.store(42, Ordering::Relaxed);
        shared.polling_hz.store(1000, Ordering::Relaxed);
        shared.counts_sec.store(5520, Ordering::Relaxed); // 552.0 cps

        let snapshot = shared.snapshot();
        assert_eq!(snapshot.packet_count, 42);
        assert_eq!(snapshot.polling_rate_hz, 1000);
        assert!((snapshot.counts_per_sec - 552.0).abs() < 1e-4);
    }
}
