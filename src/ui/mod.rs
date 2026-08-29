//! Modern Dark UI for LarusX Display & Visibility Tuner with Direct Numeric Inputs & Pro Subpixel Crosshairs

use eframe::egui::{self, Color32, DragValue, Pos2, Rounding, Slider, Stroke, Vec2};
use std::sync::{Arc, Mutex};
use crate::color_engine::{build_color_matrix, DaltonismMode, DisplayTunerEngine};
use crate::crosshair::CrosshairController;
use crate::profiles::{CrosshairConfig, CrosshairStyle, DisplayProfile};

#[derive(PartialEq)]
pub enum ActiveTab {
    ColorGrading,
    BlackEqualizer,
    Crosshair,
    Profiles,
}

pub struct LarusXApp {
    pub engine: Arc<DisplayTunerEngine>,
    pub crosshair_ctrl: Arc<CrosshairController>,
    pub active_tab: ActiveTab,
    pub master_enabled: bool,
    pub auto_detect_enabled: bool,
    pub active_profile: DisplayProfile,
    pub presets: Vec<DisplayProfile>,
    pub current_foreground: Arc<Mutex<String>>,
    pub share_code_input: String,
    pub status_message: String,
}

impl LarusXApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        engine: Arc<DisplayTunerEngine>,
        crosshair_ctrl: Arc<CrosshairController>,
        current_foreground: Arc<Mutex<String>>,
    ) -> Self {
        let presets = vec![
            DisplayProfile::neutral(),
            DisplayProfile::valorant(),
            DisplayProfile::overwatch2(),
            DisplayProfile::marvel_rivals(),
        ];

        let active_profile = presets[1].clone(); // Default to Valorant

        let app = Self {
            engine,
            crosshair_ctrl,
            active_tab: ActiveTab::Crosshair, // Focus directly on crosshair for David
            master_enabled: true,
            auto_detect_enabled: true,
            active_profile,
            presets,
            current_foreground,
            share_code_input: String::new(),
            status_message: "LarusX Subpixel Crosshair Engine Active (0 FPS impact)".into(),
        };

        app.apply_current_profile();
        app
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
    }

    pub fn apply_current_profile(&self) {
        if self.master_enabled {
            let mat = build_color_matrix(
                self.active_profile.saturation,
                self.active_profile.digital_vibrance,
                self.active_profile.brightness,
                self.active_profile.contrast,
                self.active_profile.red_gain,
                self.active_profile.green_gain,
                self.active_profile.blue_gain,
                self.active_profile.daltonism,
            );
            self.engine.apply_effect(&mat);
            self.engine.apply_gamma(
                self.active_profile.gamma,
                self.active_profile.black_equalizer,
            );
            self.crosshair_ctrl.update(self.active_profile.crosshair.clone());
        } else {
            self.engine.reset_all();
            self.crosshair_ctrl.toggle(false);
        }
    }
}

