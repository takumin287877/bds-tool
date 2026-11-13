use crate::{download, service};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Status {
    pub dir: PathBuf,
    pub installed: bool,
    pub current: Option<String>,
    pub latest: String,
    pub url: String,
    pub running: bool,
    pub port: Option<u16>,
}

pub fn gather(dir: &Path) -> Result<Status> {
    let (latest, url) = download::latest()?;
    let current = fs::read_to_string(dir.join(".bds_version")).ok().map(|s| s.trim().to_string());
    let port = fs::read_to_string(dir.join("server.properties")).ok().and_then(|s| parse_port(&s));
    Ok(Status {
        dir: dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf()),
        installed: dir.join("bedrock_server").is_file(),
        current,
        latest,
        url,
        running: service::is_running(),
        port,
    })
}

impl Status {
    pub fn up_to_date(&self) -> bool {
        self.current.as_deref() == Some(self.latest.as_str())
    }

    pub fn print(&self) {
        println!("サーバー: {}", self.dir.display());
        if !self.installed {
            println!("※ このディレクトリに bedrock_server がありません");
        }
        let cur = self.current.as_deref().unwrap_or("不明");
        let verdict = if self.up_to_date() { "最新です" } else { "更新があります" };
        let state = if self.running { "起動中" } else { "停止中" };
        let port = self.port.map_or("不明".to_string(), |p| p.to_string());
        println!("現在 {cur} / 最新 {} ({verdict})   状態: {state}   ポート: {port}", self.latest);
    }
}

pub fn parse_port(props: &str) -> Option<u16> {
    props
        .lines()
        .find_map(|l| l.trim().strip_prefix("server-port="))
        .and_then(|v| v.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn port() {
        let p = "# c\nserver-name=x\nserver-port=19136\nserver-portv6=19133\n";
        assert_eq!(parse_port(p), Some(19136));
        assert_eq!(parse_port("server-portv6=19133\n"), None);
        assert_eq!(parse_port("server-port=abc\n"), None);
    }
}
