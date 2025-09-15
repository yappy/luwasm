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

`SDL_image` 等のサブプロジェクトはそこにはなく、こちらにあった。\
<https://www.libsdl.org/projects/old/>\
リンクからは 2.0 に飛ぶが、そこに `SDL_image 1.2` のようなリンクが存在する。
素の SDL では jpeg, png に対応しておらず Windows bmp のみなので
`SDL_image` は欲しいだろう。
結局はバックエンドが JavaScript 実装なので、bmp ロード関数でブラウザが対応している
画像形式全部が読み込めてもおかしくないけど。。

SDL(1) のヘッダは
`emsdk/upstream/emscripten/cache/sysroot/include/SDL`
にインストールされる。

サンプルは
`emsdk/upstream/emscripten/test/browser`
にインストールされる。

しかしなんだか公式 `1.2.15` と微妙に構造体定義が異なる箇所がある気がする。
また、`SDL_image` 等、SDL を冠するが別メンテのライブラリも同じディレクトリに
入っているがバージョンが違う。

## トラブルシューティング

### SDL_init() を呼ぶとキーボードショートカットが効かなくなる

ブラウザの F5 更新や F12 開発機能呼び出しが効かなくなってつらい。

SDL 固有の `Module` JS インタフェースはドキュメントに記載がないので、
`reference/sdl.c` (チュートリアルの SDL サンプル) をビルドして `sdl.js` を
読むのが一番よい。

```js
var Browser = {
  ...
  getCanvas:() => Module['canvas'],
  ...
};
```

例えばこのようにグルーコードからは `Module['canvas']` を描画先として
参照していることが分かる。
`Module['` で検索すれば設定項目名とその使われ方が分かる。

```js
/** @param{number} initFlags */
var _SDL_Init = (initFlags) => {
  SDL.startTime = Date.now();
  SDL.initFlags = initFlags;

  // capture all key events. we just keep down and up, but also capture press to prevent default actions
  if (!Module['doNotCaptureKeyboard']) {
    var keyboardListeningElement = Module['keyboardListeningElement'] || document;
    keyboardListeningElement.addEventListener("keydown", SDL.receiveEvent);
    keyboardListeningElement.addEventListener("keyup", SDL.receiveEvent);
    keyboardListeningElement.addEventListener("keypress", SDL.receiveEvent);
    window.addEventListener("focus", SDL.receiveEvent);
    window.addEventListener("blur", SDL.receiveEvent);
    document.addEventListener("visibilitychange", SDL.receiveEvent);
  }
  ...
}
```

* `doNotCaptureKeyboard`
  * キーボードのキャプチャをやめさせることができる。
* `keyboardListeningElement`
  * これのデフォルト値が document になっているのが原因。
  * canvas にすると、そこをクリックしないとキーイベントが配送されない。
    完成版ゲームでは document の方がいいのかもしれない。

`Module` オブジェクトの初期化は HTML 出力を参考にするか、
`--pre-js <jsfile>` オプションをコンパイラ (リンカ) に渡す。

## SDL_ttf

`emsdk/upstream/emscripten/cache/sysroot/include/SDL` に `SDL_ttf.h` がある。
バージョンは 2.0.11 と書いてあり、本家が SDL2 時代に書かれたものっぽいが、
内部実装は JavaScript Canvas で drawString するだけだろうし
SDL1 の中に混ざっていることに関してはあまり深く考えてはいけない。

<https://www.libsdl.org/projects/old/SDL_ttf/docs/index.html>

### OpenFont

本来はファイルシステム上のフォントファイルを読み込むが、
生成される js コードを読むと、name パラメータはフォントファイルパスではなく、
代わりに HTML/JavaScript システムでの font-family 指定に使用されるようだ。

固有のフォント名でない総称ファミリ名の一覧

* serif
  * 明朝体
* sans-serif
  * ゴシック体
* cursive
  * 筆記体
* fantasy
  * かわいい系？
* monospace
  * 等幅フォント

```JS
var _TTF_OpenFont = (name, size) => {
    name = PATH.normalize(UTF8ToString(name));
    var id = SDL.fonts.length;
    SDL.fonts.push({
      name, // but we don't actually do anything with it..
      size
    });
    return id;
  };

var _TTF_CloseFont = (font) => {
    SDL.fonts[font] = null;
  };

makeFontString(height, fontName) {
      if (fontName.charAt(0) != "'" && fontName.charAt(0) != '"') {
        // https://developer.mozilla.org/ru/docs/Web/CSS/font-family
        // Font family names containing whitespace should be quoted.
        // BTW, quote all font names is easier than searching spaces
        fontName = '"' + fontName + '"';
      }
      return height + 'px ' + fontName + ', serif';
    },
```
