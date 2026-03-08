# Donation App Layer

狀態：design draft

這份筆記描述一個由 Mycel 承載、偏向捐贈流程的應用層，同時把支付執行留在核心協議之外。

核心原則是：

- Mycel 承載 donation campaigns、donation intents、settlement state、allocation state 與 audit history
- client 讓使用者檢查 campaigns、建立 donation intents、並查看 receipts
- payment runtime 負責執行外部 payment actions
- core protocol 保持中立且純技術化

## 0. 目標

讓 Mycel 可以承載一個可持久保存的 donation workflow，同時不把核心協議變成 payment processor。

放在 Mycel 裡：

- donation app definition
- campaign state
- donation intents
- settlement receipts
- allocation 與 reporting state
- governance 與 audit history

留在 Mycel core 外：

- payment execution
- card 或 bank credential handling
- wallet secret handling
- processor-specific authentication
- 不可逆的 settlement side effects

## 1. 三層分工

### 1.1 Client Layer

client 是面向使用者的層。

責任：

- 顯示 active campaigns 與 donation targets
- 建立 donation intents 或 pledges
- 顯示 settlement status
- 顯示 receipts、allocations 與 audit trails
- 呈現 campaign-specific UI

非責任：

- 預設不執行 payment side effects
- 不持有可複製的 payment secrets
- 不重定義 accepted allocation 規則

### 1.2 Runtime Layer

runtime 是 payment executor 與 settlement observer。

責任：

- 監看 accepted donation state
- 建立或對帳 payment sessions
- 驗證允許的 payment methods
- 發布 settlement receipts
- 發布 failure 或 retry receipts

非責任：

- 不重定義 protocol verification
- 不繞過 accepted-head governance
- 不把未 accepted 的 branches 視為可執行的 payment instructions

### 1.3 Effect Layer

effect layer 顯式表示與支付有關的 side effects。

例子：

- create payment session
- poll settlement status
- verify on-chain confirmation
- send donor notification

effect objects 應保持可審計、replay-safe、且語義明確。

## 2. 設計規則

Donation App Layer 應遵守五條規則。

1. Revision replay MUST 保持 side-effect free。
2. Payment execution MUST 發生在核心協議之外。
3. Settlement evidence 與 allocation decisions SHOULD 以 app-level records 形式保存。
4. client SHOULD 明確區分 payment completion 與 allocation acceptance。
5. 本地 safety policy SHOULD NOT 靜默改寫 accepted donation state。

## 3. 核心 Donation 物件

### 3.1 Donation App Manifest

定義 donation 應用本身。

建議欄位：

- `app_id`
- `app_version`
- `campaign_documents`
- `intent_documents`
- `receipt_documents`
- `allocation_documents`
- `allowed_effect_types`
- `runtime_profile`
- `capability_policy`

用途：

- 識別 donation app
- 宣告參與的 document families
- 宣告 payment 與 runtime 的預期

### 3.2 Donation Campaign

定義一個 active 或歷史 campaign。

建議欄位：

- `campaign_id`
- `app_id`
- `title`
- `description`
- `status`
- `target_amount`
- `currency_policy`
- `accepted_payment_methods`
- `allocation_policy_ref`
- `created_at`
- `updated_at`

典型 `status` 值：

- `draft`
- `active`
- `paused`
- `closed`
- `archived`

### 3.3 Donation Target

定義資金預期的去向。

建議欄位：

- `target_id`
- `campaign_id`
- `destination_ref`
- `allocation_label`
- `currency_policy`
- `visibility`

典型 `visibility` 值：

- `public`
- `restricted`
- `internal`

### 3.4 Donation Intent

表示使用者端要捐贈的請求或 pledge。

建議欄位：

- `intent_id`
- `campaign_id`
- `target_id`
- `donor_ref`
- `intent_kind`
- `amount`
- `currency`
- `payment_method`
- `status`
- `created_at`
- `updated_at`

典型 `intent_kind` 值：

- `direct-payment`
- `pledge`
- `recurring-consent`

典型 `status` 值：

- `pending`
- `payment-requested`
- `settled`
- `failed`
- `cancelled`

### 3.5 Donation Receipt

