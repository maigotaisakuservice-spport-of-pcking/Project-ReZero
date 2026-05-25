# Project ReZero

**決定論的かつ可逆なコンピューティングの極致**

Project ReZeroは、ソフトウェアの信頼性を数学的に保証し、かつ実行状態を物理的に巻き戻すことを可能にする、次世代のネイティブコンパイラスタックおよびIDE環境です。

## コアコンセプト

### 1. 形式検証による「バグの不在」の証明
Z3 SMTソルバをコンパイラバックエンドに統合。配列境界外アクセス、ゼロ除算、NULLポインタ参照、およびユーザ定義の事前・事後条件をコンパイル時に数学的に証明します。

### 2. LRS（Linear Reversible Stack）による可逆実行
破壊的代入を物理的な「情報の損失」と捉え、代入直前に状態を履歴スタックへ退避します。これにより、OS上で直接動作するネイティブバイナリでありながら、任意の時点への実行巻き戻し（タイムトラベルデバッグ）を可能にします。

### 3. 物理的エントロピーの抑制
ランドアウアの原理（Landauer's principle）に基づき、情報の消去に伴う最小発熱量をリアルタイムでプロファイリング。計算資源の物理的な効率を極限まで高める設計をサポートします。

## ディレクトリ構成

- `rezero-core/`: Rust製ネイティブコンパイラ & 検証器
- `rezero-studio/`: VS Code拡張機能 (LSP / Webview)
- `rezero-vs-extension/`: Visual Studio (IDE) 拡張機能
- `rezero-standalone/`: Electron製スタンドアロンエディタ
- `docs/`: 日本語による法的・技術ドキュメント

## クイックスタート

1. **コンパイラのビルド**:
   ```bash
   cd rezero-core
   cargo build --release
   ```

2. **検証とコンパイル**:
   ```bash
   # 形式検証
   rezero-core verify program.rz
   # ネイティブバイナリの構築
   rezero-core build program.rz -o program.exe
   ```

## ライセンス

本ソフトウェアは「現状有姿」で提供されます。詳細は `docs/license.html` を参照してください。
