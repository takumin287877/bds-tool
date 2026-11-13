# bds-tool

公式 Bedrock Dedicated Server (Linux) を、新規インストール・更新するツール。

## クイックスタート
Linux (x86_64) の bash で、次の1行を実行すると、ツールが入ります。
```bash
curl -fsSL https://github.com/takumin287877/bds-tool/releases/latest/download/install.sh | bash
```
そのあと、新しく開いたシェルで次のように使います。
```bash
bds-tool install ~/mcserver --run    # サーバーを新規インストールして起動
bds-tool update ~/mcserver --run     # (サーバーを stop したあとで)更新して起動
```

## 1. ツールを入れる(1回だけ)
Linux (x86_64) の bash で、次を実行します。最新のリリースをダウンロードして、`~/.local/bin/bds-tool` に入れ、Tab 補完も登録します。
```bash
curl -fsSL https://github.com/takumin287877/bds-tool/releases/latest/download/install.sh | bash
```
- ダウンロードしたバイナリは、チェックサム(SHA-256)を確認してから入れます。
- `~/.local/bin` が PATH に無いときは、案内が出ます。
- `bds-tool` 自体は、ホームのファイルを書き換えません。書き換えるのは `install.sh` だけで、何度実行しても構いません(上書きします)。

ソースからビルドする場合(Rust が必要):
```bash
git clone https://github.com/takumin287877/bds-tool && cd bds-tool
cargo build --release
./install.sh        # target/release/bds-tool を入れる
```
実行ファイルだけを持っているときは、`./install.sh /path/to/bds-tool` で入れられます。

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

## ライセンス
[MIT](LICENSE)
