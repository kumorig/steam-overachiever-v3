//! History side panel - Uses shared stats panel from core

use eframe::egui;
use overachiever_core::{render_stats_content, StatsPanelConfig};

use crate::app::SteamOverachieverApp;

impl SteamOverachieverApp {
    pub(crate) fn render_history_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("history_panel")
            .min_width(350.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let config = StatsPanelConfig::desktop();
                    render_stats_content(ui, self, &config);
                });
            });
    }
}
