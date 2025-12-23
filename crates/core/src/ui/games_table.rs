//! Games table panel - shared between desktop and WASM
//!
//! Renders: Filterable, sortable games list with expandable achievement details
//! Features: Column sorting, tri-state filters, expandable rows with achievements

use egui::{self, Color32, RichText, Ui};
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular;

use crate::Game;
use super::{StatsPanelPlatform, instant_tooltip};

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Copy, PartialEq, Default)]
pub enum SortColumn {
    #[default]
    Name,
    LastPlayed,
    Playtime,
    AchievementsTotal,
    AchievementsPercent,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum SortOrder {
    #[default]
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn toggle(&self) -> Self {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }
}

/// Tri-state filter: All, Only With, Only Without
#[derive(Clone, Copy, PartialEq, Default)]
pub enum TriFilter {
    #[default]
    All,
    With,
    Without,
}

impl TriFilter {
    pub fn cycle(&self) -> Self {
        match self {
            TriFilter::All => TriFilter::With,
            TriFilter::With => TriFilter::Without,
            TriFilter::Without => TriFilter::All,
        }
    }
    
    pub fn label(&self, with_text: &str, without_text: &str) -> String {
        match self {
            TriFilter::All => "All".to_string(),
            TriFilter::With => with_text.to_string(),
            TriFilter::Without => without_text.to_string(),
        }
    }
}

// ============================================================================
// Games Table Platform Trait
// ============================================================================

/// Platform abstraction for the games table
/// 
/// This trait allows desktop and WASM to provide platform-specific
/// functionality (like icon loading, achievements fetching) while
/// sharing the table rendering logic.
pub trait GamesTablePlatform: StatsPanelPlatform {
    /// Get the current sort column
    fn sort_column(&self) -> SortColumn;
    
    /// Get the current sort order
    fn sort_order(&self) -> SortOrder;
    
    /// Set sort column and toggle order if same column
    fn set_sort(&mut self, column: SortColumn);
    
    /// Get filter text for name search
    fn filter_name(&self) -> &str;
    
    /// Set filter text for name search
    fn set_filter_name(&mut self, name: String);
    
    /// Get achievements filter state
    fn filter_achievements(&self) -> TriFilter;
    
    /// Set achievements filter state
    fn set_filter_achievements(&mut self, filter: TriFilter);
    
    /// Get playtime filter state
    fn filter_playtime(&self) -> TriFilter;
    
    /// Set playtime filter state
    fn set_filter_playtime(&mut self, filter: TriFilter);
    
    /// Check if a game row is expanded
    fn is_expanded(&self, appid: u64) -> bool;
    
    /// Toggle expanded state for a game
    fn toggle_expanded(&mut self, appid: u64);
    
    /// Get cached achievements for a game (if available)
    fn get_cached_achievements(&self, appid: u64) -> Option<&Vec<crate::GameAchievement>>;
    
    /// Request achievements to be loaded for a game
    fn request_achievements(&mut self, appid: u64);
    
    /// Get flash intensity for a row (for highlighting recently updated games)
    /// Returns 0.0-1.0 intensity, or None if not flashing
    fn get_flash_intensity(&self, _appid: u64) -> Option<f32> {
        None
    }
    
    /// Get the current navigation target (appid, apiname) for scroll-to behavior
    /// Returns None if no navigation is pending
    fn get_navigation_target(&self) -> Option<(u64, String)> {
        None
    }
    
    /// Clear the navigation target after scrolling to it
    fn clear_navigation_target(&mut self) {}
    
    /// Check if we need to scroll to the navigation target (one-time scroll)
    fn needs_scroll_to_target(&self) -> bool { false }
    
    /// Mark that we've scrolled to the target (call after scrolling)
    fn mark_scrolled_to_target(&mut self) {}
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Format a Unix timestamp as YYYY-MM-DD
pub fn format_timestamp(ts: u32) -> String {
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "—".to_string())
}

/// Get sort indicator icon for a column
pub fn sort_indicator(platform: &impl GamesTablePlatform, column: SortColumn) -> &'static str {
    if platform.sort_column() == column {
        match platform.sort_order() {
            SortOrder::Ascending => regular::CARET_UP,
            SortOrder::Descending => regular::CARET_DOWN,
        }
    } else {
        ""
    }
}