impl eframe::App for LarusXApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(16, 18, 22);
        visuals.window_fill = Color32::from_rgb(20, 23, 28);
        visuals.override_text_color = Some(Color32::from_rgb(235, 240, 250));
        ctx.set_visuals(visuals);

        let mut changed = false;

        if self.auto_detect_enabled {
            let active_fg = self.current_foreground.try_lock().ok().map(|g| g.clone());
            if let Some(fg) = active_fg {
                if !fg.is_empty() {
                    let mut switch_to: Option<DisplayProfile> = None;
                    for preset in &self.presets {
                        if preset.matches_process(&fg) && self.active_profile.id != preset.id {
                            switch_to = Some(preset.clone());
                            break;
                        }
                    }
                    if let Some(target_preset) = switch_to {
                        let name = target_preset.name.clone();
                        self.active_profile = target_preset;
                        self.apply_current_profile();
                        self.set_status(format!("Auto-switched to {} for {}", name, fg));
                    }
                }
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::F7)) {
            self.master_enabled = !self.master_enabled;
            changed = true;
            let status = if self.master_enabled { "Enabled" } else { "Disabled" };
            self.set_status(format!("Master Engine: {}", status));
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F8)) {
            self.active_profile.crosshair.enabled = !self.active_profile.crosshair.enabled;
            changed = true;
            let status = if self.active_profile.crosshair.enabled { "ON" } else { "OFF" };
            self.set_status(format!("Crosshair Overlay: {}", status));
        }

        egui::TopBottomPanel::top("top_header").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("LARUS").strong().color(Color32::from_rgb(0, 255, 170)));
                ui.heading(egui::RichText::new("X").strong().color(Color32::from_rgb(255, 255, 255)));
                ui.label(egui::RichText::new("DISPLAY & VISIBILITY").small().color(Color32::from_rgb(120, 130, 150)));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(if self.master_enabled {
                        egui::RichText::new("● ACTIVE").color(Color32::from_rgb(0, 255, 150)).strong()
                    } else {
                        egui::RichText::new("○ BYPASS").color(Color32::from_rgb(150, 150, 150))
                    })).clicked() {
                        self.master_enabled = !self.master_enabled;
                        changed = true;
                    }

                    if ui.checkbox(&mut self.auto_detect_enabled, "Auto-Detect Game").changed() {
                        changed = true;
                    }
                });
            });
            ui.add_space(4.0);

            // Tab bar
            ui.horizontal(|ui| {
                let tabs = [
                    (ActiveTab::ColorGrading, "🎨 Color & Vibrance"),
                    (ActiveTab::BlackEqualizer, "🌙 Shadow & Gamma"),
                    (ActiveTab::Crosshair, "🎯 Subpixel Crosshair"),
                    (ActiveTab::Profiles, "💾 Presets & Codes"),
                ];

                for (tab, label) in tabs {
                    let is_active = self.active_tab == tab;
                    let text = if is_active {
                        egui::RichText::new(label).strong().color(Color32::from_rgb(0, 255, 170))
                    } else {
                        egui::RichText::new(label).color(Color32::from_rgb(170, 180, 200))
                    };

                    if ui.selectable_label(is_active, text).clicked() {
                        self.active_tab = tab;
                    }
                }
            });
            ui.add_space(4.0);
        });

        // Bottom Status Bar with Syojhin Signature
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("✦ Made by Syojhin").small().strong().color(Color32::from_rgb(0, 255, 180)));
                ui.label(egui::RichText::new("|").color(Color32::from_rgb(60, 70, 85)));
                ui.label(egui::RichText::new(&self.status_message).small().color(Color32::from_rgb(160, 210, 255)));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let fg_name = self.current_foreground.lock().unwrap().clone();
                    if !fg_name.is_empty() {
                        ui.label(egui::RichText::new(format!("Focused: {}", fg_name)).small().color(Color32::from_rgb(140, 160, 190)));
                    }
                    ui.label(egui::RichText::new("[F7] Toggle | [F8] Crosshair").small().color(Color32::from_rgb(100, 110, 130)));
                });
            });
        });

        // Central Content Area
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.active_tab {
                    ActiveTab::ColorGrading => {
                        changed |= self.render_color_grading_tab(ui);
                    }
                    ActiveTab::BlackEqualizer => {
                        changed |= self.render_black_equalizer_tab(ui);
                    }
                    ActiveTab::Crosshair => {
                        changed |= self.render_crosshair_tab(ui);
                    }
                    ActiveTab::Profiles => {
                        changed |= self.render_profiles_tab(ui);
                    }
                }
            });
        });

        if changed {
            self.apply_current_profile();
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(33));
    }
}

