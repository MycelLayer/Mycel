# Mycel App Layer

狀態：design draft

這份筆記描述一個由 Mycel 承載的應用層，同時把副作用執行維持在核心協議之外。

核心原則是：

- Mycel 承載 app definition、app state、intents、effect history
- runtime 負責執行外部副作用
- client 面向使用者讀寫 app 狀態，但不直接定義 transport 或 execution semantics

## 0. 目標

讓 Mycel 可以承載 application behavior，同時不把核心協議變成 side-effect engine。

放在 Mycel 裡：

- app definition
- app state
- app governance
- app intents
- effect requests
- effect receipts

留在 Mycel core 外：

- HTTP execution
- timers
- secret handling
- network credentials
- OS-level 或 cloud-level side effects

## 1. 三層分工

App Layer 應拆成三個明確層次。

### 1.1 Client Layer

client 是面向人類使用者或本地使用者的層。

責任：

- 顯示 accepted app state
- 建立 intents 或 actions
- 檢查 effect status
- 檢查 receipts 與 audit trails
- 呈現 app-specific UI

非責任：

- 預設不直接執行 privileged effects
- 不重定義 accepted-head 規則
- 不把本地 secrets 寫進可複製的 Mycel objects

### 1.2 Runtime Layer

runtime 是 effect executor。

責任：

- 監看 accepted heads
- 讀取 app manifests 與 app state
- 解析 pending effect requests
- 執行 capability policy
- 執行被允許的 side effects
- 把 effect receipts 再寫回 Mycel

非責任：

- 不重定義 protocol verification
- 不繞過 governance rules
- 不把任意 branch 視為可執行，除非它在 active profile 下已 accepted

### 1.3 Effect Layer

effect layer 是對 side effects 的顯式表示。

責任：

- 描述要求執行什麼外部動作
- 紀錄實際執行了什麼外部動作
- 保留可驗證的 execution trail

effect objects 應該可審計、可避免 replay 觸發、且語義明確。

## 2. 設計規則

Revision replay MUST 保持 side-effect free。

這表示：

- 重放 Mycel 歷史只會重建 state
- 重放 Mycel 歷史不會重新觸發 HTTP calls
- effect execution 由 runtime 觀察 accepted state transitions 後再驅動

這條規則對保持 verification 的決定性非常重要。

## 3. 核心 App 物件

一個 App Layer 可以用少量物件家族表達。

### 3.1 App Manifest

定義應用本身。

建議欄位：

- `app_id`
- `app_version`
- `entry_documents`
- `state_documents`
- `intent_documents`
- `allowed_effect_types`
- `runtime_profile`
- `capability_policy`

用途：

- 識別 app
- 宣告哪些 documents 屬於它
- 宣告允許哪些 effect 類型

### 3.2 App State Document

保存 accepted application state。

例子：

- workflow state
- queue state
- approval state
- resource summary
- last successful effect status

### 3.3 Intent 或 Action Document

保存被要求的業務動作。

例子：

- submit task
- request publication
- request sync
- request API fetch

intent 本身不是 side effect。
它是一個有狀態的請求，之後才可能產生 effect request。

### 3.4 Effect Request

表示對外部執行的請求。

建議欄位：

- `effect_request_id`
- `app_id`
- `effect_type`
- `trigger_revision`
- `requested_by`
- `request_payload`
- `idempotency_key`
- `requested_at`

`effect_type` 例子：

- `http.fetch`
- `http.post`
- `webhook.deliver`
- `notification.send`

### 3.5 Effect Receipt

表示 runtime 實際做了什麼。

建議欄位：

- `effect_receipt_id`
- `effect_request_id`
- `executor`
- `status`
- `started_at`
- `finished_at`
- `response_digest`
- `response_summary`
- `error_summary`

用途：

- 證明執行曾經發生
- 記錄成功或失敗
- 支援 audit 與 retry logic

