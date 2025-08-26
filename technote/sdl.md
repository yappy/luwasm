# SDL

## バージョン

<https://emscripten.org/docs/compiling/Building-Projects.html#emscripten-ports>

ここにあるように、公式にポートしたライブラリは `emcc --use_port=<name>` で
利用できる。`emcc --show-ports` で利用可能なリストを得られる。

SDL2 はこの中にある。
しかし噂によるとリンクするだけで MB 単位のサイズになってしまうそうだ…。

SDL1 はビルトインで何もしなくても利用可能。
ヘッダも emsdk のインストール先に入っている。

## SDL 1.2

現在は従来の SDL2 or 最新の SDL3 という時代なので、SDL1 の資料を探すのが難しい。。
SDL2 にも引き続き存在する同名の関数は多く、その場合は大抵 SDL2 の情報が
出てきてしまう。

公式にあった 1.x 系の最新リリースのドキュメント: \
<https://www.libsdl.org/release/SDL-1.2.15/docs/html/index.html>

emsdk でインストールされる `SDL_version.h` によると `1.3.0` になっているが、
公式からは `1.2.15` が最終リリースのようだ。
apt には `1.2.12` があるようだ。
