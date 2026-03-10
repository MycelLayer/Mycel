# Sensor-triggered Donation App/Profile Architecture

狀態：design draft

這份筆記描述一個 app/profile architecture：runtime 觀察由機器導出的使用者狀態事件，並可在事先授權的 consent policy 下建立與 donation 有關的記錄。

白話說，這是一個讓未來修行者可以在事先約定的條件下，透過靜坐來收 donation 的模型。

核心原則是：

- Mycel 承載 consent state、session state、derived user-state events、donation intents 或 pledges、settlement receipts 與 audit history
- app layer 定義 record families、runtime interfaces 與使用者可見流程
- 固定 profile 定義 trigger eligibility、fund 上限與 execution constraints
- client 讓使用者檢查與設定整個流程
- sensing 與 payment runtime 負責執行外部觀測與 payment side effects
- core protocol 保持中立且純技術化

## 0. 目標

讓 Mycel 可以承載一個可追溯的 sensor-triggered donation workflow，同時不把核心協議變成 sensor processor 或 payment engine。

放在 Mycel 裡：

- consent policy state
- session summaries
- derived user-state events
- donation intents 或 pledges
- settlement receipts
- audit 與 dispute history

留在 Mycel core 外：

- raw sensor-signal capture
- low-level sensor interpretation
- payment execution
- secret handling
- 不可逆的 settlement side effects

## 1. 設計規則

這個 app 應遵守六條規則。

1. Revision replay MUST 保持 side-effect free。
2. Raw sensor streams MUST NOT 透過一般 Mycel state 被複製。
3. 單一 derived user-state event MUST NOT 直接等於 payment consent。
4. Auto-triggered donation behavior MUST 以明確的 consent profile 為前提。
5. Payment-side effects MUST 發生在核心協議之外。
6. donor MUST 能撤回、暫停或提出爭議。

### 1.1 App/Profile 分工

在這個 case 裡，架構應這樣拆：

- **app layer** 定義 record families、使用者可見狀態、runtime interaction points 與 audit surfaces
- **固定 profile** 定義什麼算合格事件、多久可觸發一次 funding、金額邊界是什麼，以及結果是 `pledge`、`manual-confirmation` 還是 `pre-authorized-payment`
- **core protocol** 只負責可驗證物件、可重播歷史、accepted-state derivation 與 replication

這樣可以把「靜坐合格後收 fund」的邏輯留在 app/profile 層，而不是塞進 protocol core。

## 2. 四層分工

### 2.1 Client Layer

client 是面向使用者的層。

責任：

- 顯示 consent profile state
- 顯示 session history 與 derived events
- 讓使用者啟用、暫停或撤回功能
- 顯示 donation pledges、intents、receipts 與 disputes
- 顯示為什麼某次 donation 有或沒有被觸發

非責任：

- 預設不直接解讀 raw signals
- 不執行 payment side effects
- 不繞過 consent policy

### 2.2 Sensor Runtime Layer

sensor runtime 觀察裝置輸出，並導出高階狀態事件。

責任：

- 連到已核准的 sensor interface
- 摘要一個 sensing session
- 導出明確的高階事件，例如 `stable-focus` 或 `stable-rest`
- 若部署 policy 有要求，則簽署或發布 session evidence summaries

非責任：

- 不把 raw signal streams 發布到 replicated state
- 不直接結算 payments

### 2.3 Payment Runtime Layer

payment runtime 負責執行 payment-side effects。

責任：

- 讀取 accepted 的 consent 與 trigger state
- 判定是否可建立 donation pledge 或 intent
- 在允許時執行或準備外部 payment steps
- 發布 settlement receipts 或 failure receipts

非責任：

- 不重定義 consent 規則
- 不使用未 accepted 的 branch state 作為 payment input
- 不把 sensor events 視為無上限授權

### 2.4 Effect Layer

effect layer 顯式表示外部觀測與支付動作。

例子：

- create sensor session
- derive high-level user-state summary
- create payment session
- check settlement result
- send donor notification

## 3. 核心物件

### 3.1 Consent Profile

定義使用者預先授權了什麼。

建議欄位：

- `consent_id`
- `user_ref`
- `trigger_mode`
- `allowed_amount`
- `currency`
- `cooldown_seconds`
- `max_triggers_per_day`
- `runtime_policy_ref`
- `status`
- `created_at`
- `updated_at`

典型 `status` 值：

- `active`
- `paused`
- `revoked`
- `expired`

### 3.2 Session Record

表示一個 sensing session 的摘要。

建議欄位：

- `session_id`
- `user_ref`
- `device_ref`
- `runtime_ref`
- `started_at`
- `ended_at`
- `summary_hash`
- `status`

典型 `status` 值：

- `complete`
- `failed`
- `discarded`

### 3.3 User-State Event

表示由一個完成的 session 所導出的高階事件。

建議欄位：

- `event_id`
- `session_id`
- `user_ref`
- `state_label`
- `stability_score`
- `duration_ms`
- `trigger_eligible`
- `created_at`

典型 `state_label` 值：

- `stable-focus`
- `stable-rest`
- `transition-state`