impl LarusXApp {
    fn render_color_grading_tab(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.add_space(6.0);
        ui.group(|ui| {
            ui.heading("Primary Color Controls");
            ui.label(egui::RichText::new("Drag sliders or click directly on any number to type exact values.").small().color(Color32::from_rgb(140, 150, 170)));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Saturation (Digital Vibrance):");
                let mut sat_pct = (self.active_profile.saturation * 100.0).round() as i32;
                if ui.add(DragValue::new(&mut sat_pct).speed(1).range(0..=300).suffix("%")).changed() {
                    self.active_profile.saturation = sat_pct as f32 / 100.0;
                    changed = true;
                }
            });
            if ui.add(Slider::new(&mut self.active_profile.saturation, 0.0..=3.0).show_value(false).step_by(0.01)).changed() {
                changed = true;
            }

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Digital Vibrance Boost:");
                let mut vib_pct = (self.active_profile.digital_vibrance * 100.0).round() as i32;
                if ui.add(DragValue::new(&mut vib_pct).speed(1).range(0..=100).prefix("+").suffix("%")).changed() {
                    self.active_profile.digital_vibrance = vib_pct as f32 / 100.0;
                    changed = true;
                }
            });
            if ui.add(Slider::new(&mut self.active_profile.digital_vibrance, 0.0..=1.0).show_value(false).step_by(0.01)).changed() {
                changed = true;
            }

            ui.add_space(6.0);

            ui.columns(2, |cols| {
                cols[0].horizontal(|h| {
                    h.label("Contrast:");
                    let mut c_pct = (self.active_profile.contrast * 100.0).round() as i32;
                    if h.add(DragValue::new(&mut c_pct).speed(1).range(50..=200).suffix("%")).changed() {
                        self.active_profile.contrast = c_pct as f32 / 100.0;
                        changed = true;
                    }
                });
                if cols[0].add(Slider::new(&mut self.active_profile.contrast, 0.5..=2.0).show_value(false).step_by(0.01)).changed() {
                    changed = true;
                }

                cols[1].horizontal(|h| {
                    h.label("Brightness Offset:");
                    let mut b_pct = (self.active_profile.brightness * 100.0).round() as i32;
                    if h.add(DragValue::new(&mut b_pct).speed(1).range(-50..=50).suffix("%")).changed() {
                        self.active_profile.brightness = b_pct as f32 / 100.0;
                        changed = true;
                    }
                });
                if cols[1].add(Slider::new(&mut self.active_profile.brightness, -0.5..=0.5).show_value(false).step_by(0.01)).changed() {
                    changed = true;
                }
            });
        });

        ui.add_space(6.0);

        ui.group(|ui| {
            ui.heading("Daltonisation & Player Outline Separation");
            ui.label(egui::RichText::new("Isolates player silhouettes from background map textures.").small().color(Color32::from_rgb(140, 150, 170)));
            ui.add_space(4.0);

            for mode in DaltonismMode::all() {
                let selected = self.active_profile.daltonism == *mode;
                if ui.radio(selected, mode.name()).clicked() {
                    self.active_profile.daltonism = *mode;
                    changed = true;
                }
            }
        });

        ui.add_space(6.0);

        ui.group(|ui| {
            ui.heading("RGB Channel Tuning");
            ui.label(egui::RichText::new("Click any box below to type exact multiplier (e.g. 0.95, 1.20, 1.10)").small().color(Color32::from_rgb(140, 150, 170)));
            ui.add_space(4.0);

            ui.columns(3, |cols| {
                cols[0].horizontal(|h| {
                    h.label("Red:");
                    if h.add(DragValue::new(&mut self.active_profile.red_gain).speed(0.01).range(0.4..=2.0)).changed() {
                        changed = true;
                    }
                });
                if cols[0].add(Slider::new(&mut self.active_profile.red_gain, 0.4..=2.0).show_value(false).step_by(0.01)).changed() {
                    changed = true;
                }

                cols[1].horizontal(|h| {
                    h.label("Green:");
                    if h.add(DragValue::new(&mut self.active_profile.green_gain).speed(0.01).range(0.4..=2.0)).changed() {
                        changed = true;
                    }
                });
                if cols[1].add(Slider::new(&mut self.active_profile.green_gain, 0.4..=2.0).show_value(false).step_by(0.01)).changed() {
                    changed = true;
                }

                cols[2].horizontal(|h| {
                    h.label("Blue:");
                    if h.add(DragValue::new(&mut self.active_profile.blue_gain).speed(0.01).range(0.4..=2.0)).changed() {
                        changed = true;
                    }
                });
                if cols[2].add(Slider::new(&mut self.active_profile.blue_gain, 0.4..=2.0).show_value(false).step_by(0.01)).changed() {
                    changed = true;
                }
            });
        });

        changed
    }

    fn render_black_equalizer_tab(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.add_space(6.0);
        ui.group(|ui| {
            ui.heading("Black Equalizer (Night Vision & Shadow Clarifier)");
            ui.label("Lifts dark shadow details and uncovers enemies hiding in pitch black corners without over-exposing the sky.");
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Shadow Boost Intensity:");
                let mut be_pct = (self.active_profile.black_equalizer * 100.0).round() as i32;
                if ui.add(DragValue::new(&mut be_pct).speed(1).range(0..=100).suffix("%")).changed() {
                    self.active_profile.black_equalizer = be_pct as f32 / 100.0;
                    changed = true;
                }
            });
            if ui.add(Slider::new(&mut self.active_profile.black_equalizer, 0.0..=1.0).show_value(false).step_by(0.01)).changed() {
                changed = true;
            }

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Overall Hardware Gamma:");
                if ui.add(DragValue::new(&mut self.active_profile.gamma).speed(0.01).range(0.4..=2.5)).changed() {
                    changed = true;
                }
            });
            if ui.add(Slider::new(&mut self.active_profile.gamma, 0.4..=2.5).show_value(false).step_by(0.01)).changed() {
                changed = true;
            }

            ui.add_space(10.0);

            ui.label(egui::RichText::new("Transfer Function Curve:").small().color(Color32::from_rgb(140, 150, 170)));
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 90.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, Rounding::same(6.0), Color32::from_rgb(12, 14, 18));
            ui.painter().rect_stroke(rect, Rounding::same(6.0), Stroke::new(1.0_f32, Color32::from_rgb(40, 45, 55)));

            ui.painter().line_segment(
                [Pos2::new(rect.left(), rect.bottom()), Pos2::new(rect.right(), rect.top())],
                Stroke::new(1.0_f32, Color32::from_rgb(60, 65, 75)),
            );

            let points: Vec<Pos2> = (0..=50)
                .map(|i| {
                    let norm = i as f32 / 50.0;
                    let gamma_val = norm.powf(1.0 / self.active_profile.gamma);
                    let shadow_boost = if self.active_profile.black_equalizer > 0.001 {
                        self.active_profile.black_equalizer * 0.45 * (1.0 - norm).powf(2.0) * (norm + 0.1).sqrt()
                    } else {
                        0.0
                    };
                    let final_val = (gamma_val + shadow_boost).clamp(0.0, 1.0);

                    let x = rect.left() + norm * rect.width();
                    let y = rect.bottom() - final_val * rect.height();
                    Pos2::new(x, y)
                })
                .collect();

            for i in 0..points.len() - 1 {
                ui.painter().line_segment([points[i], points[i + 1]], Stroke::new(2.0_f32, Color32::from_rgb(0, 255, 180)));
            }
        });

        changed
    }

    fn render_crosshair_tab(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.add_space(6.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Subpixel Anti-Aliased Crosshair HUD");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.checkbox(&mut self.active_profile.crosshair.enabled, "Enable Overlay").changed() {
                        changed = true;
                    }
                });
            });
            ui.label(egui::RichText::new("32-Bit ARGB GDI+ Subpixel Rendered • Buttery Smooth Curved Edges • Zero-Lag Topmost").small().color(Color32::from_rgb(0, 255, 180)));

            ui.add_space(6.0);

            // Pro 1-Click Presets
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Pro Presets:").strong().color(Color32::from_rgb(180, 220, 255)));

                if ui.small_button("🎯 TenZ Micro-Dot").clicked() {
                    self.active_profile.crosshair = CrosshairConfig {
                        enabled: true,
                        style: CrosshairStyle::Dot,
                        size: 2.0,
                        thickness: 1.5,
                        gap: 0.0,
                        dot: true,
                        dot_size: 1.8,
                        color_r: 0,
                        color_g: 255,
                        color_b: 255,
                        color_a: 255,
                        outline: true,
                        outline_thickness: 1.0,
                        outline_color_r: 0,
                        outline_color_g: 0,
                        outline_color_b: 0,
                    };
                    changed = true;
                }

                if ui.small_button("□ Demon1 Box").clicked() {
                    self.active_profile.crosshair = CrosshairConfig {
                        enabled: true,
                        style: CrosshairStyle::Box,
                        size: 3.5,
                        thickness: 1.5,
                        gap: 1.0,
                        dot: false,
                        dot_size: 1.0,
                        color_r: 0,
                        color_g: 255,
                        color_b: 128,
                        color_a: 255,
                        outline: true,
                        outline_thickness: 1.0,
                        outline_color_r: 0,
                        outline_color_g: 0,
                        outline_color_b: 0,
                    };
                    changed = true;
                }

                if ui.small_button("+ ScreaM One-Tap").clicked() {
                    self.active_profile.crosshair = CrosshairConfig {
                        enabled: true,
                        style: CrosshairStyle::Cross,
                        size: 3.5,
                        thickness: 1.5,
                        gap: 1.5,
                        dot: false,
                        dot_size: 1.0,
                        color_r: 0,
                        color_g: 255,
                        color_b: 255,
                        color_a: 255,
                        outline: true,
                        outline_thickness: 1.0,
                        outline_color_r: 0,
                        outline_color_g: 0,
                        outline_color_b: 0,
                    };
                    changed = true;
                }

                if ui.small_button("○ Radiant Ring").clicked() {
                    self.active_profile.crosshair = CrosshairConfig {
                        enabled: true,
                        style: CrosshairStyle::Circle,
                        size: 4.5,
                        thickness: 1.8,
                        gap: 2.0,
                        dot: true,
                        dot_size: 1.5,
                        color_r: 255,
                        color_g: 40,
                        color_b: 120,
                        color_a: 255,
                        outline: true,
                        outline_thickness: 1.0,
                        outline_color_r: 0,
                        outline_color_g: 0,
                        outline_color_b: 0,
                    };
                    changed = true;
                }

                if ui.small_button("^ Chevron Apex").clicked() {
                    self.active_profile.crosshair = CrosshairConfig {
                        enabled: true,
                        style: CrosshairStyle::Chevron,
                        size: 4.0,
                        thickness: 1.8,
                        gap: 1.0,
                        dot: false,
                        dot_size: 1.0,
                        color_r: 255,
                        color_g: 255,
                        color_b: 0,
                        color_a: 255,
                        outline: true,
                        outline_thickness: 1.0,
                        outline_color_r: 0,
                        outline_color_g: 0,
                        outline_color_b: 0,
                    };
                    changed = true;
                }
            });

            ui.add_space(6.0);

            // Style selection
            ui.horizontal_wrapped(|ui| {
                ui.label("Style:");
                for style in CrosshairStyle::all() {
                    let selected = self.active_profile.crosshair.style == *style;
                    if ui.selectable_label(selected, style.name()).clicked() {
                        self.active_profile.crosshair.style = *style;
                        changed = true;
                    }
                }
            });

            ui.add_space(6.0);

            let crosshair_clone = self.active_profile.crosshair.clone();

            // Preview Box & Sliders
            ui.columns(2, |cols| {
                cols[0].group(|col_ui| {
                    col_ui.label("Sight Preview (Subpixel Canvas)");
                    let avail_w = col_ui.available_width();
                    let (rect, _) = col_ui.allocate_exact_size(Vec2::new(avail_w, 140.0), egui::Sense::hover());
                    col_ui.painter().rect_filled(rect, Rounding::same(6.0), Color32::from_rgb(22, 26, 32));
                    col_ui.painter().rect_stroke(rect, Rounding::same(6.0), Stroke::new(1.0_f32, Color32::from_rgb(50, 55, 65)));

                    let center = rect.center();
                    let cross_color = Color32::from_rgba_premultiplied(
                        crosshair_clone.color_r,
                        crosshair_clone.color_g,
                        crosshair_clone.color_b,
                        crosshair_clone.color_a,
                    );
                    let outline_color = Color32::from_rgb(
                        crosshair_clone.outline_color_r,
                        crosshair_clone.outline_color_g,
                        crosshair_clone.outline_color_b,
                    );

                    if crosshair_clone.outline {
                        let outline_stroke = Stroke::new(crosshair_clone.thickness + crosshair_clone.outline_thickness * 2.0, outline_color);
                        draw_crosshair_geometry(col_ui, center, &crosshair_clone, outline_stroke);
                    }
                    let main_stroke = Stroke::new(crosshair_clone.thickness, cross_color);
                    draw_crosshair_geometry(col_ui, center, &crosshair_clone, main_stroke);
                });

                cols[1].group(|col_ui| {
                    col_ui.label("Subpixel Controls");
                    col_ui.horizontal(|h_ui| {
                        h_ui.label("Size / Radius:");
                        if h_ui.add(DragValue::new(&mut self.active_profile.crosshair.size).speed(0.1).range(0.5..=30.0)).changed() {
                            changed = true;
                        }
                    });
                    col_ui.horizontal(|h_ui| {
                        h_ui.label("Center Gap:");
                        if h_ui.add(DragValue::new(&mut self.active_profile.crosshair.gap).speed(0.1).range(0.0..=20.0)).changed() {
                            changed = true;
                        }
                    });
                    col_ui.horizontal(|h_ui| {
                        h_ui.label("Line Thickness:");
                        if h_ui.add(DragValue::new(&mut self.active_profile.crosshair.thickness).speed(0.1).range(0.5..=8.0)).changed() {
                            changed = true;
                        }
                    });
                    col_ui.horizontal(|h_ui| {
                        h_ui.label("Dot Size:");
                        if h_ui.add(DragValue::new(&mut self.active_profile.crosshair.dot_size).speed(0.1).range(0.5..=10.0)).changed() {
                            changed = true;
                        }
                    });
                    col_ui.horizontal(|h_ui| {
                        if h_ui.checkbox(&mut self.active_profile.crosshair.dot, "Center Dot").changed() {
                            changed = true;
                        }
                        if h_ui.checkbox(&mut self.active_profile.crosshair.outline, "Anti-Aliased Border").changed() {
                            changed = true;
                        }
                    });
                });
            });

            ui.add_space(6.0);

            // Color selection & Opacity
            ui.horizontal(|ui| {
                ui.label("Sight Color:");
                let mut color = [
                    self.active_profile.crosshair.color_r,
                    self.active_profile.crosshair.color_g,
                    self.active_profile.crosshair.color_b,
                ];
                if ui.color_edit_button_srgb(&mut color).changed() {
                    self.active_profile.crosshair.color_r = color[0];
                    self.active_profile.crosshair.color_g = color[1];
                    self.active_profile.crosshair.color_b = color[2];
                    changed = true;
                }

                ui.label("Opacity (Alpha):");
                if ui.add(DragValue::new(&mut self.active_profile.crosshair.color_a).speed(1).range(0..=255)).changed() {
                    changed = true;
                }

                ui.add_space(10.0);

                let color_presets = [
                    ("Cyan", 0, 255, 255),
                    ("Neon Green", 0, 255, 128),
                    ("Magenta", 255, 0, 180),
                    ("Yellow", 255, 255, 0),
                    ("Pure Red", 255, 30, 30),
                    ("White", 255, 255, 255),
                ];
                for (name, r, g, b) in color_presets {
                    if ui.small_button(name).clicked() {
                        self.active_profile.crosshair.color_r = r;
                        self.active_profile.crosshair.color_g = g;
                        self.active_profile.crosshair.color_b = b;
                        changed = true;
                    }
                }
            });
        });

        changed
    }

    fn render_profiles_tab(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.add_space(6.0);
        ui.group(|ui| {
            ui.heading("Core Game Presets");
            ui.add_space(4.0);

            let mut selected_idx: Option<usize> = None;
            for (idx, preset) in self.presets.iter().enumerate() {
                let is_active = self.active_profile.id == preset.id;
                ui.horizontal(|ui| {
                    let label = if is_active {
                        egui::RichText::new(format!("▶ {}", preset.name)).strong().color(Color32::from_rgb(0, 255, 170))
                    } else {
                        egui::RichText::new(&preset.name).color(Color32::from_rgb(200, 210, 230))
                    };

                    if ui.button(label).clicked() {
                        selected_idx = Some(idx);
                    }

                    if !preset.process_target.is_empty() {
                        ui.label(egui::RichText::new(format!("({})", preset.process_target)).small().color(Color32::from_rgb(110, 120, 140)));
                    }
                });
            }

            if let Some(idx) = selected_idx {
                self.active_profile = self.presets[idx].clone();
                let name = self.active_profile.name.clone();
                changed = true;
                self.set_status(format!("Loaded profile: {}", name));
            }
        });

        ui.add_space(6.0);

        ui.group(|ui| {
            ui.heading("Current Profile Settings & Auto-Hook Target");
            ui.horizontal(|ui| {
                ui.label("Profile Name:");
                ui.text_edit_singleline(&mut self.active_profile.name);
            });
            ui.horizontal(|ui| {
                ui.label("Target Executable(s):");
                ui.text_edit_singleline(&mut self.active_profile.process_target);
                ui.label(egui::RichText::new("(comma separated)").small().color(Color32::from_rgb(120, 130, 150)));
            });

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.button("Save as New Custom Preset").clicked() {
                    let mut new_p = self.active_profile.clone();
                    new_p.id = format!("custom_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
                    self.presets.push(new_p);
                    self.set_status("Custom preset saved!");
                }

                if ui.button("Reset to Neutral Desktop").clicked() {
                    self.active_profile = DisplayProfile::neutral();
                    changed = true;
                    self.set_status("Reset to neutral calibrated desktop.");
                }
            });
        });

        ui.add_space(6.0);

        ui.group(|ui| {
            ui.heading("Shareable Profile Codes (Community Sharing)");
            ui.label("Import or export profiles using single-line share strings.");

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui.button("📋 Copy Current Share Code").clicked() {
                    if let Ok(code) = self.active_profile.to_share_code() {
                        ui.output_mut(|o| o.copied_text = code);
                        self.set_status("Share code copied to clipboard!");
                    }
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.share_code_input);
                if ui.button("Import Code").clicked() {
                    match DisplayProfile::from_share_code(&self.share_code_input) {
                        Ok(p) => {
                            self.active_profile = p;
                            changed = true;
                            self.set_status("Profile imported successfully!");
                        }
                        Err(e) => {
                            self.set_status(format!("Import error: {}", e));
                        }
                    }
                }
            });
        });

        changed
    }
}

