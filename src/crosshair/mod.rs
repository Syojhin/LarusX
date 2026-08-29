//! Subpixel Anti-Aliased Hardware Crosshair Overlay Window (GDI+ & 32-bit ARGB UpdateLayeredWindow)

use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use crate::profiles::{CrosshairConfig, CrosshairStyle};

#[cfg(windows)]
use windows_sys::Win32::Foundation::{HWND, POINT, SIZE};
#[cfg(windows)]
use windows_sys::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HDC,
};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetSystemMetrics,
    PeekMessageW, PostQuitMessage, RegisterClassW, SetWindowPos, ShowWindow,
    TranslateMessage, CS_HREDRAW, CS_VREDRAW, HWND_TOPMOST, MSG, PM_REMOVE,
    SM_CXSCREEN, SM_CYSCREEN, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SW_HIDE,
    SW_SHOWNOACTIVATE, ULW_ALPHA, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

#[repr(C)]
struct BlendFunction {
    blend_op: u8,
    blend_flags: u8,
    source_constant_alpha: u8,
    alpha_format: u8,
}

const AC_SRC_OVER: u8 = 0x00;
const AC_SRC_ALPHA: u8 = 0x01;

#[link(name = "user32")]
extern "system" {
    fn UpdateLayeredWindow(
        hwnd: HWND,
        hdc_dst: HDC,
        ppt_dst: *const POINT,
        psize: *const SIZE,
        hdc_src: HDC,
        ppt_src: *const POINT,
        cr_key: u32,
        pblend: *const BlendFunction,
        dw_flags: u32,
    ) -> i32;
}

#[repr(C)]
struct GdiplusStartupInput {
    gdiplus_version: u32,
    debug_event_callback: *mut std::ffi::c_void,
    suppress_background_thread: i32,
    suppress_external_codecs: i32,
}

#[repr(C)]
struct PointF {
    x: f32,
    y: f32,
}

