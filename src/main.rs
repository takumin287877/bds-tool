mod download;
mod install;
mod service;
mod status;

use anyhow::{bail, Context, Result};
use clap::{CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::Shell;
use std::path::{Path, PathBuf};
use std::fs;

const EXAMPLES: &str = "\
使用例:
  bds-tool install ~/server             サーバーを新規インストール(--run で起動まで)
  bds-tool status ~/server              版・状態・ポートを表示するだけ
  bds-tool update ~/server              更新だけ
  bds-tool update ~/server --run        更新してからサーバーを起動

更新の前に、サーバーは `stop` で停止しておいてください。
worlds / config / server.properties / allowlist.json / permissions.json は上書きしません。";

/// 統合版(Bedrock Dedicated Server)の更新ツール
#[derive(Parser)]
#[command(name = "bds-tool", version, after_help = EXAMPLES, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// 現在の版・最新版・起動状態・ポートを表示する
    Status {
        /// サーバーのディレクトリ(省略時はカレント)
        #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
        path: PathBuf,
    },
    /// 最新版へ更新する
    Update {
        /// サーバーのディレクトリ(省略時はカレント)
        #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
        path: PathBuf,
        /// 同じ版でも再適用する
        #[arg(long)]
        force: bool,
        /// 更新後にサーバーを起動する
        #[arg(short, long)]
        run: bool,
    },
    /// サーバーを新規インストールする(最新版を指定ディレクトリに展開)
    Install {
        /// インストール先のディレクトリ(省略時はカレント)
        #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
        path: PathBuf,
        /// インストール後にサーバーを起動する
        #[arg(short, long)]
        run: bool,
    },
    /// 補完スクリプトを標準出力に出す(`install.sh` が使う)
    Completions {
        /// シェルの種類
        #[arg(default_value = "bash")]
        shell: Shell,
    },
}

/// 補完スクリプトを作る。
/// clap_complete は、ハイフン入りのコマンド名で bash の case ラベルを
/// `bds__subcmd__tool__…` と誤って出すため、関数側の名前 `bds__tool__…` に直す。
fn completion_script(shell: Shell) -> String {
    let mut buf = Vec::new();
    clap_complete::generate(shell, &mut Cli::command(), "bds-tool", &mut buf);
    String::from_utf8_lossy(&buf).replace("bds__subcmd__tool__subcmd__", "bds__tool__subcmd__")
}

fn completions_cmd(shell: Shell) -> ! {
    print!("{}", completion_script(shell));
    std::process::exit(0) // スクリプトの末尾に空行を足さない
}

/// サーバーを新規インストールする。run が true なら、そのまま起動する。
fn install_server_cmd(dir: &Path, run: bool) -> Result<()> {
    if dir.join("bedrock_server").exists() {
        bail!(
            "{} には既に bedrock_server があります。更新は `bds-tool update {}` を使ってください",
            dir.display(),
            dir.display()
        );
    }
    if run && service::is_running() {
        bail!("bedrock_server が起動中です。サーバーで `stop` を入力して停止してから、もう一度実行してください");
    }
    let (version, url) = download::latest()?;
    println!("→ {} に最新版 {version} を新規インストールします", dir.display());
    apply(dir, &version, &url)?;
    if run {
        println!("→ サーバーを起動します(止めるときは、この画面で `stop` と入力)");
        return Err(service::exec_server(dir));
    }
    println!("起動するには、そのディレクトリで `bds-tool update --run` を実行してください");
    Ok(())
}

fn main() {
    let result = real_main();
    // 終了後にシェルのプロンプトが出力にくっつかないよう、空行を入れる
    // (サーバーを起動した場合は、プロセスが置き換わるのでここには来ない)
    match result {
        Ok(()) => println!(),
        Err(e) => {
            eprintln!("エラー: {e:#}\n");
            std::process::exit(1);
        }
    }
}

fn real_main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Status { path } => {
            status::gather(&path)?.print();
            Ok(())
        }
        Cmd::Update { path, force, run } => update_cmd(&path, force, run),
        Cmd::Install { path, run } => install_server_cmd(&path, run),
        Cmd::Completions { shell } => completions_cmd(shell),
    }
}

/// 更新する。run が true なら、そのままこのプロセスでサーバーを起動する。
fn update_cmd(dir: &Path, force: bool, run: bool) -> Result<()> {
    let st = status::gather(dir)?;
    st.print();
    if st.running {
        bail!("bedrock_server が起動中です。サーバーで `stop` を入力して停止してから、もう一度実行してください");
    }
    if !st.installed {
        bail!(
            "{} に bedrock_server がありません。サーバーの新規インストールは `bds-tool install {}` を使ってください",
            st.dir.display(),
            dir.display()
        );
    }

    println!();
    let need = force || !st.up_to_date();
    if need {
        println!("→ {} をダウンロードして上書きします(worlds / config / server.properties / allowlist.json / permissions.json は残します)", st.latest);
    } else {
        println!("→ 最新です。更新は不要です");
    }
    if run {
        let prefix = if need { "更新が終わったら、" } else { "" };
        println!("→ {prefix}サーバーを起動します");
    }
    if need {
        apply(dir, &st.latest, &st.url)?;
    }
    if run {
        return Err(service::exec_server(dir));
    }
    Ok(())
}

fn apply(dir: &Path, version: &str, url: &str) -> Result<()> {
    fs::create_dir_all(dir)?;
    let tmp = tempfile::tempdir().context("一時ディレクトリを作れません")?;
    let zip_path = tmp.path().join("bds.zip");
    let extracted = tmp.path().join("extracted");
    println!("ダウンロード中: {url}");
    download::fetch(url, &zip_path)?;
    install::extract(&zip_path, &extracted)?;
    install::install(&extracted, dir)?;
    fs::write(dir.join(".bds_version"), format!("{version}\n"))?;
    println!("{version} への更新が完了しました\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_completion_labels_match_function_names() {
        let s = completion_script(Shell::Bash);
        assert!(s.contains("cmd=\"bds__tool__subcmd__update\""));
        assert!(s.contains("bds__tool__subcmd__update)"));
        assert!(!s.contains("bds__subcmd__tool"));
    }
}
