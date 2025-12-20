//! Steam image URL proxying helpers for CORS avoidance

/// Convert Steam CDN URLs to proxied URLs to avoid CORS issues
/// Handles both steamcdn-a.akamaihd.net and media.steampowered.com URLs
pub fn proxy_steam_image_url(url: &str) -> String {
    // Get the current origin for relative URLs
    let origin = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default();
    
    if url.contains("steamcdn-a.akamaihd.net") {
        // https://steamcdn-a.akamaihd.net/steamcommunity/public/images/apps/...
        // -> /steam-media/steamcommunity/public/images/apps/...
        if let Some(path) = url.strip_prefix("https://steamcdn-a.akamaihd.net/") {
            return format!("{}/steam-media/{}", origin, path);
        }
        if let Some(path) = url.strip_prefix("http://steamcdn-a.akamaihd.net/") {
            return format!("{}/steam-media/{}", origin, path);
        }
    }
    
    if url.contains("media.steampowered.com") {
        // https://media.steampowered.com/steamcommunity/public/images/apps/...
        // -> /steam-media/steamcommunity/public/images/apps/...
        if let Some(path) = url.strip_prefix("https://media.steampowered.com/") {
            return format!("{}/steam-media/{}", origin, path);
        }
        if let Some(path) = url.strip_prefix("http://media.steampowered.com/") {
            return format!("{}/steam-media/{}", origin, path);
        }
    }
    
    // Return original URL if not a Steam CDN URL
    url.to_string()
}

/// Build a game icon URL using the proxy
/// Game icons are at: media.steampowered.com/steamcommunity/public/images/apps/{appid}/{hash}.jpg
pub fn game_icon_url(appid: u64, icon_hash: &str) -> String {
    let origin = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default();
    // Use steam-media proxy which routes to steamcdn-a.akamaihd.net
    format!("{}/steam-media/steamcommunity/public/images/apps/{}/{}.jpg", origin, appid, icon_hash)
}
