# Mycel Signer Activity Model

狀態：design draft

這份文件描述 Mycel-based custody deployment 應如何評估 signer activity，而不是把 signer availability（可用性）當成二元猜測。

核心設計原則是：

- signer 可能合法 enrolled，但在操作上已不可用
- signer 可能可聯絡，但仍不足以安全地被視為 fully active
- signer activity 應明確量測，而不是從日常溝通中模糊推定
- custody 的實際安全性取決於 currently active signer set，而不只是 configured signer set

## 0. 目標

讓 Mycel-based m-of-n custody system 能區分：

- 誰已 enrolled
- 誰目前被允許簽章
- 誰在操作上真的還能簽章
- 誰仍可被安全地算入實際可用容量

這份文件是對 consent 與 custody-policy notes 的補充。

它不重新定義：

- signer consent
- policy scope
- execution authorization

## 1. 為什麼要把 Activity 獨立建模

signer 可能出現許多無法用簡單 `active / revoked` 狀態描述的失效方式：

- signer 仍同意參與，但已失去裝置存取
- signer 可聯絡，但經常錯過簽章時窗
- signer key material 仍存在，但 recovery procedure 已不可行
- signer 仍在名單中，但實際上已依附於另一個 signer 而失去獨立性

如果系統忽略這些差異，名義上的 `3-of-7` 在實務上可能只剩脆弱的 `3-of-4`。

## 2. Activity 維度

signer activity 至少應沿著四個維度評估。

### 2.1 Technical Readiness

signer 是否仍能完成安全簽章所需的技術路徑？

例子：

- signed heartbeat 成功
- challenge-response 成功
- signer node、HSM 或 MPC endpoint 可連線
- 憑證與裝置狀態仍然有效

### 2.2 Operational Responsiveness

signer 是否能在預期的操作時窗內回應？

例子：

- 在關鍵聯絡 SLA 內回應
- 參與 drills 與 rotations
- 不會一再錯過 execution windows
- 仍能使用既定的安全通訊路徑

### 2.3 Governance Participation

signer 是否仍屬於 accepted 的操作結構？

例子：

- 仍在 accepted signer set 中
- 仍接受 active policy scope
- 尚未退出角色
- 理解目前的 pause、revoke 與 rotation 狀態

### 2.4 Security Hygiene and Independence

signer 是否仍被獨立且安全地控制？

例子：

- 沒有失控 delegation（委派）
- 沒有不安全的裝置共用
- 沒有明顯帳號或憑證被入侵跡象
- 沒有會讓 signer independence 坍縮的隱藏依賴

## 3. Activity States

這份文件建議至少使用以下操作狀態：

- `active`
- `degraded`
- `inactive`
- `retired`
- `revoked`

這些狀態不應與 consent state 視為同義。

### 3.1 Active

signer：

- 通過最近的技術檢查
- 在預期時窗內回應
- 仍位於 accepted 的治理範圍內
- 維持可接受的 security hygiene

### 3.2 Degraded

signer 仍屬於系統，但可靠度已下降。

常見原因：

- 一再延遲回應
- 錯過 drills
- runtime endpoint 不穩
- recovery path 不完整
- 對其他人產生更高的操作依賴

`degraded` signer 不應被忽略。

它是 effective signer set 可能正在縮水的早期警訊。

### 3.3 Inactive

signer 在實務上已無法被視為可穩定參與。

常見原因：

- 重複 failed heartbeat 或 challenge checks
- 缺少可靠聯絡路徑
- 長期遺失裝置
- 無法完成必要 recovery procedure

`inactive` signer 不應被算入實際簽章容量。

### 3.4 Retired

signer 以有序方式退出角色。

它仍應保留於歷史中，但不應再被期待參與未來 activity。

### 3.5 Revoked

signer 已被移出未來有效參與資格。

revocation 是治理與資格事件，不只是 activity observation。

## 4. Activity 評估的最低證據

activity status 應建立在明確證據上，而不是 operator 的直覺。

建議證據類型：

- 最近的 signed heartbeat
- 最近的 challenge-response 結果
- 最近成功完成的 drill participation
- 最近驗證過的 contact-path confirmation
- 最近的 rotation 或 policy acknowledgment
- 最近的 security-hygiene review

## 5. 建議評估規則

精確門檻取決於部署規模與風險，但實務基準可包括：

- 30 天內至少一次 signed heartbeat
- 90 天內至少一次 recovery 或 signing drill
- 7 天內成功回應關鍵聯絡
- 連續多次 missed checks 後降為 `degraded`
- 持續無法驗 readiness 後降為 `inactive`

系統應保留底層證據，而不只是最後標籤。

## 6. Effective Signer Set

custody 安全性應同時對照：

- configured signer set
- currently active signer set

例子：

- configured set：`3-of-7`
- currently active signers：`4`

這在 paper 上仍符合 policy，但在操作風險上更接近 `3-of-4`。

合規 deployment 應明確顯示這個差異。

## 7. 必要的 Client 與 Runtime 行為

### 7.1 Client

合規 client 應顯示：

- configured signer count
- 目前 `active`、`degraded`、`inactive`、`retired`、`revoked` 的數量
- 每個 signer 最近一次成功驗證時間
- 最近一次 drill 或 recovery-check 時間
- 是否建議 rotation

client 不應暗示 enrollment 本身就等於實際 readiness。

### 7.2 Runtime

合規 runtime 應：

- 記錄明確的 activity evidence
- 區分技術檢查失敗與治理上的 revocation
- 在 signer 變成實際不可用前先發出 warnings
- 不可把過期 activity data 當成 current

## 8. Failure Cases

### 8.1 Silent inactivity

signer 已停止參與，但系統沒有明確檢查。

結果：

- 系統高估真實簽章容量

### 8.2 將 Enrollment 誤認為 Readiness

signer 仍在名單中，但裝置路徑早已損壞。

結果：

- custody 規劃者誤判實際韌性

### 8.3 Correlated degradation

多個 signer 看似分開 enrolled，卻共享某個隱藏依賴。

結果：

- activity labels 高估獨立性

### 8.4 Inactivity 後沒有 Rotation

inactive signers 長時間留在集合中。

結果：

- 名義冗餘逐步退化為操作脆弱性

## 9. 建議控制

- 要求定期 signed heartbeat
- 要求定期 challenge-response checks
- 要求定期 recovery drills
- 把 signer-activity evidence 記錄為可審計事件
- 分開追蹤 effective signer capacity 與 configured capacity
- 當 degraded 或 inactive 數量跨過門檻時，觸發 review 或 rotation

## 10. 實務判準

對 custody 安全性而言，真正重要的問題不是：

- 「這個 signer 還在名單裡嗎？」

而是：

- 「這個 signer 現在還能否安全、獨立、準時地參與？」

如果 deployment 無法用證據回答這件事，它其實並不知道自己當前的 m-of-n 韌性。
