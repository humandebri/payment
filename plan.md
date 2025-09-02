# 決済基盤実装計画書（非カストディ / ICP / ICRC-2 対応）

## 概要
本計画は、ICP 上で **非カストディ型の決済基盤**を構築するものである。  
対象資産は **ckUSDC, ckBTC, 及び将来的に追加される ICRC-1/2 準拠トークン**。  
Stripe に類似した開発者体験（PaymentIntent, Checkout, Webhook, Refund, Connect-style 分配）を提供し、  
EC・クラファン・サブスク等のユースケースに導入可能なプラットフォームを実現する。

---

## 基本方針
- **非カストディ**  
  運営者はユーザー資産を保持しない。資産移動はスマートコントラクト（Canister）によるルールのみで実行。
- **ICRC-2 Pull 型決済**  
  支払者は `approve` のみ、Canisterが `transfer_from` で捕捉。送金ミスを排除。
- **マルチアセット対応**  
  Ledgerごとに Canister 設定で差し替え可能。Intent 作成時に `asset` フィールドで指定。
- **開発者体験優先**  
  TypeScript SDK / Hosted Checkout / Webhook 再送・署名検証などを最初から整備。

---

## アーキテクチャ

### Canisters
1. **Payments Canister**
   - Intent生成・キャプチャ・リリース・返金
   - マルチアセット対応（ckUSDC, ckBTC, 他ICRC-2）
   - Intentごとに escrow サブアカウントを生成
   - イベントログ（certified data）

2. **Merchant Registry Canister（将来拡張）**
   - マーチャント情報（受取先アカウント、Webhook URL、公開鍵）
   - KYCレベルや手数料率の管理

3. **Webhook Dispatcher（将来拡張）**
   - イベントを非同期で外部送信
   - 署名付きヘッダ、再送キュー、バックオフ制御

### クライアント
- **TypeScript SDK**
  - `createPaymentIntent`, `capture`, `release`, `refund`, `verifyWebhook`
  - Node/Browser両対応
- **Next.js Hosted Checkout**
  - 決済リンク / QRコード
  - ウォレット接続（ICRC対応：ckBTC, ckUSDC）
  - Approve → Capture の最短導線
- **ダッシュボード（後続）**
  - Intent一覧・Webhookログ・返金・分配・CSVエクスポート

---

## データモデル

### PaymentIntent
- `id`: テキスト
- `merchant`: Principal
- `payer`: Optional<Account>
- `escrow`: Account (canister owner + subaccount)
- `asset`: テキスト（例: "ckUSDC", "ckBTC"）
- `amount`: nat（整数単位 μUSDC, satoshi 等）
- `status`: enum
- `created_at`, `expires_at`: nat64
- `metadata`: Key-Value ペア

### 状態遷移
requires_approval → succeeded → (released | refunded)
│
└── expired


---

## 処理フロー

### 1. Intent 作成
- マーチャントが API/SDK で `create_intent(amount, asset, merchant, expires_at)` を呼ぶ
- Escrow サブアカウントを生成
- 状態: `requires_approval`

### 2. 支払者承認
- Checkout UI がウォレットを呼び出し `icrc2_approve(spender=Payments Canister, amount, expires_at)` を実行
- Ledger に allowance 記録

### 3. Capture
- Payments Canister が `icrc2_transfer_from` を呼び、資金を Escrow に移動
- 状態: `succeeded`
- Webhook: `payment_intent.succeeded`

### 4. Release
- マーチャント or プラットフォームが `release(intent_id, splits)` を呼ぶ
- Escrow から受益者アカウントへ送金
- 状態: `released`
- Webhook: `payment_intent.released`

### 5. Refund
- `refund(intent_id, amount)` 実行で支払者に返金
- 状態: `refunded`
- Webhook: `payment_intent.refunded`

### 6. Expire
- 承認が期限切れの場合は自動で `expired`
- Webhook: `payment_intent.expired`

---

## マルチアセット設計
- Intent 作成時に `asset` を指定
- Canister 内部に **ledger registry** を保持
  ```rust
  struct LedgerInfo { ledger_id: Principal, decimals: u8 }
  ```
