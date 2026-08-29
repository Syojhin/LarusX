//! 5x5 Color Transformation Matrix calculations for Windows Magnification API

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MagColorEffect {
    pub transform: [f32; 25],
}

impl MagColorEffect {
    pub fn identity() -> Self {
        let mut transform = [0.0f32; 25];
        transform[0] = 1.0;
        transform[6] = 1.0;
        transform[12] = 1.0;
        transform[18] = 1.0;
        transform[24] = 1.0;
        Self { transform }
    }

    /// Multiply two 5x5 matrices: C = A * B
    pub fn multiply(&self, other: &Self) -> Self {
        let mut result = [0.0f32; 25];
        for row in 0..5 {
            for col in 0..5 {
                let mut sum = 0.0f32;
                for k in 0..5 {
                    sum += self.transform[row * 5 + k] * other.transform[k * 5 + col];
                }
                result[row * 5 + col] = sum;
            }
        }
        Self { transform: result }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DaltonismMode {
    None,
    Protanopia,   // Red-weak: enhances red vs green contrast
    Deuteranopia, // Green-weak: boosts player silhouettes
    Tritanopia,   // Blue-weak: lifts warm tones
    PvPHighVis,   // Custom high-differentiation PvP matrix (Rust/CS2 target spotting)
}

impl DaltonismMode {
    pub fn all() -> &'static [DaltonismMode] {
        &[
            DaltonismMode::None,
            DaltonismMode::PvPHighVis,
            DaltonismMode::Protanopia,
            DaltonismMode::Deuteranopia,
            DaltonismMode::Tritanopia,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            DaltonismMode::None => "None (Standard)",
            DaltonismMode::PvPHighVis => "PvP High-Vis (Player Outline Separation)",
            DaltonismMode::Protanopia => "Protanopia (Red Enhanced)",
            DaltonismMode::Deuteranopia => "Deuteranopia (Green Silhouette)",
            DaltonismMode::Tritanopia => "Tritanopia (Warm Tones)",
        }
    }
}

/// Compute a comprehensive color matrix given user tuning parameters
pub fn build_color_matrix(
    saturation: f32,       // 0.0 to 3.0 (1.0 = 100%, 3.0 = 300%)
    digital_vibrance: f32, // 0.0 to 1.0 extra vibrance weight
    brightness: f32,       // -0.5 to 0.5 (0.0 = normal)
    contrast: f32,         // 0.5 to 2.0 (1.0 = normal)
    red_gain: f32,         // 0.5 to 2.0 (1.0 = normal)
    green_gain: f32,       // 0.5 to 2.0 (1.0 = normal)
    blue_gain: f32,        // 0.5 to 2.0 (1.0 = normal)
    daltonism: DaltonismMode,
) -> MagColorEffect {
    // 1. Saturation Matrix using Rec.709 Luminance weights
    let effective_sat = (saturation + digital_vibrance * 0.5).clamp(0.0, 3.0);
    let rw = 0.2126f32;
    let gw = 0.7152f32;
    let bw = 0.0722f32;

    let inv_s = 1.0 - effective_sat;
    let mut sat_mat = MagColorEffect::identity();

    sat_mat.transform[0] = inv_s * rw + effective_sat;
    sat_mat.transform[1] = inv_s * rw;
    sat_mat.transform[2] = inv_s * rw;

    sat_mat.transform[5] = inv_s * gw;
    sat_mat.transform[6] = inv_s * gw + effective_sat;
    sat_mat.transform[7] = inv_s * gw;

    sat_mat.transform[10] = inv_s * bw;
    sat_mat.transform[11] = inv_s * bw;
    sat_mat.transform[12] = inv_s * bw + effective_sat;

    // 2. Channel Gain Matrix (R, G, B adjustments)
    let mut gain_mat = MagColorEffect::identity();
    gain_mat.transform[0] = red_gain;
    gain_mat.transform[6] = green_gain;
    gain_mat.transform[12] = blue_gain;

    // 3. Contrast & Brightness Matrix
    let mut cb_mat = MagColorEffect::identity();
    cb_mat.transform[0] = contrast;
    cb_mat.transform[6] = contrast;
    cb_mat.transform[12] = contrast;

    let offset = (1.0 - contrast) * 0.5 + brightness;
    cb_mat.transform[20] = offset; // Red translation
    cb_mat.transform[21] = offset; // Green translation
    cb_mat.transform[22] = offset; // Blue translation

    // 4. Daltonism Matrix
    let mut dalt_mat = MagColorEffect::identity();
    match daltonism {
        DaltonismMode::None => {}
        DaltonismMode::PvPHighVis => {
            // Amplifies red/yellow spectrum against green/brown background
            dalt_mat.transform[0] = 1.25;
            dalt_mat.transform[1] = 0.05;
            dalt_mat.transform[2] = 0.00;

            dalt_mat.transform[5] = -0.10;
            dalt_mat.transform[6] = 1.15;
            dalt_mat.transform[7] = 0.05;

            dalt_mat.transform[10] = 0.00;
            dalt_mat.transform[11] = -0.05;
            dalt_mat.transform[12] = 1.20;
        }
        DaltonismMode::Protanopia => {
            dalt_mat.transform[0] = 0.56667;
            dalt_mat.transform[1] = 0.43333;
            dalt_mat.transform[2] = 0.00000;

            dalt_mat.transform[5] = 0.55833;
            dalt_mat.transform[6] = 0.44167;
            dalt_mat.transform[7] = 0.00000;

            dalt_mat.transform[10] = 0.00000;
            dalt_mat.transform[11] = 0.24167;
            dalt_mat.transform[12] = 0.75833;
        }
        DaltonismMode::Deuteranopia => {
            dalt_mat.transform[0] = 0.625;
            dalt_mat.transform[1] = 0.375;
            dalt_mat.transform[2] = 0.000;

            dalt_mat.transform[5] = 0.700;
            dalt_mat.transform[6] = 0.300;
            dalt_mat.transform[7] = 0.000;

            dalt_mat.transform[10] = 0.000;
            dalt_mat.transform[11] = 0.300;
            dalt_mat.transform[12] = 0.700;
        }
        DaltonismMode::Tritanopia => {
            dalt_mat.transform[0] = 0.950;
            dalt_mat.transform[1] = 0.050;
            dalt_mat.transform[2] = 0.000;

            dalt_mat.transform[5] = 0.000;
            dalt_mat.transform[6] = 0.43333;
            dalt_mat.transform[7] = 0.56667;

            dalt_mat.transform[10] = 0.000;
            dalt_mat.transform[11] = 0.475;
            dalt_mat.transform[12] = 0.525;
        }
    }

    // Combine matrices in sequence: Saturation -> Gain -> Daltonism -> Contrast/Brightness
    sat_mat.multiply(&gain_mat).multiply(&dalt_mat).multiply(&cb_mat)
}
