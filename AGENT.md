## AGENT TODOs — 非カストディ型 決済基盤 (ICP / ICRC‑2)

本書は実装タスクの切り分け（MVP → 拡張）を示す。原則として小さく動く単位で着地し、各タスクは明確な受け入れ条件を持つ。

### ガードレール（共通方針）
- 非カストディ: Canister はユーザー資産を保有せず、移動は ICRC‑2 の `approve`/`transfer_from` のみで実行
- Idempotency 徹底: すべての変異 API は冪等キーとイベント記録を持つ
- 認証/認可: マーチャント Principal 単位でリソースを強制的に分離
- 可観測性: 主要イベントは Certified Data 経由で検証可能にし、ページングで取得
- アップグレード安全: Stable Memory での移行計画とバージョン管理

---

### Decision Log（合意事項 / MVP 前提）
- 言語: Rust（Canister 実装/テストともに Rust 前提）
- Capture 方針（MVP）: 単発フルキャプチャのみ対応（部分/複数キャプチャは将来）
  - 状態機械を簡素化（`requires_approval → succeeded → (released|refunded|expired)`）
  - `captured_amount` は `amount` に一致することを前提。部分一致はエラー
- 金額の意味: `amount` は「支払者から escrow へ移す額（グロス）」
  - ICRC‑2 の `transfer_from` は送金手数料を支払者口座から別途徴収（escrow 受取額は `amount`）
  - Release 手数料は escrow 残高から控除され、マーチャント実受取は `amount - release_fee` 相当
  - 将来オプション: マーチャント純受取額を保証するグロスアップ（`ensure_merchant_net`）
- 手数料ポリシー（MVP）: プラットフォーム手数料 0、ネットワーク手数料はユーザー資産由来で賄う
  - 経済帰属の注意: capture 手数料=支払者負担、release 手数料=escrow から控除（結果的にマーチャント純受取が減少）
  - SDK/ドキュメントで「マーチャント実受取は release 手数料分だけ減る」旨を明記
- Webhook Dispatcher / Merchant Registry: MVP 範囲外（将来タスク）。MVP はイベント記録 + SDK 検証関数を優先
- ローカル開発/テスト: PocketIC で ICRC‑2 準拠 Ledger を模擬（ckUSDC/ckBTC 相当はモック Ledger を使用）
  - 本番 Principal は Ledger Registry で切替。local はリファレンス ICRC‑1/2 Ledger Wasm か軽量モックをデプロイ

### Milestone 0 — プロジェクト初期化
- [ ] リポジトリ整備: ライセンス, README, Contributing, Issue/PR テンプレート
  - 受け入れ: OSS ライセンス選定（例: Apache‑2.0）、最小 README 掲載
- [ ] dfx ワークスペース/ツールチェーン整備（Rust 前提）
  - 受け入れ: `cargo build`/`dfx start`/`dfx deploy` のローカル手順が README に明記
- [ ] CI（lint/format/test/build）と最低限の release draft
  - 受け入れ: main ブランチに対して CI がグリーンである

---

### Milestone 1 — コア型/状態モデルの確立（Payments Canister 骨格）
- [ ] 型定義: `PaymentIntent`, `IntentStatus`, `LedgerInfo`, `AssetId`, `Account`, `Split`
  - 受け入れ: 単体テストで Serialize/Deserialize が通る
- [ ] Ledger Registry 実装: ICRC‑1/2 準拠 Ledger の Principal/decimals を保持
  - 受け入れ: `register_ledger`, `get_ledger(asset)` が動作
- [ ] Intent ID/サブアカウント設計: `intent_id` から deterministic に escrow subaccount を導出（ハッシュ + salt）
  - 受け入れ: 同じ入力→同じ出力、衝突テスト
- [ ] Event Log 下準備: 追加専用ログ + インデックス + Certified Data ルート更新
  - 受け入れ: `append_event` でルートが更新され、`query_events` がページング動作

---

### Milestone 2 — 基本フロー(MVP): create → approve → capture
- [ ] `create_intent(amount, asset, merchant, expires_at, metadata)`
  - 受け入れ: 状態 `requires_approval` で作成。escrow アカウント返却
- [ ] `capture(intent_id)` 実装（ICRC‑2 `transfer_from` 呼び出し）
  - 受け入れ: allowance/期限/金額検証、成功で `succeeded`、イベント記録、冪等
- [ ] 失敗系: allowance 不足/期限切れ/資産不一致/小数精度の拒否
  - 受け入れ: エラー型が型安全に表現される
- [ ] SDK（TS）最小: `createPaymentIntent` / `capture` クライアント
  - 受け入れ: Node/Browser から dfx ローカルに対して E2E で動く
