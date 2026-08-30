//! Patch canvas: nodes, cables, interaction.

mod cable;
mod debug_log;
mod knob;
mod node_view;

use std::collections::HashMap;

use eframe::egui;
use rtrb::Producer;
use waver_core::{NodeId, NodeKind, ParamId, RtCommand, param_label};

use crate::patch_state::PatchState;

pub use cable::CableState;

use self::cable::{draw_cable, jack_at, JackPos};
use self::knob::{KnobScale, rotary_knob, wave_selector};
use self::node_view::show_node;

const NODE_HEADER_HEIGHT: f32 = 24.0;

/// Patch editor widget.
pub struct PatchEditor {
    cable: CableState,
    drag_node: Option<NodeId>,
    drag_offset: egui::Vec2,
    jack_cache: HashMap<NodeId, Vec<JackPos>>,
}

impl Default for PatchEditor {
    fn default() -> Self {
        Self {
            cable: CableState::Idle,
            drag_node: None,
            drag_offset: egui::Vec2::ZERO,
            jack_cache: HashMap::new(),
        }
    }
}

impl PatchEditor {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        patch: &mut PatchState,
        commands: &mut Producer<RtCommand>,
    ) {
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.cable.cancel();
        }

        ui.horizontal(|ui| {
            if ui.button("+ VCO").clicked() {
                patch.add_node(NodeKind::Vco);
                patch.recompile(commands);
            }
            if ui.button("+ Output").clicked() {
                patch.add_node(NodeKind::Output);
                patch.recompile(commands);
            }
            if ui.button("删除选中").clicked() && patch.remove_selected() {
                patch.recompile(commands);
            }
        });

        if let Some(err) = &patch.compile_error {
            ui.colored_label(egui::Color32::from_rgb(220, 80, 80), err.to_string());
        }

        ui.separator();

        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            egui::Sense::click_and_drag(),
        );
        let rect = response.rect;
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(28, 30, 36));

        self.jack_cache.clear();
        let pointer = ui.input(|i| i.pointer.interact_pos());
        let mut all_jacks: Vec<JackPos> = Vec::new();
        let params = patch.compiled.as_ref().map(|p| p.params.clone());

        // #region agent log
        {
            let primary = ui.input(|i| i.pointer.primary_down());
            let secondary = ui.input(|i| i.pointer.secondary_down());
            if response.clicked()
                || response.dragged()
                || response.drag_started()
                || response.drag_stopped()
                || secondary
                || ui.input(|i| i.pointer.secondary_clicked())
                || (primary && (response.hovered() || self.drag_node.is_some()))
            {
                let p = pointer.unwrap_or(egui::Pos2::ZERO);
                debug_log::agent_log(
                    "A",
                    "editor/mod.rs:canvas",
                    "canvas_pointer",
                    &format!(
                        "{{\"ptr\":[{:.1},{:.1}],\"hovered\":{},\"clicked\":{},\"drag_started\":{},\"dragged\":{},\"drag_stopped\":{},\"primary_down\":{},\"secondary_down\":{},\"secondary_clicked\":{},\"drag_node\":{},\"params_present\":{}}}",
                        p.x,
                        p.y,
                        response.hovered(),
                        response.clicked(),
                        response.drag_started(),
                        response.dragged(),
                        response.drag_stopped(),
                        primary,
                        secondary,
                        ui.input(|i| i.pointer.secondary_clicked()),
                        self.drag_node.map(|id| id.raw()).unwrap_or(u32::MAX),
                        params.is_some()
                    ),
                );
            }
        }
        // #endregion

        let nodes: Vec<_> = patch.graph.nodes().to_vec();
        for node in &nodes {
            let pos = patch.position(node.id);
            let selected = patch.selected == Some(node.id);
            let layout = show_node(
                ui,
                node,
                pos,
                selected,
                params.as_ref(),
            );
            self.jack_cache.insert(node.id, layout.jacks.clone());
            all_jacks.extend(layout.jacks.clone());

            let header_rect = egui::Rect::from_min_size(
                layout.rect.min,
                egui::vec2(layout.rect.width(), NODE_HEADER_HEIGHT),
            );

            // #region agent log
            if response.clicked() || response.drag_started() || response.dragged() {
                let p = pointer.unwrap_or(egui::Pos2::ZERO);
                debug_log::agent_log(
                    "B",
                    "editor/mod.rs:node_drag",
                    "node_hit_test",
                    &format!(
                        "{{\"node\":{},\"ptr\":[{:.1},{:.1}],\"pos\":[{:.1},{:.1}],\"in_header\":{},\"in_body\":{},\"header\":[{:.1},{:.1},{:.1},{:.1}],\"body\":[{:.1},{:.1},{:.1},{:.1}],\"drag_started\":{},\"clicked\":{}}}",
                        node.id.raw(),
                        p.x,
                        p.y,
                        pos.x,
                        pos.y,
                        header_rect.contains(p),
                        layout.rect.contains(p),
                        header_rect.min.x,
                        header_rect.min.y,
                        header_rect.max.x,
                        header_rect.max.y,
                        layout.rect.min.x,
                        layout.rect.min.y,
                        layout.rect.max.x,
                        layout.rect.max.y,
                        response.drag_started(),
                        response.clicked()
                    ),
                );
            }
            // #endregion

            if response.clicked() {
                if header_rect.contains(pointer.unwrap_or(egui::Pos2::ZERO)) {
                    patch.selected = Some(node.id);
                } else if !layout.rect.contains(pointer.unwrap_or(egui::Pos2::ZERO))
                    && patch.selected.is_some()
                {
                    patch.selected = None;
                }
            }

            if response.drag_started()
                && header_rect.contains(pointer.unwrap_or(egui::Pos2::ZERO))
            {
                // #region agent log
                debug_log::agent_log(
                    "B",
                    "editor/mod.rs:node_drag",
                    "drag_node_acquired",
                    &format!(
                        "{{\"node\":{},\"offset\":[{:.1},{:.1}]}}",
                        node.id.raw(),
                        pointer.unwrap().x - pos.x,
                        pointer.unwrap().y - pos.y
                    ),
                );
                // #endregion
                self.drag_node = Some(node.id);
                self.drag_offset = pointer.unwrap() - pos;
            }
        }

        if response.dragged() {
            if let (Some(id), Some(p)) = (self.drag_node, pointer) {
                // #region agent log
                debug_log::agent_log(
                    "B",
                    "editor/mod.rs:node_drag",
                    "drag_node_move",
                    &format!(
                        "{{\"node\":{},\"new_pos\":[{:.1},{:.1}]}}",
                        id.raw(),
                        (p - self.drag_offset).x,
                        (p - self.drag_offset).y
                    ),
                );
                // #endregion
                patch.set_position(id, p - self.drag_offset);
            }
        }
        if response.drag_stopped() {
            // #region agent log
            if self.drag_node.is_some() {
                debug_log::agent_log(
                    "B",
                    "editor/mod.rs:node_drag",
                    "drag_node_stopped",
                    &format!("{{\"node\":{}}}", self.drag_node.unwrap().raw()),
                );
            }
            // #endregion
            self.drag_node = None;
        }

        self.draw_edges(&painter, patch, &all_jacks);

        if let Some(p) = pointer {
            self.handle_cable_input(ui, patch, commands, p, &all_jacks);
        }

        if let CableState::Dragging { from_pos, .. } = &self.cable {
            if let Some(p) = pointer {
                draw_cable(
                    &painter,
                    *from_pos,
                    p,
                    egui::Color32::from_rgb(180, 180, 100),
                    2.0,
                    true,
                );
            }
        }

        ui.separator();
        self.param_panel(ui, patch);
    }

    fn draw_edges(
        &self,
        painter: &egui::Painter,
        patch: &PatchState,
        jacks: &[JackPos],
    ) {
        for edge in patch.graph.edges() {
            let from = jacks.iter().find(|j| j.port == edge.from);
            let to = jacks.iter().find(|j| j.port == edge.to);
            if let (Some(from), Some(to)) = (from, to) {
                draw_cable(
                    painter,
                    from.center,
                    to.center,
                    egui::Color32::from_rgb(200, 200, 80),
                    2.5,
                    false,
                );
            }
        }
    }

    fn handle_cable_input(
        &mut self,
        ui: &egui::Ui,
        patch: &mut PatchState,
        commands: &mut Producer<RtCommand>,
        pointer: egui::Pos2,
        jacks: &[JackPos],
    ) {
        if ui.input(|i| i.pointer.secondary_clicked()) {
            // #region agent log
            let edge_count = patch.graph.edges().len();
            debug_log::agent_log(
                "C",
                "editor/mod.rs:cable_delete",
                "secondary_clicked",
                &format!(
                    "{{\"ptr\":[{:.1},{:.1}],\"edge_count\":{},\"jack_count\":{}}}",
                    pointer.x,
                    pointer.y,
                    edge_count,
                    jacks.len()
                ),
            );
            // #endregion
            for (idx, edge) in patch.graph.edges().iter().enumerate() {
                let from = jacks.iter().find(|j| j.port == edge.from);
                let to = jacks.iter().find(|j| j.port == edge.to);
                if let (Some(from), Some(to)) = (from, to) {
                    let mid = from.center.lerp(to.center, 0.5);
                    let dist = pointer.distance(mid);
                    // #region agent log
                    debug_log::agent_log(
                        "C",
                        "editor/mod.rs:cable_delete",
                        "edge_mid_hit_test",
                        &format!(
                            "{{\"idx\":{},\"from\":[{:.1},{:.1}],\"to\":[{:.1},{:.1}],\"mid\":[{:.1},{:.1}],\"dist\":{:.2},\"hit\":{}}}",
                            idx,
                            from.center.x,
                            from.center.y,
                            to.center.x,
                            to.center.y,
                            mid.x,
                            mid.y,
                            dist,
                            dist < 16.0
                        ),
                    );
                    // #endregion
                    if dist < 16.0 {
                        // #region agent log
                        debug_log::agent_log(
                            "C",
                            "editor/mod.rs:cable_delete",
                            "disconnect_attempt",
                            &format!("{{\"idx\":{},\"ok\":true}}", idx),
                        );
                        // #endregion
                        if patch.disconnect_edge(idx) {
                            patch.recompile(commands);
                        }
                        return;
                    }
                } else {
                    // #region agent log
                    debug_log::agent_log(
                        "D",
                        "editor/mod.rs:cable_delete",
                        "missing_jack_for_edge",
                        &format!(
                            "{{\"idx\":{},\"has_from\":{},\"has_to\":{}}}",
                            idx,
                            from.is_some(),
                            to.is_some()
                        ),
                    );
                    // #endregion
                }
            }
            self.cable.cancel();
            return;
        }

        if ui.input(|i| i.pointer.primary_clicked()) {
            let hit = jack_at(pointer, jacks);
            // #region agent log
            let nearest = jacks
                .iter()
                .map(|j| {
                    (
                        j.center.distance(pointer),
                        j.center,
                        j.is_output,
                        j.port.node.raw(),
                    )
                })
                .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            debug_log::agent_log(
                "D",
                "editor/mod.rs:jack_click",
                "primary_clicked_jack_scan",
                &format!(
                    "{{\"ptr\":[{:.1},{:.1}],\"hit\":{},\"nearest_dist\":{:.2},\"nearest_jack\":[{:.1},{:.1}],\"nearest_out\":{},\"nearest_node\":{},\"jack_count\":{}}}",
                    pointer.x,
                    pointer.y,
                    hit.is_some(),
                    nearest.map(|n| n.0).unwrap_or(-1.0),
                    nearest.map(|n| n.1.x).unwrap_or(0.0),
                    nearest.map(|n| n.1.y).unwrap_or(0.0),
                    nearest.map(|n| n.2).unwrap_or(false),
                    nearest.map(|n| n.3).unwrap_or(u32::MAX),
                    jacks.len()
                ),
            );
            // #endregion
            if let Some(jack) = hit {
                match &self.cable {
                    CableState::Idle => {
                        if jack.is_output {
                            self.cable.start_drag(jack.port, jack.center);
                        }
                    }
                    CableState::Dragging { from, .. } => {
                        if !jack.is_output && jack.port.node != from.node {
                            if patch.try_connect(*from, jack.port) {
                                patch.recompile(commands);
                            }
                            self.cable.cancel();
                        } else if jack.is_output {
                            self.cable.start_drag(jack.port, jack.center);
                        }
                    }
                }
            } else {
                self.cable.cancel();
            }
        }
    }

    fn param_panel(&self, ui: &mut egui::Ui, patch: &PatchState) {
        let Some(selected) = patch.selected else {
            ui.label("选中节点标题栏以编辑；VCO 旋钮可直接在模块上拖动。");
            return;
        };
        let Some(node) = patch.graph.node(selected) else {
            return;
        };
        let Some(compiled) = &patch.compiled else {
            ui.label("补丁尚未编译。");
            return;
        };

        if node.kind == NodeKind::Vco {
            ui.label(
                egui::RichText::new("VCO：拖动 FREQ / AMP 旋钮，或点击波形按钮。双击旋钮恢复默认。")
                    .color(egui::Color32::from_gray(160)),
            );
            ui.horizontal(|ui| {
                if let Some(freq) = compiled.params.get(selected, ParamId::new(0)) {
                    let mut v = freq.value();
                    if rotary_knob(
                        ui,
                        egui::Id::new(("panel_knob", selected.raw(), 0u32)),
                        "频率",
                        &mut v,
                        20.0..=2000.0,
                        KnobScale::Logarithmic,
                    )
                    .changed()
                    {
                        freq.set(v);
                    }
                }
                if let Some(amp) = compiled.params.get(selected, ParamId::new(1)) {
                    let mut v = amp.value();
                    if rotary_knob(
                        ui,
                        egui::Id::new(("panel_knob", selected.raw(), 1u32)),
                        "振幅",
                        &mut v,
                        0.0..=1.0,
                        KnobScale::Linear,
                    )
                    .changed()
                    {
                        amp.set(v);
                    }
                }
                if let Some(wave) = compiled.params.get(selected, ParamId::new(2)) {
                    let mut v = wave.value();
                    wave_selector(ui, &mut v);
                    if (v - wave.value()).abs() > f32::EPSILON {
                        wave.set(v);
                    }
                }
            });
            return;
        }

        ui.heading(format!("{} 参数", kind_label(node.kind)));
        let param_count = node.kind.port_counts().params;
        for raw in 0..param_count {
            let param = ParamId::new(raw);
            let Some(cell) = compiled.params.get(selected, param) else {
                continue;
            };
            let label = param_label(node.kind, param);
            let mut value = cell.value();
            ui.add(egui::Slider::new(&mut value, 0.0..=1.0).text(label));
            cell.set(value);
        }
    }
}

fn kind_label(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Vco => "VCO",
        NodeKind::Output => "Output",
        NodeKind::Delay => "Delay",
        _ => "节点",
    }
}