## 4. Accepted-Head 驅動的執行

runtime 應只從 accepted state 觸發 effects。

建議規則：

1. 讀取 app state documents 的 active accepted head
2. 找出新被 accepted 的 effect requests
3. 檢查 capability 與 runtime policy
4. 執行被允許的 effects
5. 發布 effect receipts
6. 讓後續 accepted state 吸收這些 receipts

這樣 app execution 才會和 Mycel governance 對齊。

## 5. HTTP 例子

對一個可做 HTTP 的 app：

### Client

- 顯示表單或 task UI
- 寫入一個要求 outbound call 的 intent
- 顯示 pending 或 completed 的 execution state

### Runtime

- 發現有 accepted 的 effect request
- 驗證 `http.post` 是否被允許
- 檢查 URL allowlist 與 runtime policy
- 執行 HTTP request
- 寫入摘要回應的 receipt

### Effect 物件

- `effect_request`：`POST` 這份 payload 到這個已核准 endpoint
- `effect_receipt`：runtime X 在時間 T 執行，得到 status 200

## 6. Capability Model

runtime 不應執行來自任意 replicated content 的任意 effects。

建議 capability controls：

- effect type allowlist
- HTTP domain allowlist
- method allowlist
- timeout limits
- payload size limits
- response size limits
- 除非明確允許，否則不得直接存取 local network

capabilities 應在 app definition 中宣告，並由 runtime policy 強制執行。

## 7. Secrets 與 Credentials

secrets 不應存在於可複製的 Mycel objects 裡。

建議規則：

- Mycel objects 可以命名 credential reference
- 只有 runtime 解析真正的 secret
- receipts 不應暴露 secrets

例子：

- `credential_ref: vault:mailgun-prod`
- 而不是 `api_key: ...`

## 8. Idempotency 與重複執行

因為多個 runtimes 可能同時觀察到同一個 accepted state，所以必須預期 duplicate execution。

建議控制：

- 每個 effect request 都有 `idempotency_key`
- runtime 保存看過的 request IDs
- receipts 必須回指精確的 request ID
- app logic 要把 duplicate receipts 視為可協調狀態，而不是 protocol corruption

## 9. Governance Integration

App Layer 應遵守 Mycel governance，而不是繞過它。

建議整合方式：

- 只有 accepted heads 可以進入 executable effect queue
- view-maintainers 透過控制 accepted heads，間接控制哪些東西可以被執行
- editor-maintainers 可以建立 app changes 或 requests，但它們在被 accepted 前不可執行

這與目前 profile-governed accepted-head 模型一致。

## 10. 最小生命週期

最小 app lifecycle 可以長這樣：

1. 定義 `app_manifest`
2. 建立 app state documents
3. 提交 intent
4. 把 intent 接受到 active accepted head
5. 實體化或導出 effect request
6. runtime 執行 effect
7. runtime 寫入 effect receipt
8. accepted state 反映執行結果

## 11. 實際適合先做的 Apps

適合先做的 App Layer 例子：

- webhook workflow app
- approval app
- content publish app
- notification app
- external sync app

這些都比一開始就做 fully general remote-execution platform 更實際。

## 12. Non-Goals

這個 App Layer 不應試圖變成：

- smart-contract VM
- unrestricted function-as-a-service platform
- external HTTP 的 deterministic replay engine
- secret storage system

## 13. 建議的未來規範方向

如果這個設計成熟，後續 spec 可定義：

- app manifest schema
- effect request schema
- effect receipt schema
- runtime profile schema
- capability policy schema
- client/runtime conformance split

## 14. 開放問題

- effect requests 應成為 first-class protocol objects，還是只當 app-level document content？
- receipts 是否應一律由 runtime keys 簽章？
- runtimes 是否也應像 view-maintainers 一樣被治理準入？
- 第一個 App Layer profile 應支援完整 HTTP，還是只支援更小的 webhook delivery subset？
