# Mycel Signer-availability Emergency Response

狀態：design draft

這份文件描述 Mycel-based m-of-n custody deployment 應如何偵測危險的 signer availability（可用性）下降，並在實際 lockout（鎖死）前，觸發 emergency rotation、address renewal 或其他 recovery path。

核心設計原則是：

- 不要等到 signer 完全失效才行動
- 持續量測 effective signer capacity
- 把 early warning 與 emergency lockout 分開
- 在需要之前就先定義 recovery paths

## 0. 目標

讓 custody deployment 能回答以下問題：

- 目前配置了多少 signers
- 現在實際可用的 signers 有多少
- signer set 何時已進入操作脆弱狀態
- 在每一個風險等級下，應觸發什麼動作

這份文件是以下文件的補充：

- signer activity evaluation
- signer consent modeling
- m-of-n custody rotation

## 1. 核心定義

### 1.1 Configured Signer Set

configured signer set 是 policy 上定義的名義 signer membership。

例子：

- configured set：`3-of-7`

### 1.2 Effective Signer Set

effective signer set 是仍能安全、獨立、準時參與的 signer 子集合。

它應建立在明確的 activity evidence 上，而不是只看 enrollment。

### 1.3 Lockout Risk

lockout risk 指系統已接近失去實際達到 `m` 的能力。

### 1.4 Emergency Renewal

emergency renewal 指在持續惡化導致實際失控前，把未來使用遷移離開當前 custody target。

依部署不同，它可能意味著：

- signer-set rotation
- address rotation
- 緊急搬到 recovery address
- 暫時凍結 execution 並進入 recovery workflow

## 2. 為什麼 Early Detection 很重要

如果系統等到 `effective_signers < m` 才處理，可能已經來不及用正常路徑搬移資產或 renew address。

因此實務系統必須提早偵測：

- 持續下降的 signer availability
- 高於 `m` 的安全緩衝正在縮小
- correlated signer failure（關聯性失效）
- 反覆出現的 degraded 狀態，並據此預測未來 lockout

## 3. 必要輸入

emergency response logic 至少應使用以下輸入：

- 已配置的 `m` 與 `n`
- 每個 signer 的 activity state
- 最近的 heartbeat 結果
- 最近的 challenge-response 結果
- 最近的 drill 或 recovery-check 結果
- contact-path success 或 failure
- correlated dependencies 的證據

## 4. Effective Signer 計算

deployment 至少應計算：

- `configured_signers`
- `active_signers`
- `degraded_signers`
- `inactive_signers`
- `revoked_or_retired_signers`
- `effective_signers`

建議的實務規則：

- `active` signer 算入 effective
- `degraded` signer 分開追蹤，並保守處理
- `inactive`、`retired`、`revoked` signer 不算入 effective

如果 deployment 選擇暫時把部分 `degraded` signer 算入，這個決定應明確且有時間限制。

## 5. 風險狀態

這份文件建議至少定義三個遞增風險狀態：

- `warning`
- `critical`
- `emergency`

### 5.1 Warning

signer set 仍可運作，但安全緩衝正在變薄。

常見觸發條件：

- `effective_signers <= m + 1`
- 反覆出現 `degraded`
- 多次錯過 drills
- 出現 correlated failure 的早期跡象

### 5.2 Critical

signer set 在技術上仍可運作，但再少一個就可能 lockout。

常見觸發條件：

- `effective_signers = m`
- `warning` 持續過久仍未恢復
- 多個 key owners 同時 degraded

### 5.3 Emergency

deployment 已失去，或即將失去，實際簽章能力。

常見觸發條件：

- `effective_signers < m`
- 單一 common-mode failure 一次移除多個 signers
- 在 critical 狀態下，必要 recovery checks 又失敗

## 6. 各風險狀態的建議動作

### 6.1 Warning 動作

- 提高驗證頻率
- 聯絡 degraded signers
- 立即執行 recovery checks
- 準備 signer rotation 或 address renewal
- 限制非必要 execution

### 6.2 Critical 動作

- 凍結非必要支出
- 優先遷移到新 signer set 或新 address
- 啟動更高審查強度
- 縮短監控間隔
- 若存在 emergency-only execution path，預先啟動準備

### 6.3 Emergency 動作

- 啟用預先授權的 recovery path
- 若 `m` 還勉強可達，只允許搬到 recovery target
- 若 `m` 已不可達，轉入 governance、法律或組織性 recovery process
- 保留完整 incident evidence

## 7. Address Renewal 與其他 Recovery Paths

emergency response 不應假設只有一種機制。

實務選項包括：

### 7.1 Lockout 前的正常 Rotation

若 `m` 仍可達，就應提早 rotation。

建議動作：

- 建立新的 signer set
- 建立新的 address 或 settlement target
- 在容量跌破 `m` 前先完成資產遷移

### 7.2 預先授權的 Emergency Recovery Path

deployment 可以定義一條比正常 execution 更窄的特殊路徑。

例子：

- 只能搬到固定 recovery address
- 只允許單一 asset class
- 套用更嚴格的 review 或 timelock

這條路徑不應變成一般支出的隱藏繞道。

### 7.3 Governance 或 Organizational Recovery

若實際簽章能力已低於 `m`，單靠一般 cryptographic control 可能已不夠。

此時系統可能需要：

- governance escalation
- legal recovery process
- organizational reconstitution
- 接受舊 target 已不可再用

## 8. 常見 Failure Cases

### 8.1 等太久才處理

系統已偵測到退化，但直到 `effective_signers < m` 才行動。

結果：

- 失去正常 renewal path

### 8.2 對 Degraded Signers 過度樂觀

operators 一直假設 degraded signers 仍等同 effective。

結果：

- 真實安全緩衝被高估

### 8.3 沒有定義 Emergency Path

deployment 已知道自己進入危機，但沒有事前 agreed 的處置路徑。

結果：

- 延誤、混亂與不一致行動

### 8.4 忽略 Common-mode Failure

多個 signers 因單一共享依賴一起失效。

結果：

- 名義上的冗餘其實只是幻覺

## 9. 必要控制

- 預先定義風險狀態門檻
- 定期計算 effective signer capacity
- 在 `warning` 時觸發明確 review
- 在 `critical` 時要求 action plan
- 在 `emergency` 時保留密封證據
- 讓 emergency paths 比正常 execution paths 更窄
- 明確連結 signer rotation、address renewal 與 policy updates

## 10. 實務判準

最正確的 renew address 或 rotate signer set 時機，通常不是 lockout 之後。

而是當系統還能誠實地說：

- 「我們現在還能安全搬移，但再拖可能就不行了」

如果 deployment 缺少這個決策點，它其實離不可逆失敗太近。 
