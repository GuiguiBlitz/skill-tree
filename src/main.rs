use eframe::egui;
use egui::{Color32, Pos2, Shape, Stroke};
use rand::Rng;
use std::f32::consts::PI;

const MAX_STAT_VAL: f32 = 100.0;
const MIN_STAT_VAL: f32 = 10.0;
const MAX_TOTAL_POINTS: f32 = 120.0;

// Angles (radians)
const ANG_STR_RED: f32 = 135.0 * (PI / 180.0);
const ANG_DEX_GREEN: f32 = 45.0 * (PI / 180.0);
const ANG_INT_BLUE: f32 = 270.0 * (PI / 180.0);

#[derive(Clone)]
struct PerkPoint {
    name: String,
    description: String,
    angle: f32,
    radius_val: f32,
    cost: f32,
}

struct StatApp {
    strength: f32,
    intelligence: f32,
    dexterity: f32,
    zoom: f32,
    offset: egui::Vec2,
    perks: Vec<PerkPoint>,
}

impl StatApp {
    // Shared math for elliptical radius calculation
    // We make this static so we can use it during generation
    fn calculate_ellipse_radius(v1: f32, v2: f32, t_sector: f32) -> f32 {
        if v1 < 1.0 && v2 < 1.0 {
            0.0
        } else {
            let phi = t_sector * (PI / 2.0);
            (v1 * v2) / ((v2 * phi.cos()).powi(2) + (v1 * phi.sin()).powi(2)).sqrt()
        }
    }
}

impl Default for StatApp {
    fn default() -> Self {
        let mut perks = Vec::new();
        let mut rng = rand::thread_rng();

        // --- A. Fixed SUPERNOVAS (Cost 10.0) ---
        let fixed_supernovas = vec![
            (
                "Warrior",
                "Increase area of effect by 30%",
                ANG_STR_RED,
                80.0,
            ),
            ("Ranger", "+ 2 additional projectiles", ANG_DEX_GREEN, 80.0),
            (
                "Mage",
                "Spells chain to 2 additional targets",
                ANG_INT_BLUE,
                80.0,
            ),
            (
                "Duelist",
                "Attack speed scales with STR/DEX",
                (ANG_STR_RED + ANG_DEX_GREEN) / 2.0,
                40.0,
            ),
            (
                "Monk",
                "Unarmed strikes stun enemies",
                (ANG_STR_RED + ANG_DEX_GREEN) / 2.0,
                55.0,
            ),
            (
                "Ranger-Mage",
                "Arrows deal 5% more elemental damage",
                (ANG_DEX_GREEN + (ANG_INT_BLUE + 2.0 * PI)) / 2.0,
                40.0,
            ),
            (
                "Arcane Trickster",
                "Teleport on crit",
                (ANG_DEX_GREEN + (ANG_INT_BLUE + 2.0 * PI)) / 2.0,
                55.0,
            ),
            (
                "Battlemage",
                "Gain Energy Shield based on INT",
                (ANG_INT_BLUE + ANG_STR_RED) / 2.0,
                40.0,
            ),
            (
                "Paladin",
                "Heal allies on hit",
                (ANG_INT_BLUE + ANG_STR_RED) / 2.0,
                55.0,
            ),
        ];

        for (name, desc, angle, rad) in fixed_supernovas {
            perks.push(PerkPoint {
                name: name.to_string(),
                description: desc.to_string(),
                angle,
                radius_val: rad,
                cost: 10.0,
            });
        }

        // --- RANDOM GENERATION HELPERS ---
        // We define the sectors to pick from
        // (Start Angle, End Angle, V1_is_Start?)
        let sectors = [
            (ANG_DEX_GREEN, ANG_STR_RED, true),             // Dex -> Str
            (ANG_STR_RED, ANG_INT_BLUE, true),              // Str -> Int
            (ANG_INT_BLUE, ANG_DEX_GREEN + 2.0 * PI, true), // Int -> Dex
        ];

        // Function to generate a point inside the reachable area
        let mut generate_safe_point = |pt_name: String, cost: f32, min_r_percent: f32| {
            // 1. Pick a random sector
            let sector_idx = rng.gen_range(0..3);
            let (start_ang, end_ang, _) = sectors[sector_idx];

            // 2. Pick a random 't' (position in the arc)
            let t = rng.gen_range(0.0..1.0);

            // 3. Calculate exact Angle
            let angle = start_ang + t * (end_ang - start_ang);

            // 4. Calculate the MAX POSSIBLE Radius at this angle given the 120 point cap.
            // We have 120 points. Min stat is 10. So we have 90 points to distribute between V1 and V2.
            // At t=0 (Axis), V1=100, V2=10.
            // At t=0.5 (Midpoint), V1=55, V2=55.
            // At t=1 (Next Axis), V1=10, V2=100.
            // We approximate the boundary distribution linearly based on 't':
            let max_v1 = 100.0 - (90.0 * t);
            let max_v2 = 10.0 + (90.0 * t);

            // Calculate the physical radius limit using the elliptical formula
            let max_radius_limit = StatApp::calculate_ellipse_radius(max_v1, max_v2, t);

            // 5. Generate a random radius *inside* this limit
            // We ensure it's not too close to the center (min_r_percent)
            let radius = rng.gen_range((max_radius_limit * min_r_percent)..max_radius_limit);

            perks.push(PerkPoint {
                name: pt_name,
                description: "Passive bonus".to_string(),
                angle,
                radius_val: radius,
                cost,
            });
        };

        // --- B. Generate 40 RED GIANTS (Cost 5.0) ---
        for i in 0..40 {
            generate_safe_point(format!("Red Giant {}", i + 1), 5.0, 0.4);
        }

        // --- C. Generate 300 STARS (Cost 2.0) ---
        for i in 0..300 {
            generate_safe_point(format!("Star {}", i + 1), 2.0, 0.2);
        }

        Self {
            strength: MIN_STAT_VAL,
            intelligence: MIN_STAT_VAL,
            dexterity: MIN_STAT_VAL,
            zoom: 1.0,
            offset: egui::Vec2::ZERO,
            perks,
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
            ui.add_enabled_ui(*value > MIN_STAT_VAL, |ui| {
                if ui.button("-").clicked() {
                    *value = (*value - 5.0).clamp(MIN_STAT_VAL, MAX_STAT_VAL);
                }
            });
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

        // Use the shared static helper
        Self::calculate_ellipse_radius(v1, v2, t_sector)
    }
}

impl eframe::App for StatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let total_points = self.strength + self.intelligence + self.dexterity;

