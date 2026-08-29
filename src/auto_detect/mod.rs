//! Foreground Window & Game Process Auto-Detection

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HWND, MAX_PATH};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub struct ProcessDetector {
    running: Arc<AtomicBool>,
}

impl ProcessDetector {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Retrieve the filename of the current foreground process (e.g. "rustclient.exe", "cs2.exe")
    #[cfg(windows)]
    pub fn get_foreground_process_name() -> Option<String> {
        unsafe {
            let hwnd: HWND = GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }

            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut process_id);
            if process_id == 0 {
                return None;
            }

            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
            if handle.is_null() {
                return None;
            }

            let mut buffer = [0u16; MAX_PATH as usize * 2];
            let mut size = buffer.len() as u32;

            let success = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut size);
            CloseHandle(handle);

            if success != 0 && size > 0 {
                let full_path = String::from_utf16_lossy(&buffer[..size as usize]);
                let file_name = full_path
                    .split('\\')
                    .last()
                    .unwrap_or(&full_path)
                    .to_lowercase();
                Some(file_name)
            } else {
                None
            }
        }
    }

    #[cfg(not(windows))]
    pub fn get_foreground_process_name() -> Option<String> {
        None
    }

    /// Start a background monitoring loop that invokes a callback when the foreground executable changes
    pub fn start_monitoring<F>(&self, mut on_process_changed: F)
    where
        F: FnMut(String) + Send + 'static,
    {
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        std::thread::spawn(move || {
            let mut last_proc = String::new();

            while running.load(Ordering::SeqCst) {
                if let Some(proc) = Self::get_foreground_process_name() {
                    if proc != last_proc {
                        last_proc = proc.clone();
                        on_process_changed(proc);
                    }
                }
                std::thread::sleep(Duration::from_millis(400));
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Drop for ProcessDetector {
    fn drop(&mut self) {
        self.stop();
    }
}
