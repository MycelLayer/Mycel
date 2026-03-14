# 企業即時通訊應用層

狀態：設計草案

這份筆記描述 Mycel 如何承載一個面向企業規劃、內部協作與合規管理的即時通訊 app，同時不把核心協議變成聊天傳輸協定、DLP 引擎或秘密銷毀系統。

核心原則是：

- Mycel 承載名冊狀態、對話政策、成員資格、訊息信封與合規歷史
- runtime services 負責 blob 傳遞、金鑰保管、計時執行、機密內容掃描與企業橋接整合
- 合規 client 在 active profile 下渲染 accepted conversation view，同時保留可稽核性

另見：

- [`DESIGN-NOTES.mycel-app-layer.zh-TW.md`](./DESIGN-NOTES.mycel-app-layer.zh-TW.md)：一般 app-layer 分層
- [`DESIGN-NOTES.realtime-media-app-layer.zh-TW.md`](./DESIGN-NOTES.realtime-media-app-layer.zh-TW.md)：另一個把傳輸留在 Mycel core 之外的例子

## 0. 目標

讓 Mycel 可以支援一個適合企業規劃與內部治理的即時通訊系統，用於：

- 員工名冊管理
- 部門與專案群組管理
- 受政策約束的內部訊息流
- 機密訊息過濾、隔離與放行
- 計時型訊息到期與銷毀流程
- 合規稽核、升級處理與 legal hold

放在 Mycel 裡：

- 組織與名冊定義
- 群組與成員資格狀態
- 對話中繼資料
- 訊息信封與政策標籤
- 過濾決策與放行歷史
- 保留與銷毀政策狀態
- 稽核軌跡與例外核准

留在 Mycel core 外：

- 即時傳輸與投遞
- push notification 基礎設施
- 訊息 blob 儲存
- plaintext secrets 處理
- 企業金鑰管理
- timer 執行
- DLP 引擎、惡意程式掃描與 HRIS / IdP 橋接憑證

## 1. 設計規則

這個 messenger app 應遵守八條規則。

1. Revision replay MUST 保持無副作用。
2. 如果需求包含真正的銷毀，plaintext secrets SHOULD NOT 直接作為一般可複製的 Mycel 訊息本文保存。
3. 名冊變更、成員資格變更與權限變更 MUST 以顯式簽章物件表示。
4. 機密訊息過濾 MUST 以明確的 policy decision 表示，而不是靜默改寫內容。
5. 計時型銷毀 MUST 以 governed state 加上 runtime receipts 表示，不能假設 append-only 歷史會自己抹除。
6. Legal hold 與銷毀覆寫 MUST 是一等政策狀態。
7. Accepted conversation rendering SHOULD 來自固定 profile 規則，而不是各 client 自行信任。
8. 外部企業橋接 MAY 協助同步，但它們的動作 MUST 把可稽核 receipts 寫回 Mycel。

最重要的含意是：

- 如果企業需要強語義的「到期後真的看不到」，Mycel 應保存簽章信封、retention contract 與內容 digest，而可撤銷的 ciphertext 與 keys 則放在外部 sealed store

## 2. 分層拆分

### 2.1 Client Layer

client 是面向員工或操作人員的介面層。

責任：

- 顯示 accepted 的名冊、群組、對話與訊息狀態
- 建立新訊息與管理 intent
- 顯示機密等級與傳遞限制
- 顯示銷毀計時器、legal hold 與放行狀態
- 顯示可被稽核的過濾或隔離結果

非責任：

- 不自行定義 accepted policy
- 不繞過群組或名冊政策
- 不把被阻擋內容靜默替換成編修後內容
- 不把企業主金鑰放進可複製物件

### 2.2 Enterprise Runtime Layer

runtime 是橋接與政策執行層。

責任：

