use eframe::egui;
use egui::{Color32, Pos2, Shape, Stroke};
use std::f32::consts::PI;

const MAX_STAT_VAL: f32 = 100.0;

// Angles (radians)
// Red (Strength) Top Left, Green (Dex) Top Right, Blue (Int) Bottom
const ANG_STR_RED: f32 = 135.0 * (PI / 180.0);
const ANG_DEX_GREEN: f32 = 45.0 * (PI / 180.0);
const ANG_INT_BLUE: f32 = 270.0 * (PI / 180.0);

struct StatApp {
    strength: f32,
    intelligence: f32,
    dexterity: f32,
}

impl Default for StatApp {
    fn default() -> Self {
        Self {
            strength: 50.0,
            intelligence: 50.0,
            dexterity: 50.0,
        }
    }
}

impl StatApp {
    // FIX: Removed `&mut self`. This is now an "associated function" (static helper).
    // It only touches the arguments we explicitly pass to it.
    fn stat_control_ui(ui: &mut egui::Ui, label: &str, value: &mut f32, color: Color32) {
        ui.horizontal(|ui| {
            ui.colored_label(color, format!("{}: {:>3.0}", label, value));
            ui.add_space(10.0);
            if ui.button("-").clicked() {
                *value = (*value - 5.0).clamp(0.0, MAX_STAT_VAL);
            }
            if ui.button("+").clicked() {
                *value = (*value + 5.0).clamp(0.0, MAX_STAT_VAL);
            }
        });
    }
}

impl eframe::App for StatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("controls_panel").show(ctx, |ui| {
            ui.add_space(20.0);

            // FIX: We call the function using `Self::` instead of `self.`
            // This allows us to pass `&mut self.strength` without locking the rest of `self`.
            Self::stat_control_ui(
                ui,
                "STR",
                &mut self.strength,
                Color32::from_rgb(200, 50, 50),
            );
            Self::stat_control_ui(
                ui,
                "INT",
                &mut self.intelligence,
                Color32::from_rgb(50, 50, 200),
            );
            Self::stat_control_ui(
                ui,
                "DEX",
                &mut self.dexterity,
                Color32::from_rgb(50, 200, 50),
            );

            ui.add_space(30.0);
            if ui.button("Reset").clicked() {
                *self = Self::default();
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::hover());

            let center = response.rect.center();
            // Calculate max radius (responsive to window size)
            let max_radius = response.rect.width().min(response.rect.height()) / 2.0 * 0.8;

            // 1. Draw Background Circle
            painter.circle_stroke(center, max_radius, Stroke::new(1.0, Color32::from_gray(80)));

            // 2. Draw Axes
            let axes = [
                (ANG_STR_RED, Color32::RED),
                (ANG_DEX_GREEN, Color32::GREEN),
                (ANG_INT_BLUE, Color32::BLUE),
            ];

            for &(angle, color) in &axes {
                let tip = Pos2::new(
                    center.x + max_radius * angle.cos(),
                    center.y - max_radius * angle.sin(),
                );
                painter.line_segment([center, tip], Stroke::new(1.5, color.gamma_multiply(0.5)));
            }

            // 3. Generate the "Blob" Shape
            // We interpolate the radius value for every degree of the circle to create
            // a smooth transition that forms a perfect circle when stats are equal.
            let mut points = Vec::new();
            let steps = 72; // Higher = smoother circle

            for i in 0..steps {
                let angle_rad = (i as f32 / steps as f32) * 2.0 * PI;
                let angle_deg = angle_rad.to_degrees().rem_euclid(360.0);

                // Interpolate the radius based on which sector (wedge) of the circle we are in.
                let radius_val = if (45.0..135.0).contains(&angle_deg) {
                    // Sector: Dex -> Str
                    let t = (angle_rad - ANG_DEX_GREEN) / (ANG_STR_RED - ANG_DEX_GREEN);
                    self.dexterity + (self.strength - self.dexterity) * t
                } else if (135.0..270.0).contains(&angle_deg) {
                    // Sector: Str -> Int
                    let t = (angle_rad - ANG_STR_RED) / (ANG_INT_BLUE - ANG_STR_RED);
                    self.strength + (self.intelligence - self.strength) * t
                } else {
                    // Sector: Int -> Dex (wraps around 360/0)
                    let start = ANG_INT_BLUE;
                    let end = ANG_DEX_GREEN + 2.0 * PI;
                    let curr = if angle_rad < ANG_INT_BLUE {
                        angle_rad + 2.0 * PI
                    } else {
                        angle_rad
                    };
                    let t = (curr - start) / (end - start);
                    self.intelligence + (self.dexterity - self.intelligence) * t
                };

                // Convert Polar (angle, radius) to Cartesian (x, y)
                let r_px = (radius_val / MAX_STAT_VAL) * max_radius;
                points.push(Pos2::new(
                    center.x + r_px * angle_rad.cos(),
                    center.y - r_px * angle_rad.sin(),
                ));
            }

            // 4. Draw the Shape
            let total = self.strength + self.dexterity + self.intelligence;
            let fill_color = if total > 0.0 {
                Color32::from_rgba_premultiplied(
                    (self.strength / total * 200.0) as u8,     // Red component
                    (self.dexterity / total * 200.0) as u8,    // Green component
                    (self.intelligence / total * 200.0) as u8, // Blue component
                    128,
                )
            } else {
                Color32::TRANSPARENT
            };

            painter.add(Shape::Path(egui::epaint::PathShape {
                points,
                closed: true,
                fill: fill_color,
                stroke: Stroke::new(2.0, Color32::WHITE),
            }));
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Stat Plotter",
        options,
        Box::new(|_cc| Box::<StatApp>::default()),
    )
}
