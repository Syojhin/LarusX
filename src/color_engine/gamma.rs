//! Hardware Gamma & Black Equalizer LUT via Win32 SetDeviceGammaRamp

use std::ffi::c_void;
use windows_sys::Win32::Graphics::Gdi::{GetDC, ReleaseDC, HDC};

#[link(name = "gdi32")]
extern "system" {
    fn SetDeviceGammaRamp(hdc: HDC, lpRamp: *const c_void) -> i32;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GammaRamp {
    pub red: [u16; 256],
    pub green: [u16; 256],
    pub blue: [u16; 256],
}

impl GammaRamp {
    pub fn linear() -> Self {
        let mut ramp = GammaRamp {
            red: [0; 256],
            green: [0; 256],
            blue: [0; 256],
        };
        for i in 0..256 {
            let val = ((i as u32 * 65535) / 255) as u16;
            ramp.red[i] = val;
            ramp.green[i] = val;
            ramp.blue[i] = val;
        }
        ramp
    }

    /// Generates a gamma ramp with Black Equalizer (selective dark-tone expansion).
    /// * `gamma`: 0.5 (bright) to 2.5 (dark). Standard is 1.0.
    /// * `black_equalizer`: 0.0 (off) to 1.0 (maximum shadow boost without highlight clipping).
    pub fn compute(gamma: f32, black_equalizer: f32) -> Self {
        let mut ramp = GammaRamp {
            red: [0; 256],
            green: [0; 256],
            blue: [0; 256],
        };

        let safe_gamma = gamma.clamp(0.4, 3.0);
        let inv_gamma = 1.0 / safe_gamma;
        let be = black_equalizer.clamp(0.0, 1.0);

        for i in 0..256 {
            let normalized = i as f32 / 255.0;

            // 1. Standard gamma curve
            let mut val = normalized.powf(inv_gamma);

            // 2. Black Equalizer: Smooth shadow lift
            // Boost values in the 0.0 - 0.5 range while tapering off smoothly towards 1.0
            if be > 0.001 {
                let shadow_weight = (1.0 - normalized).powf(2.0); // Stronger at shadows, zero at whites
                let boost = be * 0.45 * shadow_weight * (normalized + 0.1).sqrt();
                val = (val + boost).clamp(0.0, 1.0);
            }

            let u16_val = (val * 65535.0).clamp(0.0, 65535.0) as u16;
            ramp.red[i] = u16_val;
            ramp.green[i] = u16_val;
            ramp.blue[i] = u16_val;
        }

        ramp
    }
}

pub struct GammaController {
    hdc: HDC,
}

impl GammaController {
    pub fn new() -> Option<Self> {
        unsafe {
            let hdc = GetDC(std::ptr::null_mut());
            if !hdc.is_null() {
                Some(Self { hdc })
            } else {
                None
            }
        }
    }

    pub fn apply(&self, ramp: &GammaRamp) -> bool {
        unsafe {
            if !self.hdc.is_null() {
                let res = SetDeviceGammaRamp(self.hdc, ramp as *const _ as *const c_void);
                res != 0
            } else {
                false
            }
        }
    }

    pub fn reset(&self) -> bool {
        let linear = GammaRamp::linear();
        self.apply(&linear)
    }
}

impl Drop for GammaController {
    fn drop(&mut self) {
        unsafe {
            if !self.hdc.is_null() {
                self.reset();
                ReleaseDC(std::ptr::null_mut(), self.hdc);
                self.hdc = std::ptr::null_mut();
            }
        }
    }
}