- [ ] Hosted Checkout 最小: ウォレット接続 → approve → capture ボタン
  - 受け入れ: ローカル ckUSDC モックで 1 決済が成功

---

### Milestone 3 — release / refund / expire
- [ ] `release(intent_id, splits)` 実装（escrow → 受益者へ送金）
  - 受け入れ: 合計が捕捉残高以下、複数受益者 split、残高/手数料更新
- [ ] `refund(intent_id, amount)` 実装（部分/全額）
  - 受け入れ: 支払者への返金が成功、残高・状態・イベントが整合
- [ ] 期限切れ処理: `expires_at` に基づく lazy もしくは periodic expire
  - 受け入れ: アクセス時に自動遷移、イベント発行
- [ ] Webhook 仕様策定（署名, 再送, バックオフ）と Dispatcher スタブ
  - 受け入れ: 署名検証ロジック（SDKの `verifyWebhook`）を含む

---

### Milestone 4 — 開発者体験/可観測性
- [ ] エラー体系/コードの整理（API/SDK 共通）
  - 受け入れ: ドキュメント化 + 型で表現
- [ ] イベントの Certified Query 公開と SDK からの取得
  - 受け入れ: 順序保証/ページング/カーソル動作が確認できる
- [ ] ダッシュボード最小: Intent/イベント一覧、Webhook ログ表示
  - 受け入れ: dfx ローカルで一覧/詳細が閲覧可能

---

### Milestone 5 — セキュリティ/運用
- [ ] アクセス制御: マーチャント単位の認可境界（本人以外の操作拒否）
- [ ] リプレイ/二重実行対策: 冪等キー + 状態遷移ガード + nonce
- [ ] レート制限/DoS 緩和（必要に応じて）
- [ ] 料金/ガス設計: サービス手数料/手数料の徴収と会計
- [ ] アップグレード手順とバックアップ/復旧手順のドキュメント

---

### SDK（TypeScript）具体タスク
- [ ] Canister インターフェイス定義（IDL）と型生成
- [ ] クライアント: `createPaymentIntent`, `capture`, `release`, `refund`, `getIntent`, `listEvents`
- [ ] `verifyWebhook`（HMAC 署名 + 時刻ずれ許容 + 署名ヘッダ）
- [ ] リトライ/冪等化/エラー分類ユーティリティ
- [ ] サンプル（Node/Next.js）

---

### Hosted Checkout（Next.js）具体タスク
- [ ] 決済リンク/QR 発行ページ
- [ ] ウォレット接続（ICRC 対応）と approve UI
- [ ] capture 実行、成功/失敗画面、リダイレクト URL
- [ ] 意図的な異常系（期限切れ/不足）の UX

---

### テスト戦略
- [ ] 単体: 型/計算/サブアカウント導出/金額・桁検証
- [ ] 結合: dfx + モック ICRC‑2 Ledger で end‑to‑end（approve/capture/release/refund/expire）
- [ ] プロパティテスト: 分配合計 ≤ 捕捉残高、状態遷移の安全性
- [ ] 回帰: 既知不具合に対する再現テスト

---

### ドキュメント/サンプル
- [ ] API リファレンス（メソッド, パラメータ, エラー, 例）
- [ ] クイックスタート（ローカルで 1 取引）
- [ ] セキュリティ/スレッドモデル/制限事項
- [ ] 移行方針（Stable Memory schema 変更手順）

---

### オープンな設計課題 / 未確定事項
- [ ] 言語選定の最終確定（Rust 想定／Motoko 可）
- [ ] `approve` 期限と `expires_at` の関係（どちらを上限とするか）
- [ ] 小数処理: assets ごとの `decimals` と端数丸め/最小単位
- [ ] 返金ポリシー: 手数料の負担主体（プラットフォーム/マーチャント）
- [ ] 分配（Connect 風）での手数料順序（gross → fee → splits / 逆）
- [ ] ckBTC の最小額/手数料と UX（サトシ単位）
- [ ] イベント保持方針: 保持期間/圧縮/スナップショット
- [ ] Webhook 配信の at‑least‑once と順序保証のバランス

---

### 受け入れ基準（MVP 完了）
- `create → approve → capture → release/refund → events` がローカルでエンドツーエンド動作
- SDK と Hosted Checkout の最小フローで 1 取引が成功し、イベント/署名検証が行える
- 主要な失敗系がテストでカバーされる（期限切れ、許容量不足、資産不一致）

---

### ロードマップ（将来）
- Merchant Registry Canister（KYC/手数料率/Webhook 秘密管理）
- Webhook Dispatcher（再送/バックオフ/署名キー輪番）
- ダッシュボード拡充（CSV、絞り込み、エクスポート）
- マルチテナント運用（隔離/課金/レート制限）
- 監査ログ/可視化、SLA/ステータスページ
