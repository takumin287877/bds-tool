use std::process::{Command, Stdio};

pub fn is_running() -> bool {
    Command::new("pgrep")
        .args(["-x", "bedrock_server"])
        .stdout(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// カレントプロセスを bedrock_server に置き換える(成功すると戻らない)
pub fn exec_server(dir: &std::path::Path) -> anyhow::Error {
    use std::os::unix::process::CommandExt;
    let dir = match dir.canonicalize() {
        Ok(d) => d,
        Err(e) => return e.into(),
    };
    let e = Command::new(dir.join("bedrock_server"))
        .current_dir(&dir)
        .env("LD_LIBRARY_PATH", ".")
        .exec();
    anyhow::anyhow!("サーバーを起動できません: {e}")
}