/// Get filtered indices based on current filters
pub fn get_filtered_indices(platform: &impl GamesTablePlatform) -> Vec<usize> {
    let filter_name_lower = platform.filter_name().to_lowercase();
    
    platform.games().iter()
        .enumerate()
        .filter(|(_, g)| {
            // Name filter
            if !filter_name_lower.is_empty() && !g.name.to_lowercase().contains(&filter_name_lower) {
                return false;
            }
            // Achievements filter
            let has_achievements = g.achievements_total.map(|t| t > 0).unwrap_or(false);
            match platform.filter_achievements() {
                TriFilter::All => {}
                TriFilter::With => if !has_achievements { return false; }
                TriFilter::Without => if has_achievements { return false; }
            }
            // Playtime filter
            let has_playtime = g.rtime_last_played.map(|ts| ts > 0).unwrap_or(false);
            match platform.filter_playtime() {
                TriFilter::All => {}
                TriFilter::With => if !has_playtime { return false; }
                TriFilter::Without => if has_playtime { return false; }
            }
            true
        })
        .map(|(idx, _)| idx)
        .collect()
}

/// Sort games in place based on current sort settings
pub fn sort_games(games: &mut [Game], sort_column: SortColumn, sort_order: SortOrder) {
    match sort_column {
        SortColumn::Name => {
            games.sort_by(|a, b| {
                let cmp = a.name.to_lowercase().cmp(&b.name.to_lowercase());
                if sort_order == SortOrder::Descending { cmp.reverse() } else { cmp }
            });
        }
        SortColumn::LastPlayed => {
            games.sort_by(|a, b| {
                let cmp = b.rtime_last_played.cmp(&a.rtime_last_played);
                if sort_order == SortOrder::Descending { cmp.reverse() } else { cmp }
            });
        }
        SortColumn::Playtime => {
            games.sort_by(|a, b| {
                let cmp = b.playtime_forever.cmp(&a.playtime_forever);
                if sort_order == SortOrder::Descending { cmp.reverse() } else { cmp }
            });
        }
        SortColumn::AchievementsTotal => {
            games.sort_by(|a, b| {
                let cmp = b.achievements_total.cmp(&a.achievements_total);
                if sort_order == SortOrder::Descending { cmp.reverse() } else { cmp }
            });
        }
        SortColumn::AchievementsPercent => {
            games.sort_by(|a, b| {
                let a_pct = a.completion_percent().unwrap_or(-1.0);
                let b_pct = b.completion_percent().unwrap_or(-1.0);
                let cmp = b_pct.partial_cmp(&a_pct).unwrap_or(std::cmp::Ordering::Equal);
                if sort_order == SortOrder::Descending { cmp.reverse() } else { cmp }
            });
        }
    }
}

// ============================================================================
// Render Functions
// ============================================================================

/// Render the filter bar above the games table
pub fn render_filter_bar<P: GamesTablePlatform>(ui: &mut Ui, platform: &mut P) {
    ui.horizontal(|ui| {
        ui.label("Filter:");
        
        let mut filter_name = platform.filter_name().to_string();
        let response = ui.add(egui::TextEdit::singleline(&mut filter_name)
            .hint_text("Search by name...")
            .desired_width(150.0));
        if response.changed() {
            platform.set_filter_name(filter_name);
        }
        
        ui.add_space(10.0);
        
        // Achievements filter - tri-state toggle button
        let ach_label = format!("Achievements: {}", platform.filter_achievements().label("With", "Without"));
        if ui.button(&ach_label).clicked() {
            let next = platform.filter_achievements().cycle();
            platform.set_filter_achievements(next);
        }
        
        // Playtime filter - tri-state toggle button
        let play_label = format!("Played: {}", platform.filter_playtime().label("Yes", "No"));
        if ui.button(&play_label).clicked() {
            let next = platform.filter_playtime().cycle();
            platform.set_filter_playtime(next);
        }
        
        // Clear filters button
        let has_filters = !platform.filter_name().is_empty() 
            || platform.filter_achievements() != TriFilter::All 
            || platform.filter_playtime() != TriFilter::All;
        
        if !has_filters {
            ui.add_enabled(false, egui::Button::new("Clear"));
        } else if ui.button("Clear").clicked() {
            platform.set_filter_name(String::new());
            platform.set_filter_achievements(TriFilter::All);
            platform.set_filter_playtime(TriFilter::All);
        }
    });
}