#[link(name = "gdiplus")]
extern "system" {
    fn GdiplusStartup(
        token: *mut usize,
        input: *const GdiplusStartupInput,
        output: *mut std::ffi::c_void,
    ) -> i32;
    fn GdiplusShutdown(token: usize);
    fn GdipCreateFromHDC(hdc: HDC, graphics: *mut *mut std::ffi::c_void) -> i32;
    fn GdipDeleteGraphics(graphics: *mut std::ffi::c_void) -> i32;
    fn GdipSetSmoothingMode(graphics: *mut std::ffi::c_void, mode: i32) -> i32;
    fn GdipSetPixelOffsetMode(graphics: *mut std::ffi::c_void, mode: i32) -> i32;
    fn GdipGraphicsClear(graphics: *mut std::ffi::c_void, color: u32) -> i32;
    fn GdipCreatePen1(color: u32, width: f32, unit: i32, pen: *mut *mut std::ffi::c_void) -> i32;
    fn GdipDeletePen(pen: *mut std::ffi::c_void) -> i32;
    fn GdipCreateSolidFill(color: u32, brush: *mut *mut std::ffi::c_void) -> i32;
    fn GdipDeleteBrush(brush: *mut std::ffi::c_void) -> i32;
    fn GdipDrawLine(
        graphics: *mut std::ffi::c_void,
        pen: *mut std::ffi::c_void,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    ) -> i32;
    fn GdipDrawEllipse(
        graphics: *mut std::ffi::c_void,
        pen: *mut std::ffi::c_void,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> i32;
    fn GdipFillEllipse(
        graphics: *mut std::ffi::c_void,
        brush: *mut std::ffi::c_void,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> i32;
    fn GdipDrawPolygon(
        graphics: *mut std::ffi::c_void,
        pen: *mut std::ffi::c_void,
        points: *const PointF,
        count: i32,
    ) -> i32;
}

const SMOOTHING_MODE_ANTI_ALIAS: i32 = 2;
const PIXEL_OFFSET_MODE_HIGH_QUALITY: i32 = 2;

pub enum CrosshairCommand {
    Update(CrosshairConfig),
    Hide,
    Show,
    Shutdown,
}

pub struct CrosshairController {
    sender: Option<Sender<CrosshairCommand>>,
}

impl CrosshairController {
    pub fn new() -> Self {
        #[cfg(windows)]
        {
            let (tx, rx) = channel::<CrosshairCommand>();
            std::thread::spawn(move || {
                run_crosshair_window_thread(rx);
            });
            Self { sender: Some(tx) }
        }
        #[cfg(not(windows))]
        {
            Self { sender: None }
        }
    }

    pub fn update(&self, config: CrosshairConfig) {
        if let Some(tx) = &self.sender {
            let _ = tx.send(CrosshairCommand::Update(config));
        }
    }

    pub fn toggle(&self, show: bool) {
        if let Some(tx) = &self.sender {
            if show {
                let _ = tx.send(CrosshairCommand::Show);
            } else {
                let _ = tx.send(CrosshairCommand::Hide);
            }
        }
    }
}

impl Drop for CrosshairController {
    fn drop(&mut self) {
        if let Some(tx) = &self.sender {
            let _ = tx.send(CrosshairCommand::Shutdown);
        }
    }
}

static CROSSHAIR_STATE: Mutex<Option<CrosshairConfig>> = Mutex::new(None);

fn set_global_crosshair_config(cfg: CrosshairConfig) {
    if let Ok(mut lock) = CROSSHAIR_STATE.lock() {
        *lock = Some(cfg);
    }
}

fn get_global_crosshair_config() -> CrosshairConfig {
    CROSSHAIR_STATE
        .lock()
        .ok()
        .and_then(|lock| lock.clone())
        .unwrap_or_default()
}

#[cfg(windows)]
fn run_crosshair_window_thread(rx: std::sync::mpsc::Receiver<CrosshairCommand>) {
    unsafe {
        let mut gdiplus_token: usize = 0;
        let startup_input = GdiplusStartupInput {
            gdiplus_version: 1,
            debug_event_callback: std::ptr::null_mut(),
            suppress_background_thread: 0,
            suppress_external_codecs: 0,
        };
        GdiplusStartup(&mut gdiplus_token, &startup_input, std::ptr::null_mut());

        let class_name: Vec<u16> = "LarusXSubpixelCrosshair\0".encode_utf16().collect();
        let window_name: Vec<u16> = "LarusXCrosshair\0".encode_utf16().collect();

        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(DefWindowProcW),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: std::ptr::null_mut(),
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };

        RegisterClassW(&wnd_class);

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);

        let overlay_size: i32 = 256;
        let x = (screen_w - overlay_size) / 2;
        let y = (screen_h - overlay_size) / 2;

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST
                | WS_EX_LAYERED
                | WS_EX_TRANSPARENT
                | WS_EX_TOOLWINDOW
                | WS_EX_NOACTIVATE,
            class_name.as_ptr(),
            window_name.as_ptr(),
            WS_POPUP,
            x,
            y,
            overlay_size,
            overlay_size,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null(),
        );

        if hwnd.is_null() {
            GdiplusShutdown(gdiplus_token);
            return;
        }

        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );

        let hdc_screen = GetDC(std::ptr::null_mut());
        let hdc_mem = CreateCompatibleDC(hdc_screen);

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = overlay_size;
        bmi.bmiHeader.biHeight = -overlay_size;
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let hbitmap = CreateDIBSection(
            hdc_mem,
            &bmi,
            DIB_RGB_COLORS,
            &mut bits,
            std::ptr::null_mut(),
            0,
        );
        let _old_bitmap = SelectObject(hdc_mem, hbitmap);

        let mut graphics: *mut std::ffi::c_void = std::ptr::null_mut();
        GdipCreateFromHDC(hdc_mem, &mut graphics);
        GdipSetSmoothingMode(graphics, SMOOTHING_MODE_ANTI_ALIAS);
        GdipSetPixelOffsetMode(graphics, PIXEL_OFFSET_MODE_HIGH_QUALITY);

        let render_and_update = |cfg: &CrosshairConfig| {
            if !graphics.is_null() && !bits.is_null() {
                GdipGraphicsClear(graphics, 0x00000000);

                if cfg.enabled {
                    draw_subpixel_crosshair(graphics, cfg, (overlay_size / 2) as f32, (overlay_size / 2) as f32);
                }

                let pt_src = POINT { x: 0, y: 0 };
                let pt_dst = POINT { x, y };
                let size_dst = SIZE {
                    cx: overlay_size,
                    cy: overlay_size,
                };
                let blend = BlendFunction {
                    blend_op: AC_SRC_OVER,
                    blend_flags: 0,
                    source_constant_alpha: 255,
                    alpha_format: AC_SRC_ALPHA,
                };

                UpdateLayeredWindow(
                    hwnd,
                    hdc_screen,
                    &pt_dst,
                    &size_dst,
                    hdc_mem,
                    &pt_src,
                    0,
                    &blend,
                    ULW_ALPHA,
                );
            }
        };

        let mut msg = MSG {
            hwnd: std::ptr::null_mut(),
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: POINT { x: 0, y: 0 },
        };

        let mut should_exit = false;

        while !should_exit {
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    CrosshairCommand::Update(cfg) => {
                        let enabled = cfg.enabled;
                        set_global_crosshair_config(cfg.clone());
                        render_and_update(&cfg);
                        if enabled {
                            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                        } else {
                            ShowWindow(hwnd, SW_HIDE);
                        }
                    }
                    CrosshairCommand::Show => {
                        let cfg = get_global_crosshair_config();
                        render_and_update(&cfg);
                        ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                    }
                    CrosshairCommand::Hide => {
                        ShowWindow(hwnd, SW_HIDE);
                    }
                    CrosshairCommand::Shutdown => {
                        PostQuitMessage(0);
                        should_exit = true;
                        break;
                    }
                }
            }

            while PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE) != 0 {
                if msg.message == windows_sys::Win32::UI::WindowsAndMessaging::WM_QUIT {
                    should_exit = true;
                    break;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            std::thread::sleep(std::time::Duration::from_millis(16));
        }

        if !graphics.is_null() {
            GdipDeleteGraphics(graphics);
        }
        DeleteObject(hbitmap);
        DeleteDC(hdc_mem);
        ReleaseDC(std::ptr::null_mut(), hdc_screen);
        GdiplusShutdown(gdiplus_token);
    }
}