表示 runtime 觀察到的 settlement 或 payment confirmation。

建議欄位：

- `receipt_id`
- `intent_id`
- `executor`
- `payment_ref`
- `amount_received`
- `currency`
- `status`
- `settled_at`
- `processor_summary`
- `error_summary`

典型 `status` 值：

- `confirmed`
- `pending-confirmation`
- `failed`
- `reversed`

### 3.6 Allocation Resolution

表示已結算 donations 要如何被分配或認列。

建議欄位：

- `resolution_id`
- `campaign_id`
- `covered_receipts`
- `allocations`
- `accepted_under_profile`
- `decision_trace_ref`
- `updated_at`

用途：

- 把 payment settlement 與 allocation governance 分開
- 保留 accepted distribution state
- 支援 auditability

## 4. Accepted-State 驅動的執行

runtime 應只從 accepted state 觸發與支付有關的 effects。

建議規則：

1. 讀取 campaign 與 intent state 的 active accepted head
2. 找出新被 accepted、且需要 runtime 處理的 donation intents
3. 檢查 payment capability 與 runtime policy
4. 執行被允許的 payment 或 settlement checks
5. 發布 donation receipts
6. 讓後續 accepted state 吸收這些 receipts 與任何 allocation results

這樣 donation execution 才能和 Mycel governance 與 traceability 對齊。

## 5. 範例流程

### Client

- 顯示一個 active campaign
- 讓 donor 建立 `donation_intent`
- 顯示 pending、settled 或 failed 狀態
- 顯示已結算資金之後如何被分配

### Runtime

- 發現一個 accepted donation intent 需要 payment session 或 settlement check
- 驗證所選的 payment method 是否被允許
- 與外部 payment system 互動
- 寫入 `donation_receipt`

### Governance

- maintainers 可以發布 allocation 或 campaign-closing state
- reader clients 在 active profile 下顯示 accepted allocation result

## 6. Capability Model

runtime 不應從任意 replicated content 執行任意 payment behavior。

建議 capability controls：

- payment-method allowlist
- processor allowlist
- destination allowlist
- 每個 campaign 的 currency restrictions
- 最小與最大金額檢查
- retry limits
- 禁止在 replicated objects 中放入直接 secret material

capabilities 應在 app definition 中宣告，並由 runtime policy 強制執行。

## 7. Secrets 與 Credentials

secrets 不應存在於可複製的 Mycel objects 裡。

建議規則：

- Mycel objects 可以命名 credential reference
- 只有 runtime 解析真正的 secret
- receipts 不應暴露 card、bank 或 wallet secrets

例子：

- `credential_ref: vault:stripe-prod`
- `credential_ref: vault:wallet-signer-a`

## 8. Idempotency 與 Reconciliation

donation flows 必須預期 retries、延遲 settlement 與重複通知。

建議規則：

- 每個 payment-side effect 都應帶 idempotency key
- 每個 receipt 都應回指一個 intent 或 payment request
- duplicate receipts 應被視為 state 的可協調情況，而不是 protocol corruption
- reversals 與 failures 應保留成顯式 records

## 9. Privacy 與 Audit

donation workflows 可能同時需要 privacy 與 accountability。

建議設計：

- donor-visible IDs 應與 runtime payment references 分開
- 部署可使用 pseudonymous `donor_ref`
- 公開 campaign reporting 與受限的 donor-identifying data 應分開
- 即使 donor identity 被最小化，仍應保留 receipt 與 allocation 的 audit links

## 10. 最小 v0.1 Donation Profile

對第一版實作，我建議先採較窄的 profile。

- 一種 campaign family
- 一種 donation-intent family
- 一種 receipt family
- allocation resolution family 可選
- 不在 replicated state 中放 secrets
- 不做 automatic recurring execution
- 一個 accepted settlement view 加上 audit-visible alternatives

取捨：

- automation 較低
- implementation clarity 較高
- interoperability 較容易

## 11. Open Questions

- `donation_target` 應是 dedicated record family，還是 campaign state 內的一個欄位？
- 第一版 settlement 是否只支援 processor receipts，還是也支援 on-chain confirmations？
- allocation governance 是否應對每個 receipt 都強制存在，還是只用於 campaign-level reporting？
