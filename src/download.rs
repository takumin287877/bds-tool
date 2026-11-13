use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::{fs::File, io, path::Path};

const API: &str = "https://net-secondary.web.minecraft-services.net/api/v1.0/download/links";
const UA: &str = "Mozilla/5.0 (X11; Linux x86_64) bds-update";

#[derive(Deserialize)]
struct Resp {
    result: Links,
}
#[derive(Deserialize)]
struct Links {
    links: Vec<Link>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Link {
    download_type: String,
    download_url: String,
}

/// 最新の (バージョン, URL) を返す
pub fn latest() -> Result<(String, String)> {
    let resp: Resp = ureq::get(API)
        .set("User-Agent", UA)
        .call()
        .context("ダウンロードAPIに接続できません")?
        .into_json()
        .context("APIの応答を解釈できません")?;
    let url = resp
        .result
        .links
        .into_iter()
        .find(|l| l.download_type == "serverBedrockLinux")
        .map(|l| l.download_url)
        .ok_or_else(|| anyhow!("Linux版のリンクが見つかりません"))?;
    let version = parse_version(&url).ok_or_else(|| anyhow!("URLからバージョンを取得できません: {url}"))?;
    Ok((version, url))
}

pub fn parse_version(url: &str) -> Option<String> {
    let name = url.rsplit('/').next()?;
    let v = name.strip_prefix("bedrock-server-")?.strip_suffix(".zip")?;
    (!v.is_empty()).then(|| v.to_string())
}

/// zip をファイルへストリーム保存する(サイズ上限なし)
pub fn fetch(url: &str, dest: &Path) -> Result<()> {
    let resp = ureq::get(url).set("User-Agent", UA).call().context("ダウンロードに失敗しました")?;
    let mut out = File::create(dest)?;
    io::copy(&mut resp.into_reader(), &mut out)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version() {
        let u = "https://www.minecraft.net/bedrockdedicatedserver/bin-linux/bedrock-server-1.26.63.1.zip";
        assert_eq!(parse_version(u).as_deref(), Some("1.26.63.1"));
        assert_eq!(parse_version("https://x/y.zip"), None);
    }
}
