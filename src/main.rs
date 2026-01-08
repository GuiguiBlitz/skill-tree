use eframe::egui;
use egui::{Color32, Pos2, Shape, Stroke};
use std::f32::consts::PI;

const MAX_STAT_VAL: f32 = 100.0;
const MIN_STAT_VAL: f32 = 10.0;
const MAX_TOTAL_POINTS: f32 = 120.0;

// Angles (radians)
const ANG_STR_RED: f32 = 135.0 * (PI / 180.0);
const ANG_DEX_GREEN: f32 = 45.0 * (PI / 180.0);
const ANG_INT_BLUE: f32 = 270.0 * (PI / 180.0);

struct StatApp {
    strength: f32,
    intelligence: f32,
    dexterity: f32,
}

struct PerkPoint {
    name: &'static str,
    angle: f32,
    radius_val: f32,
}

impl Default for StatApp {
    fn default() -> Self {
        Self {
            // Stats start at the minimum (10), not 0
            strength: MIN_STAT_VAL,
            intelligence: MIN_STAT_VAL,
            dexterity: MIN_STAT_VAL,
        }
    }
}

impl StatApp {
    fn stat_control_ui(
        ui: &mut egui::Ui,
        label: &str,
        value: &mut f32,
        color: Color32,
        current_total: f32,
    ) {
        ui.horizontal(|ui| {
            ui.colored_label(color, format!("{}: {:>3.0}", label, value));
            ui.add_space(10.0);

            // Decrease: Check against MIN_STAT_VAL (10.0)
            // We disable the button if we are already at the floor
            ui.add_enabled_ui(*value > MIN_STAT_VAL, |ui| {
                if ui.button("-").clicked() {
                    *value = (*value - 5.0).clamp(MIN_STAT_VAL, MAX_STAT_VAL);
                }
            });

            // Increase: Check against Global Max and Stat Max
            let remaining_global = MAX_TOTAL_POINTS - current_total;
            let remaining_stat = MAX_STAT_VAL - *value;
            let can_add = remaining_global > 0.0 && remaining_stat > 0.0;

            ui.add_enabled_ui(can_add, |ui| {
                if ui.button("+").clicked() {
                    let increment = 5.0f32.min(remaining_global);
                    *value = (*value + increment).clamp(MIN_STAT_VAL, MAX_STAT_VAL);
                }
            });
        });
    }

    fn get_current_radius_at_angle(&self, angle_rad: f32) -> f32 {
        let angle_deg = angle_rad.to_degrees().rem_euclid(360.0);

        let (v1, v2, t_sector) = if (45.0..135.0).contains(&angle_deg) {
            let t = (angle_rad - ANG_DEX_GREEN) / (ANG_STR_RED - ANG_DEX_GREEN);
            (self.dexterity, self.strength, t)
        } else if (135.0..270.0).contains(&angle_deg) {
            let t = (angle_rad - ANG_STR_RED) / (ANG_INT_BLUE - ANG_STR_RED);
            (self.strength, self.intelligence, t)
        } else {
            let start = ANG_INT_BLUE;
            let end = ANG_DEX_GREEN + 2.0 * PI;
            let curr = if angle_rad < ANG_INT_BLUE {
                angle_rad + 2.0 * PI
            } else {
                angle_rad
            };
            let t = (curr - start) / (end - start);
            (self.intelligence, self.dexterity, t)
        };

        if v1 < 1.0 && v2 < 1.0 {
            0.0
        } else {
            let phi = t_sector * (PI / 2.0);
            (v1 * v2) / ((v2 * phi.cos()).powi(2) + (v1 * phi.sin()).powi(2)).sqrt()
        }
    }
}

