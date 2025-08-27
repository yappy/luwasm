# Emscripten 情報

## WASM

Web Assembly.

コンパイラの出力した仮想の機械語をウェブブラウザで高速かつ安全に実行しようという試み。
~~Java Applet じゃん。~~
なお LLVM backend を頑張って作って LLVM 中間形式から WASM を吐けるようにしたことで、
ガチの C/C++ コードをポータブルでセキュアな実行形式に変換できるようになり、
ブラウザで実行するだけではもったいない気がしてきている模様。それはそう。

Wikipedia の例

```C
int factorial(int n) {
  if (n == 0)
    return 1;
  else
    return n * factorial(n-1);
}
```

アセンブリ言語相当 (S式なのが少々気になるが気にしてはいけない)

```text
(func (param i64) (result i64)
  local.get 0
  i64.eqz
  if (result i64)
      i64.const 1
  else
      local.get 0
      local.get 0
      i64.const 1
      i64.sub
      call 0
      i64.mul
  end)
```

(非実在) 機械語

```text
20 00
42 00
51
04 7e
42 01
05
20 00
20 00
42 01
7d
10 00
7e
0b
```

## アプローチ

* 仮想機械語バイナリを直接書く
* アセンブリ (S式) を直接書いてバイナリ形式にアセンブルする
* 既存の言語のコンパイラで出力ターゲットを wasm にする
  * LLVM backend ができており、C/C++ や Rust が使用可能 (←これは強い)

まあこれは一番下のコンパイラを使うものとして、ターゲットの機械語を吐けたとしても
実用的なプログラムが書けるところまでにはいくつもの壁がありがちである。

* 大規模な C/C++ プログラムには libc/libc++ のようなランタイムが必要
* libc には OS syscall 相当のレイヤが必要
* JavaScript/ブラウザオブジェクトへのアクセス
* ブラウザをターゲットにしない場合はランタイムの提供するシステムコール相当の
  インタフェース規格として WASI (WebAssembly System Interface) が提案されている

* 画像処理のみを切り出してアクセラレータとして活用する、のようなユースケースでは
  余分なものは邪魔だろう。WASM を吐けるコンパイラと最低限の JS-WASM 間バインディングが
  あればよい。
  * gcc/clang のターゲットで wasm を指定
  * Rust のターゲットで wasm32-unknown-unknown を指定
* 既存の大規模な C/C++ コードを移植するなら libc, posix 等のランタイムレイヤを
  自動でいい感じにやってくれないと大変すぎる
  * Emscripten
    * libc, syscall 部分を JS で自動生成してくれるため、printf とかする
      普通の C コードがそのまま動く
  * Rust のターゲットで wasm32-unknown-emcripten を指定
* Rust-wasm と JS 系エコシステムをいい感じに統合
  * wasm-pack
    * いろいろ雑多な処理をいい感じに一発でやってくれる系
    * C/C++ を入れると壊れがちかも…

## Emscripten 公式サイト

<https://emscripten.org/>

## インストール

`apt` にあるので、少々古いバージョンでもよいからとりあえずちょっと試してみたい場合は
そちらで。

```sh
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk

# Update sdk manager
git pull

# latest は最新安定バージョンのエイリアスになっている
# バージョン指定したい場合は latest の代わりに 1.38.45 のようにすれば OK
./emsdk install latest
./emsdk activate latest

# パスを通す (シェルごとに毎回必要)
source ./emsdk_env.sh
```

アンインストールは `git clean` とかリポジトリごと全部消すとかだけで完了。

## 基本的な使い方

`emcc` が `gcc` `clang` と同じ雰囲気で使えるフロントエンドコマンド。

```sh
# version
emcc -v
# help
emcc --help
```

以下のような libc が必要な C ソースファイルを用意して

```C
#include <stdio.h>

int main() {
  printf("hello!\n");
  return 0;
}
```

`gcc` `clang` と同じようにコンパイルしてやると

```sh
emcc hello.c
```

デフォルトだと `a.out.js` と `a.out.wasm` ができる。
`a.out.wasm` が WebAssembly 本体で、`a.out.js` は libc 等のランタイム、
WASM のロードと実行などを全部含めた JavaScript グルーコードである。

`a.out.js` はブラウザ環境と Node のコンソール環境に両対応しているようで、
HTML の `<script>` タグで読み込めば JavaScript のデバッグコンソールに出力されるし、
`node` に渡せば C コードそのままの実行結果が得られる。

```sh
$ node a.out.js
hello
```

基本的なオプションはおそらく gcc, clang とだいたい同じのはず。

## 出力フォーマット

出力ファイル名は gcc 系統と同じく `-o <FILENAME>` で変更できる。
この時、拡張子を特定のものにすることで出力方式が自動で変化する。

```sh
emcc hello.c -o hello.html
```

`*.html` を指定すると `*.js` を読んで画面に表示するテスト用 HTML が
(`*.js`, `*.wasm` とともに) 出力される。
HTML からロードするやり方の参考にもなる。

多くのブラウザはローカルファイルに対して XHR (XMLHttpReqeust) できず
読み込めないことが多い。
その場合は web server を立てる必要がある。
`python3 -m http.server` を起動し、出力された URL をブラウザで開くのが
簡単でおすすめ。

より詳細なあれこれは以下。\
<https://emscripten.org/docs/compiling/Building-Projects.html>

* `*.wasm`
  * WASM バイナリ。
* `*.js`
  * JavaScript グルーコード。
  * libc や syscall 相当層。
  * WASM のロードと実行コード。
* `*.html`
  * `*.js` コードの使い方サンプル兼テスト UI HTML。

## Module object

<https://emscripten.org/docs/api_reference/module.html>

`-o *.html` で出力される HTML を読んだ方が早いかもしれない。

`Module` という名前のグローバルオブジェクトを定義しておくと
WASM コードの実行環境をカスタマイズすることができる。
`*.js` では `Module` という名前のグローバル変数は定義しておらず、
存在しているか確認して存在するならばそれを使うようなコードになっている。

分かりやすいところで言うと stdout, stderr への出力を自前の関数に置き換えられる。

```js
var Module = {
  'print': function(text) { alert('stdout: ' + text) },
  'printErr': function(text) { alert('stderr: ' + text) }
};
```

HTML 中の `<script>` タグで `Module` をグローバルに定義した後、
`<script async type="text/javascript" src="hello.js"></script>` のように
emcc の吐いた `*js` ファイルを読み込めば OK。

### Module.canvas

ドキュメントには書かれていないが自動出力される HTML や canvas 無しで実行した時の
スタックトレース等から、SDL を使うと `Module.canvas` が参照される
(そこに canvas オブジェクトをあらかじめ設定しておく必要がある)。
