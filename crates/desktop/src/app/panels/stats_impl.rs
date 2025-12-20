//! Platform implementation for shared stats panel

use eframe::egui::{self, Ui};
use overachiever_core::{Game, RunHistory, AchievementHistory, LogEntry, StatsPanelPlatform};

use crate::app::SteamOverachieverApp;

impl StatsPanelPlatform for SteamOverachieverApp {
    fn games(&self) -> &[Game] {
        &self.games
    }
    
    fn run_history(&self) -> &[RunHistory] {
        &self.run_history
    }
    
    fn achievement_history(&self) -> &[AchievementHistory] {
        &self.achievement_history
    }
    
    fn log_entries(&self) -> &[LogEntry] {
        &self.log_entries
    }
    
    fn include_unplayed_in_avg(&self) -> bool {
        self.include_unplayed_in_avg
    }
    
    fn set_include_unplayed_in_avg(&mut self, value: bool) {
        self.include_unplayed_in_avg = value;
    }
    
    fn game_icon_source(&self, ui: &Ui, appid: u64, icon_hash: &str) -> egui::ImageSource<'static> {
        let game_icon_url = format!(
            "https://media.steampowered.com/steamcommunity/public/images/apps/{}/{}.jpg",
            appid, icon_hash
        );
        
        if let Some(bytes) = self.icon_cache.get_icon_bytes(&game_icon_url) {
            let cache_uri = format!("bytes://log_game/{}", appid);
            ui.ctx().include_bytes(cache_uri.clone(), bytes);
            egui::ImageSource::Uri(cache_uri.into())
        } else {
            egui::ImageSource::Uri(game_icon_url.into())
        }
    }
    
    fn achievement_icon_source(&self, ui: &Ui, icon_url: &str) -> egui::ImageSource<'static> {
        if let Some(bytes) = self.icon_cache.get_icon_bytes(icon_url) {
            let cache_uri = format!("bytes://log_ach/{}", icon_url.replace(['/', ':', '.'], "_"));
            ui.ctx().include_bytes(cache_uri.clone(), bytes);
            egui::ImageSource::Uri(cache_uri.into())
        } else {
            egui::ImageSource::Uri(icon_url.to_string().into())
        }
    }
}
