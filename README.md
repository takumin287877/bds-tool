# bds-tool

公式 Bedrock Dedicated Server (Linux) を、新規インストール・更新するツール。

## 1. ツールを入れる(1回だけ)
```bash
cargo build --release     # Rust が必要
./install.sh              # ~/.local/bin/bds-tool に入れ、bash の Tab 補完も登録する
```
- `install.sh` は、ツールを入れるときだけ使います。何度実行しても構いません(上書きします)。
- 実行ファイルだけを持っているときは、`./install.sh /path/to/bds-tool` で入れられます。
- `~/.local/bin` が PATH に無いときは、案内が出ます。
- `bds-tool` 自体は、ホームのファイルを書き換えません。

## 2. サーバーを入れる
```bash
bds-tool install ~/mcserver          # 最新版を新規インストール
bds-tool install ~/mcserver --run    # インストールして、そのまま起動 (-r)
```
`bedrock_server` が既にある場所では止まります(更新は `update`)。

## 3. 更新する
サーバーは `stop` で停止してから実行します(起動中はエラーで止まります)。
```bash
bds-tool update [PATH]          # 更新だけ
bds-tool update [PATH] --run    # 更新してからサーバーを起動 (-r)
bds-tool update [PATH] --force  # 同じ版でも再適用
bds-tool status [PATH]          # 版・起動状態・ポートを表示するだけ
```
- `PATH` はサーバーのディレクトリ(省略するとカレント)。`bedrock_server` が無い場所では `update` は止まります。
- `worlds/`、`config/`、`server.properties`、`allowlist.json`、`permissions.json` は、既にあれば上書きしません。他のファイルは上書きのみで、削除はしません。
- `--run` は、このプロセスを `bedrock_server` に置き換えて起動します(tmux のペインでそのまま動きます)。
- バージョンは `<PATH>/.bds_version` に記録します。

## Tab 補完
`bds-tool up<Tab>` → `update`、`--<Tab>` → `--run` / `--force`、パスはディレクトリ名を補完します。補完スクリプトは `bds-tool completions bash` で出力できます(zsh / fish も可)。