/// Render the games table
/// 
/// Returns a list of appids that need their achievements fetched
pub fn render_games_table<P: GamesTablePlatform>(ui: &mut Ui, platform: &mut P, filtered_indices: Vec<usize>) -> Vec<u64> {
    let text_height = egui::TextStyle::Body
        .resolve(ui.style())
        .size
        .max(ui.spacing().interact_size.y);
    
    let available_height = ui.available_height();
    
    // Calculate row heights for each filtered game (including expanded achievements)
    let row_heights: Vec<f32> = filtered_indices.iter().map(|&idx| {
        let appid = platform.games()[idx].appid;
        if platform.is_expanded(appid) {
            text_height + 330.0 // Extra height for achievement list
        } else {
            text_height
        }
    }).collect();
    
    // Track which rows need achievement fetch
    let mut needs_fetch: Vec<u64> = Vec::new();
    
    // Clone needed data to avoid borrow issues during table rendering
    let games: Vec<_> = filtered_indices.iter()
        .map(|&idx| platform.games()[idx].clone())
        .collect();
    
    // Find navigation target row index if any (only if we need to scroll)
    let nav_row_index = if platform.needs_scroll_to_target() {
        platform.get_navigation_target().and_then(|(nav_appid, _)| {
            games.iter().position(|g| g.appid == nav_appid)
        })
    } else {
        None
    };
    
    let mut table_builder = TableBuilder::new(ui)
        .striped(true)
        .resizable(false) // Table-level resizing disabled
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder().at_least(200.0).clip(true).resizable(true)) // Name - resizable
        .column(Column::exact(90.0))  // Last Played - fixed
        .column(Column::exact(80.0))  // Playtime - fixed
        .column(Column::exact(100.0)) // Achievements - fixed
        .column(Column::exact(60.0))  // Percent - fixed
        .min_scrolled_height(0.0)
        .max_scroll_height(available_height);
    
    // Scroll to navigation target row if present
    // Note: Don't mark as scrolled here - let the achievement-level scroll do that
    // This ensures clicking a different achievement in the same game still scrolls
    if let Some(row_idx) = nav_row_index {
        table_builder = table_builder.scroll_to_row(row_idx, Some(egui::Align::Center));
    }
    
    table_builder.header(20.0, |mut header| {
            header.col(|ui| {
                let indicator = sort_indicator(platform, SortColumn::Name);
                let label = if indicator.is_empty() { "Name".to_string() } else { format!("Name {}", indicator) };
                if ui.selectable_label(platform.sort_column() == SortColumn::Name, label).clicked() {
                    platform.set_sort(SortColumn::Name);
                }
            });
            header.col(|ui| {
                let indicator = sort_indicator(platform, SortColumn::LastPlayed);
                let label = if indicator.is_empty() { "Last Played".to_string() } else { format!("Last Played {}", indicator) };
                if ui.selectable_label(platform.sort_column() == SortColumn::LastPlayed, label).clicked() {
                    platform.set_sort(SortColumn::LastPlayed);
                }
            });
            header.col(|ui| {
                let indicator = sort_indicator(platform, SortColumn::Playtime);
                let label = if indicator.is_empty() { "Playtime".to_string() } else { format!("Playtime {}", indicator) };
                if ui.selectable_label(platform.sort_column() == SortColumn::Playtime, label).clicked() {
                    platform.set_sort(SortColumn::Playtime);
                }
            });
            header.col(|ui| {
                let indicator = sort_indicator(platform, SortColumn::AchievementsTotal);
                let label = if indicator.is_empty() { "Achievements".to_string() } else { format!("Achievements {}", indicator) };
                if ui.selectable_label(platform.sort_column() == SortColumn::AchievementsTotal, label).clicked() {
                    platform.set_sort(SortColumn::AchievementsTotal);
                }
            });
            header.col(|ui| {
                let indicator = sort_indicator(platform, SortColumn::AchievementsPercent);
                let label = if indicator.is_empty() { "%".to_string() } else { format!("% {}", indicator) };
                if ui.selectable_label(platform.sort_column() == SortColumn::AchievementsPercent, label).clicked() {
                    platform.set_sort(SortColumn::AchievementsPercent);
                }
            });
        })
        .body(|body| {
            body.heterogeneous_rows(row_heights.into_iter(), |mut row| {
                let row_idx = row.index();
                let game = &games[row_idx];
                let appid = game.appid;
                let is_expanded = platform.is_expanded(appid);
                let has_achievements = game.achievements_total.map(|t| t > 0).unwrap_or(false);
                
                // Check if this game should be flashing
                let flash_color = platform.get_flash_intensity(appid).map(|intensity| {
                    egui::Color32::from_rgba_unmultiplied(
                        255,  // R
                        215,  // G (gold)
                        0,    // B
                        (intensity * 100.0) as u8
                    )
                });
                
                // Name column with expand/collapse toggle
                row.col(|ui| {
                    if let Some(color) = flash_color {
                        ui.painter().rect_filled(ui.available_rect_before_wrap(), 0.0, color);
                    }
                    
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            // Expand/collapse button for games with achievements
                            if has_achievements {
                                let icon = if is_expanded { 
                                    regular::CARET_DOWN 
                                } else { 
                                    regular::CARET_RIGHT 
                                };
                                if ui.small_button(icon.to_string()).clicked() {
                                    platform.toggle_expanded(appid);
                                    // Load achievements if not cached and expanding
                                    if !is_expanded && platform.get_cached_achievements(appid).is_none() {
                                        needs_fetch.push(appid);
                                    }
                                }
                            } else {
                                ui.add_space(20.0);
                            }
                            
                            // Show game icon when expanded
                            if is_expanded {
                                if let Some(icon_hash) = &game.img_icon_url {
                                    if !icon_hash.is_empty() {
                                        let img_source = platform.game_icon_source(ui, appid, icon_hash);
                                        ui.add(
                                            egui::Image::new(img_source)
                                                .fit_to_exact_size(egui::vec2(32.0, 32.0))
                                                .corner_radius(4.0)
                                        );
                                    }
                                }
                                ui.label(RichText::new(&game.name).strong());
                            } else {
                                ui.label(&game.name);
                            }
                        });
                        
                        // Show achievements list if expanded
                        if is_expanded {
                            render_achievements_list(ui, platform, appid);
                        }
                    });
                });
                
                // Only show other columns if not expanded
                row.col(|ui| {
                    if let Some(color) = flash_color {
                        ui.painter().rect_filled(ui.available_rect_before_wrap(), 0.0, color);
                    }
                    if !is_expanded {
                        if let Some(ts) = game.rtime_last_played {
                            if ts > 0 {
                                ui.label(format_timestamp(ts));
                            } else {
                                ui.label("—");
                            }
                        } else {
                            ui.label("—");
                        }
                    }
                });
                
                row.col(|ui| {
                    if let Some(color) = flash_color {
                        ui.painter().rect_filled(ui.available_rect_before_wrap(), 0.0, color);
                    }
                    if !is_expanded {
                        let never_played = game.rtime_last_played.map(|ts| ts == 0).unwrap_or(true);
                        if never_played {
                            ui.label("--");
                        } else {
                            ui.label(format!("{:.1}h", game.playtime_forever as f64 / 60.0));
                        }
                    }
                });
                
                row.col(|ui| {
                    if let Some(color) = flash_color {
                        ui.painter().rect_filled(ui.available_rect_before_wrap(), 0.0, color);
                    }
                    if !is_expanded {
                        ui.label(game.achievements_display());
                    }
                });
                
                row.col(|ui| {
                    if let Some(color) = flash_color {
                        ui.painter().rect_filled(ui.available_rect_before_wrap(), 0.0, color);
                    }
                    if !is_expanded {
                        if let Some(pct) = game.completion_percent() {
                            // Green for 100%, gray otherwise
                            let color = if pct >= 100.0 {
                                Color32::from_rgb(100, 255, 100)
                            } else {
                                Color32::GRAY
                            };
                            ui.label(RichText::new(format!("{:.0}%", pct)).color(color));
                        } else {
                            ui.label("—");
                        }
                    }
                });
            });
        });
    
    needs_fetch
}