- 從核准的上游系統同步員工名冊
- 執行對話成員資格與裝置 / session 政策
- 透過核准的企業通道投遞訊息 blob
- 執行機密內容掃描或分類
- 執行銷毀計時器、金鑰撤銷與 purge 流程
- 把明確 receipts 與 policy outcomes 回寫到 Mycel

非責任：

- 不重定義 protocol verification
- 不把未 accepted 的 branch state 視為 policy truth
- 不靜默改寫 accepted history

### 2.3 Effect Layer

effect layer 是企業側動作的顯式表示。

例子：

- `idp.sync-roster`
- `message.blob-upload`
- `message.deliver`
- `dlp.scan`
- `message.quarantine`
- `key.revoke`
- `blob.purge`
- `legal-hold.apply`

effect objects 應保持顯式、replay-safe 且可稽核。

## 3. 核心物件家族

### 3.1 Enterprise Messenger Manifest

定義這個 app 本身。

建議欄位：

- `app_id`
- `app_version`
- `directory_documents`
- `group_documents`
- `conversation_documents`
- `message_documents`
- `compliance_documents`
- `retention_documents`
- `allowed_effect_types`
- `runtime_profile`

用途：

- 識別 messenger app
- 宣告參與的 document families
- 宣告允許的 effect classes

### 3.2 Employee Roster Entry

表示一個企業身分。

建議欄位：

- `employee_id`
- `person_ref`
- `employment_state`
- `department_id`
- `manager_ref`
- `job_title`
- `policy_tier`
- `contact_points`
- `effective_at`
- `supersedes_roster_entry`

建議的 `employment_state` 值：

- `active`
- `leave`
- `contractor`
- `suspended`
- `terminated`

用途：

- 定義誰可以參與
- 把組織政策綁到穩定身分
- 保留明確的到職、離職與停權歷史

### 3.3 Group Document

表示一個受管理的部門、專案空間或控制室。

建議欄位：

- `group_id`
- `group_kind`
- `display_name`
- `owner_refs`
- `default_message_policy`
- `membership_policy`
- `retention_policy_ref`
- `classification_policy_ref`
- `created_at`
- `updated_at`

建議的 `group_kind` 值：

- `department`
- `project`
- `incident`
- `leadership`
- `vendor-bridge`

### 3.4 Membership Grant Document

表示誰被允許加入某個群組或對話。

建議欄位：

- `membership_id`
- `subject_ref`
- `target_group_id`
- `role`
- `grant_state`
- `granted_by`
- `granted_at`
- `expires_at`
- `supersedes_membership`

建議的 `role` 值：

- `member`
- `group-admin`
- `compliance-reviewer`
- `incident-commander`
- `guest`

建議的 `grant_state` 值：

- `pending`
- `active`
- `suspended`
- `revoked`
- `expired`

### 3.5 Conversation Document

表示一個聊天室、直接對話或公告頻道。

建議欄位：

- `conversation_id`
- `conversation_kind`
- `group_id`
- `participant_refs`
- `posting_policy`
- `visibility_policy`
- `classification_policy_ref`
- `retention_policy_ref`
- `destruction_mode`
- `status`
- `created_at`

建議的 `conversation_kind` 值：

- `direct`
- `team-room`
- `announcement`
- `war-room`
- `case-room`

建議的 `destruction_mode` 值：

- `none`
- `timer-hide`
- `timer-revoke`
- `timer-purge`

### 3.6 Message Envelope Document

表示一則訊息，但不假設 plaintext 本體一定永久複製在 Mycel 內。

建議欄位：

- `message_id`
- `conversation_id`
- `sender_ref`
- `sent_at`
- `message_kind`
- `content_digest`
- `blob_ref`
- `key_ref`
- `classification_label`
- `delivery_scope`
- `expiry_policy_ref`
- `reply_to`
- `supersedes_message`

建議的 `message_kind` 值：

- `text`
- `file`
- `announcement`
- `task-card`
- `approval-request`

用途：

- 識別訊息
- 把它綁到 policy 與 retention controls
- 在不要求 plaintext 複製的前提下支援驗證

