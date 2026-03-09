# Mycel Signature-priority Note

狀態：design draft

這份文件整理目前 Mycel repo 裡哪些 object families 應優先要求簽章，而不是把所有可能的 object family 都視為同等緊急。

核心設計原則是：

- 先簽定義 authority 的 objects，再簽只負責摘要結果的 objects
- 先簽參與 accepted-state derivation 的 objects，再簽方便性的 objects
- 先簽一旦被偽造就會靜默改變 governance 意義的 objects
- 以分層方式向外擴張，而不是一開始就把所有東西都要求簽章

## 0. 目標

為目前 repo 文件中的 object 提供一個實務上的簽章優先順序。

這份文件聚焦於由 Mycel 承載的 objects。

它不取代：

- release artifact signing
- transport envelope signing
- 僅限本地的 audit artifacts

## 1. 優先等級

這份文件使用三個優先等級：

- `P1`：最先必須簽
- `P2`：接下來應該簽
- `P3`：之後有價值，但不是第一批要穩定的簽章邊界

## 2. P1：最先必須簽

這些 objects 定義 app identity、governance authority 或 execution authorization。

如果它們沒有簽章，或簽章邊界很弱，accepted-state reasoning 就會不安全。

### 2.1 `app_manifest`

優先原因：

- 定義 application 是什麼
- 宣告哪些 documents 與 state families 屬於它
- 錨定 app 層的 authority 與 scope

可能的簽署者：

- app author 或 app maintainer

### 2.2 Governance proposals 與 approvals

以目前 repo 的詞彙為例：

- allocation proposal
- signer approval
- governance signal
- accepted resolution inputs

優先原因：

- 這些 records 會改變系統可接受為 accepted 的內容
- 偽造 governance records 會靜默重寫歷史意義

可能的簽署者：

- profile 定義的 governance actor、maintainer 或 signer role

### 2.3 `signer_enrollment`

優先原因：

- 證明 signer 的確知情地加入 custody system
- 錨定後續 consent 與 signer-activity 判斷

可能的簽署者：

- 正在加入的 signer
- 視情況再加一個 governance 或 operator 確認角色

### 2.4 `consent_scope`

優先原因：

- 定義 signer 接受了哪個有界 policy scope
- 若這裡沒有清楚 authorship，自動簽章就會變弱或無效

可能的簽署者：

- 給出 consent 的 signer

### 2.5 `signer_set`

優先原因：

- 定義後續 execution 依賴的 membership 與 m-of-n 規則
- 偽造 signer-set state 可能重導 approval power

可能的簽署者：

- governance 或 custody-maintainer 角色

### 2.6 `policy_bundle`

優先原因：

- 定義 execution 被允許做什麼
- 是自動化行為最直接的 authorization boundary

可能的簽署者：

- governance 或 policy-authorizing 角色

### 2.7 `trigger_record`，當它是 accepted 的 execution trigger 時

優先原因：

- policy-bound execution 就從這裡開始
- 偽造 trigger 會產生未授權的 execution intents

可能的簽署者：

- 依 profile 而定，可能是 trusted runtime、governance source 或 trigger-author 角色

### 2.8 `execution_intent`

優先原因：

- 把具體動作綁到 fund、policy、signer-set 與 amount 上下文
- 後續簽章與 execution 都要回指它

可能的簽署者：

- 系統導出的授權角色、authorized runtime 或 governance-derived executor role

## 3. P2：接下來應該簽

這些 objects 對 auditability、dispute handling 與高完整性運作非常重要，但 authority boundary 通常略早就已經開始。

### 3.1 `signer_attestation`

優先原因：

- 證明某個 signer 端的評估與結果
- 對 audit、threshold counting 與 mismatch analysis 很重要

可能的簽署者：

- signer runtime 或 signer identity

### 3.2 `execution_receipt`

優先原因：

- 證明 executor 或 runtime 實際做了什麼
- 對事後 audit 與外部 settlement linkage 很重要

可能的簽署者：

- runtime 或 executor identity

### 3.3 `pause_or_revoke_record`

優先原因：

- 改變未來 execution eligibility
- 應被簽章，避免靜默本地覆寫或偽造 emergency state

可能的簽署者：

- profile 定義的 governance authority、signer 或 emergency-control 角色

### 3.4 一般 app-layer systems 的 effect receipts

優先原因：

- 只要 runtime 會做外部 side effects，它就很重要
- 用來正確歸屬 runtime 行為

可能的簽署者：

- runtime identity

## 4. P3：之後有價值

這些在某些 deployment 仍然可能需要簽章，但不是第一批應穩定下來的邊界。

### 4.1 派生摘要與 snapshots

例如：

- balance snapshots
- resource summaries
- replayed indexes

之後再簽的原因：

- 它們雖然有價值，但理想上應可由已簽 source history 重新計算

### 4.2 快取型或便利型 state documents

例如：

- 本地 status summaries
- accepted state 的 convenience mirrors

之後再簽的原因：

- 它們不應凌駕於 canonical signed source objects 之上

### 4.3 選配的 monitoring 與 operator annotations

之後再簽的原因：

- 對操作很有用，但不是第一道 authority boundary

## 5. 目前 Repo 的最小第一批簽章集合

如果現在要在 repo 裡選一組狹窄但夠強的 first signature set，最實用的起點是：

1. `app_manifest`
2. governance proposals 與 approvals
3. `signer_enrollment`
4. `consent_scope`
5. `signer_set`
6. `policy_bundle`
7. `trigger_record`
8. `execution_intent`
9. `execution_receipt`

這組集合覆蓋了：

- app identity
- governance authority
- signer legitimacy
- execution authorization
- 最終 runtime evidence

## 6. 實務判準

如果偽造某個 object 會讓下列問題被靜默答錯，它就應該優先被簽：

- 誰有權決定？
- 到底批准了什麼？
- 誰有權簽章？
- 哪個動作被授權？
- 實際上發生了哪個動作？

如果某個 object 只是摘要已被簽過的 truth，它通常就不是第一順位的簽章目標。