impl eframe::App for StatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let total_points = self.strength + self.intelligence + self.dexterity;

        egui::SidePanel::left("controls_panel").show(ctx, |ui| {
            ui.add_space(20.0);
            ui.heading("Build Stats");

            ui.label(format!("Points: {} / {}", total_points, MAX_TOTAL_POINTS));
            // Progress bar shows how much of the 140 is used
            ui.add(egui::ProgressBar::new(total_points / MAX_TOTAL_POINTS).show_percentage());

            ui.add_space(20.0);

            Self::stat_control_ui(
                ui,
                "STR",
                &mut self.strength,
                Color32::from_rgb(200, 50, 50),
                total_points,
            );
            Self::stat_control_ui(
                ui,
                "INT",
                &mut self.intelligence,
                Color32::from_rgb(50, 50, 200),
                total_points,
            );
            Self::stat_control_ui(
                ui,
                "DEX",
                &mut self.dexterity,
                Color32::from_rgb(50, 200, 50),
                total_points,
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
            let max_radius = response.rect.width().min(response.rect.height()) / 2.0 * 0.8;

            // Draw Background
            painter.circle_stroke(center, max_radius, Stroke::new(1.0, Color32::from_gray(60)));
            for &(angle, color) in &[
                (ANG_STR_RED, Color32::RED),
                (ANG_DEX_GREEN, Color32::GREEN),
                (ANG_INT_BLUE, Color32::BLUE),
            ] {
                let tip = Pos2::new(
                    center.x + max_radius * angle.cos(),
                    center.y - max_radius * angle.sin(),
                );
                painter.line_segment([center, tip], Stroke::new(1.0, color.gamma_multiply(0.3)));
            }

            // Draw Blob
            let mut shape_points = Vec::new();
            let steps = 90;
            for i in 0..steps {
                let angle = (i as f32 / steps as f32) * 2.0 * PI;
                let r_val = self.get_current_radius_at_angle(angle);
                let r_px = (r_val / MAX_STAT_VAL) * max_radius;
                shape_points.push(Pos2::new(
                    center.x + r_px * angle.cos(),
                    center.y - r_px * angle.sin(),
                ));
            }

            let fill_color = if total_points > 0.0 {
                Color32::from_rgba_premultiplied(
                    (self.strength / total_points * 200.0) as u8,
                    (self.dexterity / total_points * 200.0) as u8,
                    (self.intelligence / total_points * 200.0) as u8,
                    150,
                )
            } else {
                Color32::TRANSPARENT
            };

            painter.add(Shape::Path(egui::epaint::PathShape {
                points: shape_points,
                closed: true,
                fill: fill_color,
                stroke: Stroke::new(2.5, Color32::WHITE),
            }));

            // Draw Perks
            let perks = vec![
                PerkPoint {
                    name: "Warrior and Area modifiers",
                    angle: ANG_STR_RED,
                    radius_val: 80.0,
                },
                PerkPoint {
                    name: "Ranger and Projectiles modifiers",
                    angle: ANG_DEX_GREEN,
                    radius_val: 80.0,
                },
                PerkPoint {
                    name: "Mage and Forking modifiers",
                    angle: ANG_INT_BLUE,
                    radius_val: 80.0,
                },
                PerkPoint {
                    name: "Duelist",
                    angle: (ANG_STR_RED + ANG_DEX_GREEN) / 2.0,
                    radius_val: 40.0,
                },
                PerkPoint {
                    name: "Monk",
                    angle: (ANG_STR_RED + ANG_DEX_GREEN) / 2.0,
                    radius_val: 55.0,
                },
                PerkPoint {
                    name: "Ranger-Mage",
                    angle: (ANG_DEX_GREEN + (ANG_INT_BLUE + 2.0 * PI)) / 2.0,
                    radius_val: 40.0,
                },
                PerkPoint {
                    name: "Arcane Trickster",
                    angle: (ANG_DEX_GREEN + (ANG_INT_BLUE + 2.0 * PI)) / 2.0,
                    radius_val: 55.0,
                },
                PerkPoint {
                    name: "Battlemage",
                    angle: (ANG_INT_BLUE + ANG_STR_RED) / 2.0,
                    radius_val: 40.0,
                },
                PerkPoint {
                    name: "Paladin",
                    angle: (ANG_INT_BLUE + ANG_STR_RED) / 2.0,
                    radius_val: 55.0,
                },
            ];

            for perk in perks {
                let r_px = (perk.radius_val / MAX_STAT_VAL) * max_radius;
                let pos = Pos2::new(
                    center.x + r_px * perk.angle.cos(),
                    center.y - r_px * perk.angle.sin(),
                );
                let current_radius_at_perk_angle = self.get_current_radius_at_angle(perk.angle);
                // Check if the perk is unlocked (radius check)
                let is_unlocked = perk.radius_val <= current_radius_at_perk_angle + 0.5;

                let (color, stroke_color, radius) = if is_unlocked {
                    (Color32::YELLOW, Color32::WHITE, 5.0)
                } else {
                    (Color32::from_gray(40), Color32::GRAY, 3.0)
                };

                painter.circle(pos, radius, color, Stroke::new(1.0, stroke_color));
                // if is_unlocked {
                painter.text(
                    pos + egui::Vec2::new(0.0, -10.0),
                    egui::Align2::CENTER_BOTTOM,
                    perk.name,
                    egui::FontId::proportional(12.0),
                    Color32::WHITE,
                );
                // }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Stat Plotter",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1920.0, 1080.0]),
            ..Default::default()
        },
        Box::new(|_cc| Box::<StatApp>::default()),
    )
}