### 3.7 Compliance Decision Document

表示分類、過濾、隔離或放行結果。

建議欄位：

- `decision_id`
- `target_message_id`
- `decision_kind`
- `decision_state`
- `classifier_ref`
- `reason_code`
- `reason_summary`
- `issued_at`
- `release_scope`
- `supersedes_decision`

建議的 `decision_kind` 值：

- `classify`
- `quarantine`
- `redact-view`
- `release`
- `escalate`
- `false-positive-clear`

建議的 `decision_state` 值：

- `pending-review`
- `blocked`
- `restricted`
- `released`
- `held`

這個物件的存在，就是為了避免「機密過濾」退化成看不見的 silent rewrite。

### 3.8 Retention Contract Document

表示一組訊息或一個對話所受的保留與銷毀條款。

建議欄位：

- `retention_contract_id`
- `target_ref`
- `retention_class`
- `destroy_after`
- `destroy_mode`
- `legal_hold_state`
- `hold_reason`
- `approved_by`
- `effective_at`

建議的 `retention_class` 值：

- `standard`
- `confidential`
- `restricted`
- `regulated`

建議的 `destroy_mode` 值：

- `hide-only`
- `key-revoke`
- `blob-purge`
- `key-revoke-and-purge`

### 3.9 Destruction Receipt

表示 runtime 實際完成了哪些銷毀或撤銷動作。

建議欄位：

- `destruction_receipt_id`
- `retention_contract_id`
- `target_message_id`
- `executor`
- `action_taken`
- `started_at`
- `finished_at`
- `result_state`
- `artifact_digest`
- `error_summary`

用途：

- 證明銷毀流程曾被執行
- 記錄 timer 是否成功、是否被 legal hold 擋下、或只完成部分步驟
- 在不假裝 immutable history 消失的前提下支援合規稽核

## 4. 企業工作流程

### 4.1 員工名冊管理

建議流程：

1. HRIS 或 IdP bridge 發布 roster-sync intents。
2. Runtime 驗證來源系統後寫入 roster-entry 更新。
3. 群組與對話的 membership policies 消費 accepted roster state。
4. 被停權或離職的人員，透過後續 policy objects 與 receipts 失去新內容投遞與金鑰存取資格。

重要規則：

- 名冊狀態可以控制誰能收到新內容，但既有稽核可見性仍應由顯式政策治理，而不是靜默消失

### 4.2 群組管理

建議模型：

- 把部門群組、專案群組與 incident rooms 作為分離的 group objects
- 讓 conversations 繼承群組預設政策，但仍允許更嚴格的本地覆寫
- 把 owner 變更、guest access 與 emergency access 寫成明確 membership documents

這樣可以支援：

- 由員工目錄驅動的群組建立
- 臨時專案房間
- 高階主管頻道
- 具有高保留與高存取控制政策的 incident command rooms

### 4.3 機密訊息過濾

過濾應被建模成 policy pipeline，而不是隱藏式內容改寫。

建議階段：

1. Sender 發布一個指向 sealed content 的 message envelope。
2. Delivery runtime 或 review runtime 請求一個 `dlp.scan` effect。
3. 掃描系統回傳 labels 或 risk codes。
4. Compliance decision document 記錄放行、限制、隔離或升級處理結果。
5. Accepted conversation view 在 active profile 下顯示被允許的結果。

良好的企業行為應該是：

- 員工可以看見某則訊息被 hold 或 restricted
- 合規審查者可以檢查 reason codes 與 release history
- 原始內容不會被無痕替換成「乾淨版本」

### 4.4 銷毀計時器與 Legal Hold

計時型刪除必須把使用者體驗與密碼學現實分開。

建議模型：

- `timer-hide`：到期後 reader 不再渲染內容，但不宣稱強語義銷毀
- `timer-revoke`：到期後 runtime 撤銷訊息金鑰，使一般 client 無法再解讀 ciphertext
- `timer-purge`：runtime 撤銷金鑰，並對 sealed store 請求 blob purge

