//! Persistent stats (best score, wins, games) in a tiny key=value text file.

use std::{fs, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stats {
    pub best: i32, // -1 = no game finished yet
    pub wins: u32,
    pub games: u32,
}

impl Default for Stats {
    fn default() -> Self {
        Self { best: -1, wins: 0, games: 0 }
    }
}

impl Stats {
    pub fn parse(text: &str) -> Self {
        let mut s = Stats::default();
        for line in text.lines() {
            if let Some((k, v)) = line.split_once('=') {
                match k.trim() {
                    "best" => s.best = v.trim().parse().unwrap_or(-1),
                    "wins" => s.wins = v.trim().parse().unwrap_or(0),
                    "games" => s.games = v.trim().parse().unwrap_or(0),
                    _ => {}
                }
            }
        }
        s
    }

    pub fn to_text(&self) -> String {
        format!("best={}\nwins={}\ngames={}\n", self.best, self.wins, self.games)
    }

    pub fn load() -> Self {
        fs::read_to_string(path()).map(|t| Self::parse(&t)).unwrap_or_default()
    }

    pub fn save(&self) {
        let p = path();
        if let Some(dir) = p.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let _ = fs::write(p, self.to_text()); // best effort
    }
}

#[cfg(target_os = "android")]
fn path() -> PathBuf {
    PathBuf::from("/data/data/com.example.shutthebox3d/files/shutbox.cfg")
}

#[cfg(not(target_os = "android"))]
fn path() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .or_else(|| std::env::var_os("APPDATA"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("shut-the-box-3d").join("shutbox.cfg")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let s = Stats { best: 3, wins: 2, games: 7 };
        assert_eq!(Stats::parse(&s.to_text()), s);
    }
}
