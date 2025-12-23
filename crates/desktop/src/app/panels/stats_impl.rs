//! Platform implementation for shared stats panel

use eframe::egui::{self, Ui};
use overachiever_core::{Game, RunHistory, AchievementHistory, LogEntry, StatsPanelPlatform};

use crate::app::SteamOverachieverApp;
use crate::db::{open_connection, set_achievement_rating};
use crate::cloud_sync::submit_achievement_rating;

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
    
    fn achievements_graph_tab(&self) -> usize {
        self.achievements_graph_tab
    }
    
    fn set_achievements_graph_tab(&mut self, tab: usize) {
        self.achievements_graph_tab = tab;
    }
    
    fn games_graph_tab(&self) -> usize {
        self.games_graph_tab
    }
    
    fn set_games_graph_tab(&mut self, tab: usize) {
        self.games_graph_tab = tab;
    }
    
    fn is_authenticated(&self) -> bool {
        self.config.cloud_token.is_some()
    }
    
    fn get_user_achievement_rating(&self, appid: u64, apiname: &str) -> Option<u8> {
        self.user_achievement_ratings.get(&(appid, apiname.to_string())).copied()
    }
    
    fn set_user_achievement_rating(&mut self, appid: u64, apiname: String, rating: u8) {
        // Store in memory for immediate UI feedback
        self.user_achievement_ratings.insert((appid, apiname.clone()), rating);
        
        // Persist to local database (for offline/quick access)
        let steam_id = self.config.steam_id.clone();
        if let Ok(conn) = open_connection() {
            let _ = set_achievement_rating(&conn, &steam_id, appid, &apiname, rating);
        }
        
        // Submit to remote server if authenticated
        if let Some(token) = &self.config.cloud_token {
            submit_achievement_rating(token, appid, &apiname, rating);
        }
    }
    
    fn navigate_to_achievement(&mut self, appid: u64, apiname: String) {
        // Clear filters so the game is visible
        self.filter_name.clear();
        self.filter_achievements = crate::ui::TriFilter::All;
        self.filter_playtime = crate::ui::TriFilter::All;
        
        // Expand the game row
        self.expanded_rows.insert(appid);
        
        // Load achievements if not cached
        if !self.achievements_cache.contains_key(&appid) {
            if let Ok(conn) = open_connection() {
                if let Ok(achs) = crate::db::get_game_achievements(&conn, &self.config.steam_id, appid) {
                    self.achievements_cache.insert(appid, achs);
                }
            }
        }
        
        // Set navigation target for scroll-to behavior and enable one-time scroll
        self.navigation_target = Some((appid, apiname));
        self.needs_scroll_to_target = true;
    }
    
    fn get_log_selected_achievement(&self) -> Option<(u64, String)> {
        self.log_selected_achievement.clone()
    }
    
    fn set_log_selected_achievement(&mut self, appid: u64, apiname: String) {
        self.log_selected_achievement = Some((appid, apiname));
    }
}