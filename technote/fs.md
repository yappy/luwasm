# File System

<https://emscripten.org/docs/api_reference/Filesystem-API.html>

システムコールの先が JS で実装されているため、C/C++/Rust の標準ライブラリから
だいたいいい感じにアクセスできる。
ただ、`mount` がヘッダ中にあるようだったので呼んでみたらリンクエラーになってしまった。
無理に syscall I/F を使わず JS から叩いた方がよいかもしれない。
各関数は `Module.FS` に入っている。

<https://github.com/emscripten-core/emscripten/blob/main/src/lib/libfs.js>

JavaScript 実装はそのまま読めるので、何があるかや使い方の例などは
こちらを読んでしまうのがよい。
デフォルトのディレクトリ構成の生成やデバイスファイルの作成のやり方が書いてある。

`--clocure 1` をつけると名前が変わってしまって `Module.FS` にアクセスできなくなる。
`--pre-js` とかに適切にアノテートした JavaScript を置くと何とかなると
ドキュメントに書かれているが、よく分からない。
あと出力されるソースが読解不能になる。