        egui::SidePanel::left("controls_panel").show(ctx, |ui| {
            ui.add_space(20.0);
            ui.heading("Build Stats");
            ui.label(format!("Points: {} / {}", total_points, MAX_TOTAL_POINTS));
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
            ui.add_space(20.0);
            ui.small("Drag to Pan • Scroll to Zoom");

            ui.add_space(20.0);
            ui.separator();
            ui.heading("Legend");
            ui.small("Circle thickness = Cost");
            ui.add_space(10.0);
            for (name, cost) in [("Supernova", 10.0), ("Red Giant", 5.0), ("Star", 2.0)] {
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
                    ui.painter().circle(
                        rect.center(),
                        5.0,
                        Color32::from_gray(40),
                        Stroke::new(cost, Color32::YELLOW),
                    );
                    ui.label(name);
                });
                ui.add_space(5.0);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(
                ui.available_size_before_wrap(),
                egui::Sense::click_and_drag(),
            );
            if response.dragged_by(egui::PointerButton::Primary) {
                self.offset += response.drag_delta();
            }
            let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta != 0.0 {
                let zoom_factor = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
                self.zoom = (self.zoom * zoom_factor).clamp(0.1, 10.0);
            }

            let center = response.rect.center() + self.offset;
            let max_radius =
                (response.rect.width().min(response.rect.height()) / 2.0 * 0.8) * self.zoom;

            let arc_segments = vec![
                (337.5f32, 450.0f32, Color32::GREEN),
                (90.0f32, 202.5f32, Color32::RED),
                (202.5f32, 337.5f32, Color32::BLUE),
            ];
            for (start, end, col) in arc_segments {
                let mut pts = Vec::new();
                for i in 0..=60 {
                    let rad = (start + (i as f32 / 60.0) * (end - start)).to_radians();
                    pts.push(Pos2::new(
                        center.x + max_radius * rad.cos(),
                        center.y - max_radius * rad.sin(),
                    ));
                }
                painter.add(Shape::line(pts, Stroke::new(4.0, col.gamma_multiply(0.6))));
            }
            for &(ang, col) in &[
                (ANG_STR_RED, Color32::RED),
                (ANG_DEX_GREEN, Color32::GREEN),
                (ANG_INT_BLUE, Color32::BLUE),
            ] {
                let tip = Pos2::new(
                    center.x + max_radius * ang.cos(),
                    center.y - max_radius * ang.sin(),
                );
                painter.line_segment([center, tip], Stroke::new(1.0, col.gamma_multiply(0.3)));
            }

            let mut blob_pts = Vec::new();
            for i in 0..90 {
                let a = (i as f32 / 90.0) * 2.0 * PI;
                let r = (self.get_current_radius_at_angle(a) / MAX_STAT_VAL) * max_radius;
                blob_pts.push(Pos2::new(center.x + r * a.cos(), center.y - r * a.sin()));
            }
            let fill = if total_points > 0.0 {
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
                points: blob_pts,
                closed: true,
                fill,
                stroke: Stroke::new(2.5, Color32::WHITE),
            }));

            let pointer_pos = ctx.input(|i| i.pointer.hover_pos());

            for perk in &self.perks {
                let r_px = (perk.radius_val / MAX_STAT_VAL) * max_radius;
                let pos = Pos2::new(
                    center.x + r_px * perk.angle.cos(),
                    center.y - r_px * perk.angle.sin(),
                );
                let is_unlocked =
                    perk.radius_val <= self.get_current_radius_at_angle(perk.angle) + 0.5;

                let (col, stroke_col, base_rad) = if is_unlocked {
                    (Color32::YELLOW, Color32::WHITE, 5.0)
                } else {
                    (Color32::from_gray(40), Color32::GRAY, 3.0)
                };

                let vis_rad = base_rad * self.zoom.clamp(0.5, 2.0);
                painter.circle(
                    pos,
                    vis_rad,
                    col,
                    Stroke::new(perk.cost * self.zoom.clamp(0.5, 2.0), stroke_col),
                );

                if perk.cost >= 10.0 {
                    painter.text(
                        pos + egui::Vec2::new(0.0, -10.0 * self.zoom.clamp(0.5, 2.0)),
                        egui::Align2::CENTER_BOTTOM,
                        &perk.name,
                        egui::FontId::proportional(12.0 * self.zoom.clamp(0.5, 1.5)),
                        Color32::WHITE,
                    );
                }

                if pointer_pos
                    .filter(|m| m.distance(pos) <= vis_rad.max(10.0))
                    .is_some()
                {
                    ctx.request_repaint();
                    egui::show_tooltip_at(ctx, egui::Id::new(&perk.name), Some(pos), |ui| {
                        ui.label(egui::RichText::new(&perk.name).strong().size(14.0));
                        ui.horizontal(|ui| {
                            ui.label("Cost:");
                            ui.label(format!("{:.1}", perk.cost));
                        });
                        ui.separator();

                        ui.horizontal(|ui| {
                            let req = perk.radius_val;
                            let deg = perk.angle.to_degrees().rem_euclid(360.0);
                            let eps = 5.0;

                            if (deg - ANG_STR_RED.to_degrees()).abs() < eps {
                                ui.colored_label(
                                    Color32::from_rgb(255, 80, 80),
                                    format!("{:.0} STR", req),
                                );
                            } else if (deg - ANG_DEX_GREEN.to_degrees()).abs() < eps {
                                ui.colored_label(
                                    Color32::from_rgb(80, 255, 80),
                                    format!("{:.0} DEX", req),
                                );
                            } else if (deg - ANG_INT_BLUE.to_degrees()).abs() < eps {
                                ui.colored_label(
                                    Color32::from_rgb(80, 80, 255),
                                    format!("{:.0} INT", req),
                                );
                            } else {
                                if (45.0..135.0).contains(&deg) {
                                    ui.colored_label(
                                        Color32::from_rgb(255, 80, 80),
                                        format!("{:.0} STR", req),
                                    );
                                    ui.label("+");
                                    ui.colored_label(
                                        Color32::from_rgb(80, 255, 80),
                                        format!("{:.0} DEX", req),
                                    );
                                } else if (135.0..270.0).contains(&deg) {
                                    ui.colored_label(
                                        Color32::from_rgb(255, 80, 80),
                                        format!("{:.0} STR", req),
                                    );
                                    ui.label("+");
                                    ui.colored_label(
                                        Color32::from_rgb(80, 80, 255),
                                        format!("{:.0} INT", req),
                                    );
                                } else {
                                    ui.colored_label(
                                        Color32::from_rgb(80, 255, 80),
                                        format!("{:.0} DEX", req),
                                    );
                                    ui.label("+");
                                    ui.colored_label(
                                        Color32::from_rgb(80, 80, 255),
                                        format!("{:.0} INT", req),
                                    );
                                }
                            }
                        });
                        ui.separator();
                        ui.label(&perk.description);
                        if !is_unlocked {
                            ui.label(egui::RichText::new("LOCKED").color(Color32::RED).small());
                        }
                    });
                }
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