/// Render the achievements list for an expanded game row
fn render_achievements_list<P: GamesTablePlatform>(ui: &mut Ui, platform: &mut P, appid: u64) {
    // Check if we have a navigation target for this game
    let nav_target = platform.get_navigation_target();
    let target_apiname = nav_target
        .as_ref()
        .filter(|(nav_appid, _)| *nav_appid == appid)
        .map(|(_, apiname)| apiname.clone());
    
    if let Some(achievements) = platform.get_cached_achievements(appid) {
        ui.add_space(4.0);
        ui.separator();
        
        // Sort achievements: unlocked first (by unlock time desc), then locked
        let mut sorted_achs: Vec<_> = achievements.iter().collect();
        sorted_achs.sort_by(|a, b| {
            match (a.achieved, b.achieved) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (true, true) => b.unlocktime.cmp(&a.unlocktime),
                (false, false) => a.name.cmp(&b.name),
            }
        });
        
        // Collect data we need to avoid borrow issues
        let ach_data: Vec<_> = sorted_achs.iter().map(|ach| {
            (
                ach.apiname.clone(),
                ach.name.clone(),
                ach.achieved,
                if ach.achieved { ach.icon.clone() } else { ach.icon_gray.clone() },
                ach.description.clone(),
                ach.unlocktime,
            )
        }).collect();
        
        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            let is_authenticated = platform.is_authenticated();
            for (i, (apiname, name, achieved, icon_url, description, unlocktime)) in ach_data.iter().enumerate() {
                // Check if this is the navigation target
                let is_target = target_apiname.as_ref().map(|t| t == apiname).unwrap_or(false);
                
                let image_source = platform.achievement_icon_source(ui, icon_url);
                // Get user's own rating (for display purposes)
                let user_rating = if is_authenticated {
                    platform.get_user_achievement_rating(appid, apiname)
                } else {
                    None
                };
                // Get community average rating
                let avg_rating_data = platform.get_achievement_avg_rating(appid, apiname);
                
                // Alternate row background, or highlight if target
                let row_rect = ui.available_rect_before_wrap();
                let row_rect = egui::Rect::from_min_size(
                    row_rect.min,
                    egui::vec2(row_rect.width(), 52.0)
                );
                if is_target {
                    // Highlight the target achievement with a golden border
                    ui.painter().rect_filled(
                        row_rect,
                        4.0,
                        Color32::from_rgba_unmultiplied(255, 215, 0, 40) // Gold highlight
                    );
                    ui.painter().rect_stroke(
                        row_rect,
                        4.0,
                        egui::Stroke::new(2.0, Color32::from_rgb(255, 215, 0)),
                        egui::epaint::StrokeKind::Inside,
                    );
                    // Scroll to this row only if we haven't scrolled yet
                    if platform.needs_scroll_to_target() {
                        ui.scroll_to_rect(row_rect, Some(egui::Align::Center));
                        platform.mark_scrolled_to_target();
                    }
                } else if i % 2 == 1 {
                    ui.painter().rect_filled(
                        row_rect,
                        0.0,
                        ui.visuals().faint_bg_color
                    );
                }
                
                // Add top padding for the row content
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    // Add left padding so icon doesn't overlap the gold border
                    ui.add_space(4.0);
                    
                    let icon_response = ui.add(
                        egui::Image::new(image_source)
                            .fit_to_exact_size(egui::vec2(48.0, 48.0))
                            .corner_radius(4.0)
                    );
                    
                    // Show unlock date on hover (instant, no delay)
                    if let Some(unlock_dt) = unlocktime {
                        instant_tooltip(&icon_response, unlock_dt.format("%Y-%m-%d").to_string());
                    }
                    
                    let name_text = if *achieved {
                        RichText::new(name).color(Color32::WHITE)
                    } else {
                        RichText::new(name).color(Color32::DARK_GRAY)
                    };
                    
                    let description_text = description.as_deref().unwrap_or("");
                    let desc_color = if *achieved {
                        Color32::GRAY
                    } else {
                        Color32::from_rgb(80, 80, 80)
                    };
                    
                    ui.vertical(|ui| {
                        ui.add_space(4.0);
                        // Top row: name and date/stars
                        ui.horizontal(|ui| {
                            ui.label(name_text);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Show compact average rating (read-only)
                                // Use average if available, otherwise show user's own rating
                                let (display_rating, count) = if let Some((avg, cnt)) = avg_rating_data {
                                    (Some(avg.round() as u8), Some(cnt))
                                } else {
                                    (user_rating, None)
                                };
                                render_compact_avg_rating(ui, display_rating, count);
                            });
                        });
                        // Description below, full width
                        if !description_text.is_empty() {
                            ui.label(RichText::new(description_text).color(desc_color));
                        }
                    });
                });
            }
        });
    } else {
        ui.spinner();
        ui.label("Loading achievements...");
    }
}

