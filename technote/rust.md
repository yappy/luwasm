# Rust - Emscripten

## build target の追加

ビルドターゲットの追加は以下。

```sh
rustup target add wasm32-unknown-emscripten
```

Pure Rust でアクセラレータを書きたいだけなら libc 等のグルーコードは
トラブルを起こしやすいだけで邪魔かもしれない。
C/C++ 資産を使いたいわけではないなら以下の方がよいかもしれない。

```sh
rustup target add wasm32-unknown-unknown
```

~~最近のクロスコンパイルは簡単でいかん。~~

## Cargo 設定ファイル

<https://doc.rust-lang.org/cargo/reference/config.html>

`.cargo/config.toml` を作成する。

ビルドターゲットは `cargo run --target wasm32-unknown-emscripten` のように
指定できるらしいが、通常面倒すぎるので `config.toml` に以下のように書くと
デフォルトを変更できる。

```toml
[build]
target = "wasm32-unknown-emscripten"
```

環境変数を設定できるので、Emscripten の設定に有効かもしれない。

```toml
[env]
# Set ENV_VAR_NAME=value for any process run by Cargo
ENV_VAR_NAME = "value"
# Set even if already present in environment
ENV_VAR_NAME_2 = { value = "value", force = true }
# `value` is relative to the parent of `.cargo/config.toml`, env var will be the full absolute path
ENV_VAR_NAME_3 = { value = "relative/path", relative = true }
```

クロスビルド結果は通常そのまま実行できない。
`target.<triple>` で `runner` を設定するとよい。

そのまま実行しようとすると `*.js` を実行しようとして死ぬので、
`node` (emsdk でパスが通る) に渡せばコンソールアプリのように実行できる。

```toml
[target.wasm32-unknown-emscripten]
runner = "node"
```
