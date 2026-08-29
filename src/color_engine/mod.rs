pub mod gamma;
pub mod matrix;

pub use gamma::{GammaController, GammaRamp};
pub use matrix::{build_color_matrix, DaltonismMode, MagColorEffect};

use std::sync::atomic::{AtomicBool, Ordering};

#[link(name = "Magnification")]
extern "system" {
    fn MagInitialize() -> i32;
    fn MagUninitialize() -> i32;
    fn MagSetFullscreenColorEffect(pEffect: *const MagColorEffect) -> i32;
}

pub struct DisplayTunerEngine {
    initialized: AtomicBool,
    gamma_ctrl: Option<GammaController>,
}

impl DisplayTunerEngine {
    pub fn new() -> Self {
        let initialized = unsafe { MagInitialize() != 0 };
        let gamma_ctrl = GammaController::new();
        Self {
            initialized: AtomicBool::new(initialized),
            gamma_ctrl,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    pub fn apply_effect(&self, effect: &MagColorEffect) -> bool {
        if !self.is_ready() {
            return false;
        }
        unsafe { MagSetFullscreenColorEffect(effect as *const _) != 0 }
    }

    pub fn apply_gamma(&self, gamma: f32, black_equalizer: f32) -> bool {
        if let Some(ctrl) = &self.gamma_ctrl {
            let ramp = GammaRamp::compute(gamma, black_equalizer);
            ctrl.apply(&ramp)
        } else {
            false
        }
    }

    pub fn reset_all(&self) -> bool {
        let id = MagColorEffect::identity();
        let color_res = self.apply_effect(&id);
        let gamma_res = if let Some(ctrl) = &self.gamma_ctrl {
            ctrl.reset()
        } else {
            false
        };
        color_res && gamma_res
    }
}

impl Drop for DisplayTunerEngine {
    fn drop(&mut self) {
        if self.initialized.load(Ordering::SeqCst) {
            self.reset_all();
            unsafe {
                MagUninitialize();
            }
        }
    }
}
