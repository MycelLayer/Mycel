# Mycel Protocol Upgrade Philosophy

狀態：design draft

這份文件描述 Mycel 應如何演進 protocol、profile 與 design note，同時避免把核心規格變成快速漂移的目標。

核心設計原則是：

- protocol core 應保守演進
- profile 應承載較窄的 deployment 與 app-level 承諾
- design notes 應先吸收探索中的模型
- 新能力應優先放在外層，而不是先改核心協議

## 0. Goal

讓 Mycel 能持續成長，同時避免獨立實作彼此漂移、避免既有 object history 失效，並避免把每個實驗性想法都直接塞進 protocol core。

這份文件定義：

- protocol core 的建議升級姿態
- profile 的角色
- design note 的角色
- 各層之間的升級路徑
- 各層的相容性預期

## 1. Layered Upgrade Model

Mycel 應透過三層結構演進。

### 1.1 Protocol

Protocol 層定義獨立實作之間必須共享的規則。

例子：

- object model
- canonical serialization
- hashing 與 derived IDs
- signature rules
- revision replay
- wire envelope 與必要訊息語義

這一層的變更應稀少且保守。

### 1.2 Profile

Profile 定義建立在 protocol 之上的較窄、具版本的運作子集。

例子：

- Tor-oriented deployment profile
- strict Q&A profile
- fund auto-disbursement profile

Profile 應鎖定具體參數與行為，但不應重新定義 protocol validity。

### 1.3 Design Note

Design note 是探索層。

它適合承載：

- 候選模型
- 尚未解完的 tradeoffs（取捨）
- app-layer 實驗
- 尚未適合做成嚴格 conformance 的 deployment 想法

Design note 應刻意比 profile 或 protocol 更容易調整。

## 2. Core Stability Rule

Mycel 應優先維持穩定的 protocol core。

這表示：

- 避免頻繁修改 object validity rules
- 一旦採用 canonicalization rules，避免再重定義
- 除非必要，避免改變既有 wire messages 的語義
- 避免讓原本有效的歷史資料在之後突然變成無效

Protocol 應緩慢演進，因為互通性依賴的是共享不變量，而不只是共享方向。

## 3. Profile-first Expansion

新能力通常應先以 profile 或 design note 進入 Mycel，再考慮是否進入核心 protocol。

建議順序：

1. 先把想法寫成 design note
2. 再收斂成具版本的 profile
3. 觀察 profile 是否穩定且有實作價值
4. 只有在必要時，才考慮哪些部分應升級進 protocol

這樣能把實驗留在較便宜的外層。

## 4. Promotion Rules

### 4.1 Design Note -> Profile

當以下條件成立時，適合升級：

- 模型範圍已清楚
- required records 與 behaviors 已知
- 主要 tradeoffs 已被記錄
- 已有實際可行的第一版實作目標

### 4.2 Profile -> Protocol

當以下條件成立時，適合升級：

- 多個 profiles 都依賴同一條共享規則
- 獨立實作之間需要這條規則才能互通
- 這條規則已不再只是 app-specific 或 deployment-specific
- 把它留在 protocol 外的成本已高於凍結它的成本

預設答案仍應是：

- 除非有明確的互通性理由，否則先不要放進 protocol

## 5. Compatibility by Layer

不同層應有不同的相容性預期。

### 5.1 Protocol Compatibility

Protocol 應優先保護歷史有效性。

建議姿態：

- 一旦某個 object 在既有 protocol version 下有效，後續的編輯性清理不應靜默讓它失效
- 未來 protocol 修訂應優先採 additive clarification（加法式釐清），而不是不相容的重新解讀

這是 Mycel 裡最嚴格的相容區。

### 5.2 Profile Compatibility

Profile 預設不需要強 backward compatibility。

建議姿態：

- profile 應明確版本化
- 新版 profile 可以取代舊版
- deployment 可以一次只支援單一 profile version

Profile 應穩到可以實作，但也要保留可替換性。

### 5.3 Design Note Compatibility

Design note 應假設低相容性保證。

它是有結構的探索層，不是 deployment 承諾。

## 6. Versioning Posture

Mycel 應偏好：

- 緩慢的 protocol version 演進
- 明確的 profile versioning
- 快速的 design-note 迭代

這表示：

- protocol versions 應稀少且具意義
- profile versions 可以更多，且更貼近使用情境
- design notes 只要明確標示非規範性，就可以自由調整

## 7. Change Placement Checklist

當我們決定某個變更應放哪一層時，可先問：

1. 它是否影響 object validity 或 wire interoperability？
2. 獨立實作是否需要這條精確規則才能一致？
3. 這條規則是否主要屬於 app、deployment 或 governance？
4. 這個模型是否已成熟到可以凍結？

建議判斷：

- 若答案主要是互通性與不變量，可能應進 protocol
- 若答案主要是 deployment 或 app constraints，較可能屬於 profile
- 若仍在探索中，就應留在 design note

## 8. Deprecation and Replacement

Mycel 應優先採 replacement（取代）而不是 silent mutation（靜默改義）。

建議規則：

- 不要把舊 design note 靜默改成完全不同的模型而不標明
- 不要在不升版的情況下靜默改變 profile 的語義
- 不要在未明確版本化與 migration 討論前，重新解讀 protocol validity rules

這樣整套文件才可審計。

## 9. First-client Guidance

對第一版 client，最安全的實作目標是：

- 直接實作目前寫定的 protocol core
- 只支援一小組明確 profiles
- 不依賴尚未收斂的 design notes

這能降低實作漂移，也讓 repo 更容易往穩定方向收斂。

## 10. Recommended Mycel Upgrade Posture

對 Mycel 最建議的姿態是：

- 保守的 protocol core
- 具版本的 profile 演進
- 積極但受控的 design-note 實驗

簡單說：

- protocol 應扮演穩定基底
- profiles 應承載實際運作承諾
- design notes 應吸收持續中的新想法

這是讓 Mycel 保持可演進，同時不失穩定性的最乾淨路徑。