### 3.4 Donation Pledge 或 Intent

表示在一個合格事件之後，系統被允許做什麼。

建議欄位：

- `intent_id`
- `user_ref`
- `consent_id`
- `trigger_event_id`
- `intent_kind`
- `amount`
- `currency`
- `payment_method`
- `status`
- `created_at`
- `updated_at`

建議 `intent_kind` 值：

- `manual-confirmation`
- `pledge`
- `pre-authorized-payment`

### 3.5 Donation Receipt

表示 settlement 或 payment confirmation。

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

### 3.6 Dispute 或 Revocation Record

表示使用者提出的爭議、暫停或回滾請求。

建議欄位：

- `record_id`
- `user_ref`
- `related_intent_id`
- `related_receipt_id`
- `action_kind`
- `reason`
- `created_at`

典型 `action_kind` 值：

- `pause`
- `revoke`
- `dispute`
- `refund-request`

### 3.7 建議的 Document Families

在這個 case 裡，`document` 應被理解為長期歷史容器，而不是傳統散文文件。

建議 document families：

- `consent_document`：enrollment、consent scope、pause、revoke、dispute eligibility
- `session_document`：session summaries 與已核准 evidence references
- `event_document`：可進入評估的 derived user-state events
- `intent_document`：pledge、manual-confirmation 或 payment intent 的歷史
- `receipt_document`：settlement receipts 與 failure receipts
- `policy_document`：active trigger policy、fund 邊界、runtime allowlists 與 payout mode

某些部署可以把部分 families 合併，但第一版實作最好維持分離，以利 auditability。

## 4. 建議觸發政策

對第一版 client，我建議採保守的 trigger policy：

1. 使用者必須先建立明確的 consent profile
2. consent profile 必須限制金額與頻率
3. sensor runtime 必須從有限時的 session 導出高階事件
4. 該事件必須滿足最小持續時間門檻
5. cooldown window 必須已經過去
6. 在任何直接 payment 之前，系統應先建立 `pledge` 或 `manual-confirmation` intent

這樣可以把 derived user-state 與 payment authorization 清楚分開。

### 4.1 固定 Profile 必須定義什麼

一個固定 profile 至少應定義：

- `profile_id`
- 允許的 `state_label`
- 最小持續時間與穩定度門檻
- 核准的 runtime family 或 evidence format
- 金額與頻率限制
- cooldown 規則
- payout mode（`pledge`、`manual-confirmation` 或 `pre-authorized-payment`）
- dispute 與 pause 語義

不同部署可以選不同門檻，但每個 accepted outcome 都必須可追溯到一個固定 profile。

## 5. Accepted-State 驅動的執行

runtimes 應只從 accepted state 執行外部動作。

建議規則：

1. 讀取 accepted 的 consent 與 session state
2. 找出新被 accepted 的 derived events
3. 在 active consent profile 下評估它們
4. 若允許，建立 pledge 或 payment intent
5. 只有在 policy 允許時才執行外部 payment steps
6. 發布 receipts 與任何 dispute records

### 5.1 端到端流程

一個較窄的端到端流程可以是：

1. 使用者先建立一個明確的 consent profile
2. sensing runtime 建立一個有限時的 session summary
3. 部署把一個 derived event 納入 accepted state
4. 固定 profile 依金額、頻率與 cooldown 規則評估該事件
5. app 建立一個 `pledge` 或 `manual-confirmation` intent
6. 只有此時外部 payment runtime 才可嘗試 settlement
7. runtime 把 receipt 或 failure record 回寫到 Mycel

這樣可以讓 sensing、governance 與資金移動透過 accepted state 串起來，而不是靠不透明的 runtime shortcut。

## 6. Privacy 與 Data Minimization

這個 app 必須強力最小化敏感資料。

建議規則：

- 保存 session summaries，而不是 raw signals
- 保存 derived state labels 與 evidence hashes，而不是完整 waveform data
- 若可能，將 user identity 與 device identity 分開
- 將 payment references 與 user-facing records 分開
- 部署可使用 pseudonymous user references

## 7. Safety Guardrails

我建議以下硬性 guardrails：

- 沒有事前 consent 就不能 auto-trigger
- 不允許無上限金額或無上限頻率
- 不複製 raw signals
- 不允許 runtime 靜默改規則
- 不允許由未驗證或失敗的 sessions 直接觸發
- 不允許從 `manual-confirmation` 靜默退化成 direct payment

## 8. 最小 v0.1 Profile

對第一版實作，我建議先採較窄的 profile。

- 每位使用者只用一個 consent profile
- 只支援一個已核准的 sensor runtime family
- 只用一種可觸發的 derived state label
- 只允許 `pledge` 或 `manual-confirmation`
- 不做 direct automatic settlement
- 提供明確的 user pause 與 revoke controls

取捨：

- automation 較低
- safety risk 低很多
- auditability 較容易

## 9. Open Questions

- 第一版部署是否應允許 `pre-authorized-payment`，還是只能允許 `pledge`？
- runtimes 應如何證明某個 session summary 是由已核准 hardware 導出的？
- dispute records 應只保留在本地，還是當成一般 app records 複製？
