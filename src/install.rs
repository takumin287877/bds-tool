use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path};

const KEEP_FILES: &[&str] = &["server.properties", "allowlist.json", "whitelist.json", "permissions.json"];
const KEEP_DIRS: &[&str] = &["worlds", "config"];

/// 既にあるとき上書きしないファイルか(初回インストールでは配置される)
pub fn is_preserved(rel: &Path) -> bool {
    let mut comps = rel.components();
    let first = match comps.next() {
        Some(Component::Normal(s)) => s.to_string_lossy().into_owned(),
        _ => return false,
    };
    if comps.next().is_none() {
        KEEP_FILES.contains(&first.as_str())
    } else {
        KEEP_DIRS.contains(&first.as_str())
    }
}

pub fn extract(zip_path: &Path, dest: &Path) -> Result<()> {
    let mut zip = zip::ZipArchive::new(File::open(zip_path)?).context("zip を開けません")?;
    for i in 0..zip.len() {
        let mut e = zip.by_index(i)?;
        let Some(rel) = e.enclosed_name() else { continue };
        let out = dest.join(rel);
        if e.is_dir() {
            fs::create_dir_all(&out)?;
        } else {
            if let Some(p) = out.parent() {
                fs::create_dir_all(p)?;
            }
            io::copy(&mut e, &mut File::create(&out)?)?;
        }
    }
    Ok(())
}

/// src の内容を dst へコピーする。削除はしない。
pub fn install(src: &Path, dst: &Path) -> Result<()> {
    walk(src, src, dst)
}

fn walk(root: &Path, cur: &Path, dst: &Path) -> Result<()> {
    for entry in fs::read_dir(cur)? {
        let path = entry?.path();
        let rel = path.strip_prefix(root)?;
        let target = dst.join(rel);
        if path.is_dir() {
            fs::create_dir_all(&target)?;
            walk(root, &path, dst)?;
        } else if is_preserved(rel) && target.exists() {
            continue;
        } else {
            // 実行中バイナリでも置き換えられるよう、tmp に書いて rename する
            let mut tmp = target.clone().into_os_string();
            tmp.push(".tmp");
            let tmp = Path::new(&tmp);
            fs::copy(&path, tmp).with_context(|| format!("コピー失敗: {}", rel.display()))?;
            let mode = if rel == Path::new("bedrock_server") { 0o755 } else { 0o644 };
            fs::set_permissions(tmp, fs::Permissions::from_mode(mode))?;
            fs::rename(tmp, &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserved() {
        assert!(is_preserved(Path::new("server.properties")));
        assert!(is_preserved(Path::new("worlds/a/level.dat")));
        assert!(is_preserved(Path::new("config/x/y.json")));
        assert!(!is_preserved(Path::new("bedrock_server")));
        assert!(!is_preserved(Path::new("behavior_packs/a/manifest.json")));
        assert!(!is_preserved(Path::new("a/server.properties")));
    }

    #[test]
    fn install_keeps_existing_and_adds_new() {
        let t = tempfile::tempdir().unwrap();
        let (s, d) = (t.path().join("s"), t.path().join("d"));
        fs::create_dir_all(s.join("config")).unwrap();
        fs::create_dir_all(&d).unwrap();
        fs::write(s.join("server.properties"), "new").unwrap();
        fs::write(s.join("bedrock_server"), "new").unwrap();
        fs::write(s.join("config/default.json"), "new").unwrap();
        fs::write(d.join("server.properties"), "mine").unwrap();
        fs::write(d.join("bedrock_server"), "old").unwrap();
        install(&s, &d).unwrap();
        assert_eq!(fs::read_to_string(d.join("server.properties")).unwrap(), "mine");
        assert_eq!(fs::read_to_string(d.join("bedrock_server")).unwrap(), "new");
        assert_eq!(fs::read_to_string(d.join("config/default.json")).unwrap(), "new");
    }
}