fn draw_crosshair_geometry(ui: &egui::Ui, center: Pos2, cfg: &CrosshairConfig, stroke: Stroke) {
    let gap = cfg.gap;
    let size = cfg.size;

    match cfg.style {
        CrosshairStyle::Cross => {
            ui.painter().line_segment([Pos2::new(center.x, center.y - gap - size), Pos2::new(center.x, center.y - gap)], stroke);
            ui.painter().line_segment([Pos2::new(center.x, center.y + gap), Pos2::new(center.x, center.y + gap + size)], stroke);
            ui.painter().line_segment([Pos2::new(center.x - gap - size, center.y), Pos2::new(center.x - gap, center.y)], stroke);
            ui.painter().line_segment([Pos2::new(center.x + gap, center.y), Pos2::new(center.x + gap + size, center.y)], stroke);
        }
        CrosshairStyle::TStyle => {
            ui.painter().line_segment([Pos2::new(center.x, center.y + gap), Pos2::new(center.x, center.y + gap + size)], stroke);
            ui.painter().line_segment([Pos2::new(center.x - gap - size, center.y), Pos2::new(center.x - gap, center.y)], stroke);
            ui.painter().line_segment([Pos2::new(center.x + gap, center.y), Pos2::new(center.x + gap + size, center.y)], stroke);
        }
        CrosshairStyle::Dot => {
            ui.painter().circle_filled(center, cfg.dot_size.max(1.0), stroke.color);
        }
        CrosshairStyle::Circle => {
            ui.painter().circle_stroke(center, (gap + size).max(2.0), stroke);
        }
        CrosshairStyle::Diamond => {
            let d = gap + size;
            let pts = [
                Pos2::new(center.x, center.y - d),
                Pos2::new(center.x + d, center.y),
                Pos2::new(center.x, center.y + d),
                Pos2::new(center.x - d, center.y),
                Pos2::new(center.x, center.y - d),
            ];
            for i in 0..4 {
                ui.painter().line_segment([pts[i], pts[i + 1]], stroke);
            }
        }
        CrosshairStyle::Chevron => {
            let w = gap + size;
            let h = size * 1.2;
            ui.painter().line_segment([Pos2::new(center.x - w, center.y + h), center], stroke);
            ui.painter().line_segment([center, Pos2::new(center.x + w, center.y + h)], stroke);
        }
        CrosshairStyle::Box => {
            let b = gap + size;
            let rect = egui::Rect::from_center_size(center, Vec2::new(b * 2.0, b * 2.0));
            ui.painter().rect_stroke(rect, Rounding::ZERO, stroke);
        }
    }

    if cfg.dot && cfg.style != CrosshairStyle::Dot {
        ui.painter().circle_filled(center, cfg.dot_size.max(1.0), stroke.color);
    }
}