#[cfg(windows)]
unsafe fn draw_subpixel_crosshair(
    graphics: *mut std::ffi::c_void,
    cfg: &CrosshairConfig,
    cx: f32,
    cy: f32,
) {
    let argb_main: u32 = ((cfg.color_a as u32) << 24)
        | ((cfg.color_r as u32) << 16)
        | ((cfg.color_g as u32) << 8)
        | (cfg.color_b as u32);

    let argb_outline: u32 = ((cfg.color_a as u32) << 24)
        | ((cfg.outline_color_r as u32) << 16)
        | ((cfg.outline_color_g as u32) << 8)
        | (cfg.outline_color_b as u32);

    let mut pen_main: *mut std::ffi::c_void = std::ptr::null_mut();
    GdipCreatePen1(argb_main, cfg.thickness.max(0.5), 0, &mut pen_main);

    let mut pen_outline: *mut std::ffi::c_void = std::ptr::null_mut();
    if cfg.outline {
        GdipCreatePen1(
            argb_outline,
            cfg.thickness + cfg.outline_thickness * 2.0,
            0,
            &mut pen_outline,
        );
    }

    let mut brush_main: *mut std::ffi::c_void = std::ptr::null_mut();
    GdipCreateSolidFill(argb_main, &mut brush_main);

    let mut brush_outline: *mut std::ffi::c_void = std::ptr::null_mut();
    if cfg.outline {
        GdipCreateSolidFill(argb_outline, &mut brush_outline);
    }

    let gap = cfg.gap;
    let size = cfg.size;

    match cfg.style {
        CrosshairStyle::Cross => {
            if cfg.outline && !pen_outline.is_null() {
                GdipDrawLine(graphics, pen_outline, cx, cy - gap - size, cx, cy - gap);
                GdipDrawLine(graphics, pen_outline, cx, cy + gap, cx, cy + gap + size);
                GdipDrawLine(graphics, pen_outline, cx - gap - size, cy, cx - gap, cy);
                GdipDrawLine(graphics, pen_outline, cx + gap, cy, cx + gap + size, cy);
            }
            GdipDrawLine(graphics, pen_main, cx, cy - gap - size, cx, cy - gap);
            GdipDrawLine(graphics, pen_main, cx, cy + gap, cx, cy + gap + size);
            GdipDrawLine(graphics, pen_main, cx - gap - size, cy, cx - gap, cy);
            GdipDrawLine(graphics, pen_main, cx + gap, cy, cx + gap + size, cy);
        }
        CrosshairStyle::TStyle => {
            if cfg.outline && !pen_outline.is_null() {
                GdipDrawLine(graphics, pen_outline, cx, cy + gap, cx, cy + gap + size);
                GdipDrawLine(graphics, pen_outline, cx - gap - size, cy, cx - gap, cy);
                GdipDrawLine(graphics, pen_outline, cx + gap, cy, cx + gap + size, cy);
            }
            GdipDrawLine(graphics, pen_main, cx, cy + gap, cx, cy + gap + size);
            GdipDrawLine(graphics, pen_main, cx - gap - size, cy, cx - gap, cy);
            GdipDrawLine(graphics, pen_main, cx + gap, cy, cx + gap + size, cy);
        }
        CrosshairStyle::Dot => {
            let r = cfg.dot_size.max(0.5);
            if cfg.outline && !brush_outline.is_null() {
                let or = r + cfg.outline_thickness;
                GdipFillEllipse(graphics, brush_outline, cx - or, cy - or, or * 2.0, or * 2.0);
            }
            GdipFillEllipse(graphics, brush_main, cx - r, cy - r, r * 2.0, r * 2.0);
        }
        CrosshairStyle::Circle => {
            let r = (gap + size).max(2.0);
            if cfg.outline && !pen_outline.is_null() {
                GdipDrawEllipse(graphics, pen_outline, cx - r, cy - r, r * 2.0, r * 2.0);
            }
            GdipDrawEllipse(graphics, pen_main, cx - r, cy - r, r * 2.0, r * 2.0);
        }
        CrosshairStyle::Diamond => {
            let d = gap + size;
            let pts = [
                PointF { x: cx, y: cy - d },
                PointF { x: cx + d, y: cy },
                PointF { x: cx, y: cy + d },
                PointF { x: cx - d, y: cy },
            ];
            if cfg.outline && !pen_outline.is_null() {
                GdipDrawPolygon(graphics, pen_outline, pts.as_ptr(), 4);
            }
            GdipDrawPolygon(graphics, pen_main, pts.as_ptr(), 4);
        }
        CrosshairStyle::Chevron => {
            let w = gap + size;
            let h = size * 1.2;
            if cfg.outline && !pen_outline.is_null() {
                GdipDrawLine(graphics, pen_outline, cx - w, cy + h, cx, cy);
                GdipDrawLine(graphics, pen_outline, cx, cy, cx + w, cy + h);
            }
            GdipDrawLine(graphics, pen_main, cx - w, cy + h, cx, cy);
            GdipDrawLine(graphics, pen_main, cx, cy, cx + w, cy + h);
        }
        CrosshairStyle::Box => {
            let b = gap + size;
            let pts = [
                PointF { x: cx - b, y: cy - b },
                PointF { x: cx + b, y: cy - b },
                PointF { x: cx + b, y: cy + b },
                PointF { x: cx - b, y: cy + b },
            ];
            if cfg.outline && !pen_outline.is_null() {
                GdipDrawPolygon(graphics, pen_outline, pts.as_ptr(), 4);
            }
            GdipDrawPolygon(graphics, pen_main, pts.as_ptr(), 4);
        }
    }

    if cfg.dot && cfg.style != CrosshairStyle::Dot {
        let r = cfg.dot_size.max(0.5);
        if cfg.outline && !brush_outline.is_null() {
            let or = r + cfg.outline_thickness;
            GdipFillEllipse(graphics, brush_outline, cx - or, cy - or, or * 2.0, or * 2.0);
        }
        GdipFillEllipse(graphics, brush_main, cx - r, cy - r, r * 2.0, r * 2.0);
    }

    if !pen_main.is_null() {
        GdipDeletePen(pen_main);
    }
    if !pen_outline.is_null() {
        GdipDeletePen(pen_outline);
    }
    if !brush_main.is_null() {
        GdipDeleteBrush(brush_main);
    }
    if !brush_outline.is_null() {
        GdipDeleteBrush(brush_outline);
    }
}