Legal hold 行為：

- legal hold 應凍結 `timer-revoke` 或 `timer-purge`
- hold 狀態必須出現在 retention contracts 與 destruction receipts 中
- hold release 應是另一個明確的 policy event

這一點對企業規劃尤其重要，因為「自動銷毀」與「紀錄保留」通常互相衝突，除非系統直接把兩者都建模出來。

## 5. 建議政策配置

這個 messenger 至少應支援三種 policy profiles。

### 5.1 Standard Team Chat

適合日常內部協作。

特徵：

- 較廣的員工群組成員資格
- 一般 DLP 分類
- 可選 `timer-hide`
- 標準稽核保留

取捨：

- 最容易使用，但銷毀保證最弱

### 5.2 Restricted Project Room

適合財務、人資、採購或敏感合作案。

特徵：

- 顯式 membership grants
- 強制 classification labels
- 附件先隔離再放行
- `timer-revoke` 或 `timer-purge`

取捨：

- 控制更強，但審查摩擦與 runtime 依賴更高

### 5.3 Sealed Executive / Incident Room

適合危機處理、資安事件回應或董事會層級協作。

特徵：

- 極窄的名冊資格
- 成員變更需要帶外核准
- 更強的裝置與匯出限制
- 預設具備 legal-hold awareness
- 到期後積極撤銷金鑰

取捨：

- 控制姿態最強，但營運成本與支援負擔最高

## 6. 功能地圖

一般企業 IM 常見能力，建議至少分成十二組功能面來規劃。

### 6.1 身分與帳號治理

常見能力：

- `SSO`
- 員工入職 / 離職自動佈建
- `SCIM` 或 HRIS / IdP 名冊同步
- 訪客、外包與臨時帳號
- 停權、復權與 session 強制失效

這一組決定誰能進系統、何時失去存取權，以及名冊是否能跟企業真實組織同步。

### 6.2 群組與對話空間管理

常見能力：

- 部門群組
- 專案群組
- 公告頻道
- incident / war room
- direct message
- guest room

這一組決定企業內部的溝通邊界，以及哪些房間要走較嚴格的 membership 與 retention 政策。

### 6.3 訊息與互動能力

常見能力：

- thread
- `@mention`
- 引用回覆
- 置頂
- 草稿
- 排程發送
- 訊息編輯與撤回
- read receipt
- presence / 狀態訊息

這一組偏向日常使用體驗，但仍需跟合規與 retention 規則相容。

### 6.4 協作附件與流程物件

常見能力：

- 檔案分享
- 附件預覽
- task card
- approval request
- poll / 投票
- 行事曆邀請
- 會議連結
- 知識庫或工單連結

這一組讓企業 IM 從「聊天工具」提升為「工作協作入口」。

### 6.5 搜尋、整理與知識保存

常見能力：

- 全文搜尋
- 依人員 / 群組 / 標籤 / 日期過濾
- 收藏
- pin / 置頂訊息
- 封存
- 摘要
- 歷史回溯

這一組決定訊息能不能在之後成為可檢索的工作知識，而不是一次性聊天噪音。

### 6.6 合規、稽核與法務支持

常見能力：

- audit log
- 匯出
- eDiscovery
- legal hold
- retention policy
- 審查流程
- 事件升級處理
- 管理員稽核視圖

這一組是企業 IM 與一般消費型聊天工具最明顯的分界之一。

### 6.7 安全與資料保護

常見能力：

- DLP
- 敏感內容分類
- 附件隔離
- 惡意程式掃描
- 禁止轉寄 / 複製 / 下載
- 裝置綁定
- 金鑰撤銷
- 密文 blob purge

這一組直接對應本設計中的 `Compliance Decision`、`Retention Contract` 與 `Destruction Receipt`。

