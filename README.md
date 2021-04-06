# rust_stdlinux

## 主旨

ふつうの Linux プログラミング本の C 言語コードを Rust で書いていく.

Rust でシステムコール, ライブラリ関数を呼び出すお勉強になるはず.

## ビルド方法

`Cargo.toml`に詳しく記載

```
cargo build --bin [拡張子を除いた任意のファイル名]
```

例

```
cargo build --bin mv
```

## 実行

```
./target/debug/[バイナリ名] [...適宜引数]
```

ビルド時に`--release`オプションを付けた場合

```
./target/release/[バイナリ名] [...適宜引数]
```

## ディレクトリ構成

```
├── Cargo.lock
├── Cargo.toml
├── README.md
└── src
   ├── 5
   │   ├── cat.rs
   │   └── wcl.rs
   ├── 6
   │   └── cat_buf.rs
   ├── 7
   │   ├── head.rs
   │   └── head_opt.rs
   ├── 8
   │   └── grep.rs
   ├── 10
   │   ├── chmod.rs
   │   ├── ln.rs
   │   ├── ls.rs
   │   ├── mkdir.rs
   │   ├── mv.rs
   │   ├── rm.rs
   │   ├── rmdir.rs
   │   ├── stat.rs
   │   └── symlink.rs
   └── main.rs(使わない)
```
