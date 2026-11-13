#!/usr/bin/env bash
# bds-tool を ~/.local/bin に入れ、bash の Tab 補完を登録する。
#   ./install.sh              このスクリプトの隣、または target/release/ の bds-tool を入れる。
#                             無ければ、最新のリリースをダウンロードして入れる
#   ./install.sh <bds-tool>   指定した実行ファイルを入れる
# 何度実行してもよい(実行ファイルと補完を上書きする)。
set -euo pipefail

repo="takumin287877/bds-tool"
src="${1:-}"

# 手元の実行ファイルを探す(`curl ... | bash` で実行したときは探さない)
if [ -z "$src" ] && [ -f "${BASH_SOURCE[0]:-}" ]; then
  here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  for c in "$here/bds-tool" "$here/target/release/bds-tool"; do
    if [ -x "$c" ]; then src="$c"; break; fi
  done
fi

# 無ければ、最新のリリースをダウンロードする
if [ -z "$src" ]; then
  if ! command -v curl >/dev/null 2>&1; then
    echo "エラー: curl がありません。curl を入れるか、cargo build --release してから ./install.sh を実行してください" >&2
    exit 1
  fi
  echo "最新のリリースをダウンロードします"
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  base="https://github.com/$repo/releases/latest/download"
  curl -fsSL -o "$tmp/bds-tool" "$base/bds-tool" || { echo "エラー: ダウンロードに失敗しました: $base/bds-tool" >&2; exit 1; }
  curl -fsSL -o "$tmp/bds-tool.sha256" "$base/bds-tool.sha256" || { echo "エラー: チェックサムを取得できません" >&2; exit 1; }
  (cd "$tmp" && sha256sum -c bds-tool.sha256 >/dev/null) || { echo "エラー: チェックサムが一致しません" >&2; exit 1; }
  chmod +x "$tmp/bds-tool"
  src="$tmp/bds-tool"
fi

if [ ! -x "$src" ]; then
  echo "エラー: $src が見つからないか、実行できません" >&2
  exit 1
fi
if ! "$src" --version >/dev/null 2>&1; then
  echo "エラー: $src を実行できません(この環境に合わないファイルかもしれません)" >&2
  exit 1
fi

bin_dir="$HOME/.local/bin"
comp_dir="$HOME/.local/share/bash-completion/completions"
mkdir -p "$bin_dir" "$comp_dir"

# 動作中のファイルでも置き換えられるよう、別名で書いてから mv する
cp "$src" "$bin_dir/bds-tool.new"
chmod 755 "$bin_dir/bds-tool.new"
mv -f "$bin_dir/bds-tool.new" "$bin_dir/bds-tool"
"$bin_dir/bds-tool" completions bash > "$comp_dir/bds-tool.new"
mv -f "$comp_dir/bds-tool.new" "$comp_dir/bds-tool"

echo "インストールしました: $bin_dir/bds-tool"
echo "補完を登録しました:   $comp_dir/bds-tool"
case ":$PATH:" in
  *":$bin_dir:"*) ;;
  *)
    echo "注意: $bin_dir が PATH にありません。~/.bashrc に次を追加してください:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    ;;
esac
echo "新しく開いたシェルから、どこでも bds-tool が使えます。今のシェルでは次を実行してください:"
echo "  source $comp_dir/bds-tool"
