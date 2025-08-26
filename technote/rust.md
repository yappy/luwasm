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

## mlua

Bindgen と unsafe で Lua C API を呼んで気合で書くのもいいが、
さすがにやってる人がいるのでラッパライブラリを使う。

<https://github.com/mlua-rs/mlua>

```sh
cargo add mlua
```

ビルドするとどれを使うか1つだけ選べとエラーが出る。
luajit は速そうだが、どう考えても wasm で JIT は無茶なので、lua54 を選ぶ。
(なんか JS から wasm バイナリをロードして呼ぶのは標準的な方法なので、
もしかしたら理論上はできるのかもしれない。
なお適当に luajit をぶっぱしてみたら案の定死んだ模様)

```text
compile_error!("You can enable only one of the features: lua54, lua53, lua52, lua51, luajit, luajit52, luau")
```

ビルドすると wasm32 なら `vendored` feature を有効にしろと言われる。
これは Lua をソースからビルドするフラグらしい。

```text
  thread 'main' panicked at /home/yappy/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/mlua-sys-0.8.3/build/find_normal.rs:10:9:
  Please enable `vendored` feature to build for wasm32
```

なお Emscripten を切って `wasm32-unknown-unknown` をぶっぱなしたら死んだ
(libc なしはキツすぎるのでそれはそう)。
Emscripten でのビルドを個別にサポートしてくれているという感じがある。

```text
  thread 'main' panicked at /home/yappy/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lua-src-548.1.1/src/lib.rs:98:25:
  don't know how to build Lua for wasm32-unknown-unknown
```

その他、`anyhow` や `serde` 等、便利そうなフィーチャーが
デフォルト OFF で用意されている。

## トラブルシューティング

### bindgen で関数が無視される

<https://github.com/rust-lang/rust-bindgen/issues/751#issuecomment-496891269>

target = wasm32 の場合、デフォルトが `fvisibility=hidden` になるのが原因らしい。
これは elf 中のシンボルに設定できる可視属性で、
最近 (いつ？) は大量のシンボルのせいで解決が遅くなるのを防ぐため、
特に共有ライブラリでは推奨されているらしい。

これのせいで bindgen を build.rs から呼んだ場合
(build target = wasm32 を引き継ぐ) と
bindgen-cli をコマンドラインで clang-args (`--` の後) にターゲットを wasm32 で
指定した場合のみ関数が処理されず無視されてしまう。
bindgen は libclang を使用しておりその時点で hidden 属性がついてしまうらしい。

`bindgen header.h -- -target wasm32-unknown-emscripten`

解決方法としては `-fvisibility=default` を bindgen 時に clang 引数として渡す。

~~やめてくれ～~~