### 6.8 裝置、Session 與端點管理

常見能力：

- 多裝置登入
- session 管理
- 遠端登出
- 裝置信任等級
- `BYOD` 政策
- `MDM` / `MAM` 整合
- 離線訊息政策

這一組決定企業對終端裝置的控制力，以及「誰能看」與「哪台設備能看」之間是否分離治理。

### 6.9 通知、值班與營運協作

常見能力：

- 公告廣播
- 值班交接
- incident escalation
- 回覆 SLA 提醒
- 跨時區通知
- do-not-disturb 政策

這一組通常是企業 IM 在營運、客服、SRE 或資安場景中真正被依賴的部分。

### 6.10 整合、自動化與 Bot

常見能力：

- bot
- webhook
- workflow automation
- 工單系統整合
- `CRM` / `ERP` / `HR` 系統整合
- 核准與提醒工作流

這一組讓企業 IM 可以接上原有內部系統，而不是成為資訊孤島。

### 6.11 部署、租戶與資料主權

常見能力：

- 多租戶隔離
- 資料分區
- data residency
- 備援與災難復原
- `BYOK`
- `KMS` / `HSM` 整合

這一組決定產品是否能進入對資料主權與合規要求較高的企業或政府場景。

### 6.12 使用體驗與無障礙

常見能力：

- 桌面與行動端一致體驗
- 多語系
- 翻譯
- 語音轉文字
- 無障礙支援
- 高噪音頻道靜音與摘要

這一組雖然看起來比較「產品化」，但常常決定導入成敗。

## 7. 分階段規劃

如果把這個 enterprise messenger app 當成產品規劃來看，建議至少分成三段。

### 7.1 MVP

先做能驗證企業治理模型的核心能力：

1. roster entries 與名冊同步
2. managed groups 與 membership grants
3. conversation metadata
4. 帶 sealed blob references 的 message envelopes
5. classification 與 quarantine decisions
6. retention contracts 與 destruction receipts
7. 基本的 audit log 與管理端檢視

取捨：

- 能先驗證最關鍵的企業價值，但使用者體驗與整合深度會刻意收斂

### 7.2 Phase 2

補齊企業實際導入時最常被要求的能力：

- `SSO` / `SCIM`
- guest access
- announcement channels
- search / archive / pin / favorite
- read receipt / presence
- `BYOD` / 裝置政策
- webhook / bot / approval-request workflow
- eDiscovery 與 legal-hold operator view

取捨：

- 導入阻力會大幅下降，但 runtime、policy 與管理介面複雜度會上升

### 7.3 Phase 3

再往高控制、高營運需求場景擴張：

- incident / war room 套件
- 進階 export restrictions
- 更細的 data residency 與 tenant partitioning
- `BYOK` / `KMS` / `HSM` 深度整合
- 進階工作流編排與跨系統自動化
- 通話、會議、語音轉文字與跨系統 federation

取捨：

- 能打更高要求市場，但產品與運維成本都會明顯增加

## 8. Mycel 為何適合這一層

Mycel 適合這個 messenger layer，是因為它可以保留：

- 明確的名冊與成員資格歷史
- 在固定治理規則下導出的 accepted conversation policy
- 對合規與放行決策可見的 audit history
- 帶簽章歷史的 retention 與 destruction state
- 不把協定核心變成專有企業伺服器的 app-specific views

Mycel 並不是要取代：

- 傳輸協定
- 企業 KMS 或 HSM 系統
- 內容掃描引擎
- 通知閘道
- blob archive 儲存

## 9. 最小規劃切片

如果團隊想先做一個收斂的第一版，建議從這六塊開始：

1. roster entries
2. managed groups
3. conversation metadata
4. 帶 sealed blob references 的 message envelopes
5. classification 與 quarantine decisions
6. retention contracts 與 destruction receipts

這個切片已足以驗證企業規劃模型，之後再往通話、bots、workflow tasks 或外部 federation 擴張。