/// Get difficulty label for rating (with trailing space to avoid border clipping)
fn difficulty_label(rating: u8) -> &'static str {
    match rating {
        1 => "Very easy  ",
        2 => "Easy  ",
        3 => "Moderate  ",
        4 => "Hard  ",
        5 => "Extreme  ",
        _ => "",
    }
}

/// Get icon for difficulty rating (single icon per level)
fn difficulty_icon(rating: u8) -> &'static str {
    match rating {
        1 => "🐢",  // Turtle - Very easy
        2 => "🐇",  // Rabbit - Easy
        3 => "🏃",  // Runner - Moderate
        4 => "⚡",  // Lightning - Hard
        5 => "🔥",  // Fire - Extreme
        _ => "",
    }
}

/// Get color for difficulty label (green for easy, red for extreme)
fn difficulty_color(rating: u8) -> Color32 {
    match rating {
        1 => Color32::from_rgb(80, 200, 80),   // Green - Very easy
        2 => Color32::from_rgb(140, 200, 60),  // Yellow-green - Easy  
        3 => Color32::from_rgb(200, 200, 60),  // Yellow - Moderate
        4 => Color32::from_rgb(230, 140, 50),  // Orange - Hard
        5 => Color32::from_rgb(230, 60, 60),   // Red - Extreme
        _ => Color32::GRAY,
    }
}

/// Render compact average rating display (read-only, no interaction)
/// Shows a single difficulty icon with label and vote count
fn render_compact_avg_rating(ui: &mut Ui, avg_rating: Option<u8>, rating_count: Option<i32>) {
    let Some(rating) = avg_rating else {
        return; // Don't show anything if no rating
    };
    
    // Add count in parentheses first (since we're right-to-left)
    if let Some(count) = rating_count {
        ui.label(RichText::new(format!("({})", count)).color(Color32::GRAY).size(10.0));
        ui.add_space(4.0);
    }
    
    // Add difficulty label with gradient color
    ui.label(RichText::new(difficulty_label(rating)).color(difficulty_color(rating)).size(10.0));
    ui.add_space(4.0);
    
    // Single difficulty icon
    ui.label(RichText::new(difficulty_icon(rating)).color(difficulty_color(rating)).size(12.0));
}
