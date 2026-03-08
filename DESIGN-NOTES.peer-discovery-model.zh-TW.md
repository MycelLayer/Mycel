# Peer Discovery Model

狀態：design draft

這份文件描述 Mycel 系統應如何在不假設單一全球 discovery 模式的前提下，發現、分類並保留 peers。

核心設計原則是：

- peer discovery 應有邊界且受 policy 約束
- transport policy 應限制哪些 peers 可被使用
- discovery sources 應明確且可列舉
- 不同 deployment modes 應支援不同的 discovery 範圍

## 0. Goal

讓 Mycel 節點可以找到並維持有用的 peers，同時讓 discovery 行為與 transport、anonymity、governance 及 deployment constraints 保持一致。

這份文件定義：

- discovery 中的 peer roles
- address classes
- discovery sources
- admission 與 retention 規則
- deployment modes
- 常見 failure cases

## 1. Discovery Roles

不是每種節點都需要相同的 discovery 行為。

### 1.1 Reader Nodes

Reader nodes 通常只需要 bounded discovery（有邊界的節點發現）。

典型需求：

- 找到一個或多個可同步的 peers
- 刷新 accepted state
- 在需要時替換 stale peers

### 1.2 Governance-maintainer Nodes

Governance-maintainer nodes 需要更穩定的 discovery。

典型需求：

- 抓取目前 document heads
- 發布 governance 相關狀態
- 維持可靠的 peer 關係

### 1.3 Signer Nodes

Signer nodes 應採用狹窄且明確的 discovery。

典型需求：

- 維持已知 peer set
- 避免機會式的廣域網路探索
- 保持運作穩定與 policy 對齊

### 1.4 Runtime Nodes

Runtime nodes 通常應採用該部署中最嚴格的 discovery policy。

典型需求：

- 從已知 peers 讀取 accepted state
- 避免廣泛且噪音大的網路探索

## 2. Peer Address Classes

Peer discovery 應明確分類 address。

建議的 address classes：

- `clearnet:host-port`
- `tor:onion-v3`
- `restricted:bootstrap-alias`
- `local:manual-peer`
- `relay:transport-ref`

不同 address classes 不應被靜默混用。

部署 profile 應決定哪些 classes 是 preferred、allowed 或 forbidden。

## 3. Discovery Sources

Peer discovery 應依賴明確的來源。

### 3.1 Local Bootstrap List

一份靜態的本地 peer 或 alias 清單。

特性：

- 可預期
- 容易理解
- 適合 restricted 或 Tor-oriented deployments

### 3.2 Peer-provided Manifest Discovery

Peers 可透過 manifests 或本地 peer catalogs 宣告額外 peers 或 served topics。

特性：

- 更動態
- 有利於 mesh 擴張
- 需要更強的過濾

### 3.3 Out-of-band Trusted Introduction

使用者或 operator 透過執行中的網路之外的方式取得 peer 資訊。

例子：

- QR code
- signed document
- operator-managed config

特性：

- 信任度高
- 自動化低

### 3.4 Federation List

某個部署可維持一份明確的 membership list。

特性：

- 穩定
- 易於治理
- 彈性較低

## 4. Admission Rules

新發現的 peer 不應在未經檢查下自動成為 active peer。

建議的 admission checks：

1. address class 被 active deployment profile 允許
2. transport policy 允許連到該 address class
3. peer 能回應有效的 wire-compatible session
4. capabilities 符合本地 role 需求
5. 該 peer 不在本地 denylist 上

可選檢查：

- topic compatibility
- profile compatibility
- minimum reliability history

## 5. Ranking and Retention

被發現的 peers 應被刻意排序並保留。

建議 ranking inputs：

- successful session rate
- recent object availability
- transport compatibility
- role compatibility
- last successful sync time

建議 retention states：

- `candidate`
- `active`
- `degraded`
- `quarantined`
- `expired`

節點不應永遠保留所有發現過的 peers。

## 6. Deployment Modes

我建議三種實際可行的 discovery modes。

### 6.1 Public Mesh Discovery

特徵：

- peer set 較廣
- discovery sources 較動態
- 對 peer churn（節點流動）容忍較高

允許來源：

- local bootstrap
- peer-provided discovery
- 可選的 public seed infrastructure

取捨：

- 連通性更高
- 攻擊面與 metadata 暴露更大

### 6.2 Restricted Federation Discovery

特徵：

- peer set 大多預先已知
- federation 或 operator list 為主要來源
- 幾乎不做機會式擴張

允許來源：

- local bootstrap
- federation list
- trusted introduction

取捨：

- 穩定且易治理
- 擴張彈性較低

### 6.3 Tor-oriented Bounded Discovery

特徵：

- onion-first addressing
- 狹窄的 bootstrap set
- 明確的 transport constraints
- 不允許靜默 clearnet fallback

允許來源：

- local Tor-aware bootstrap list
- Tor-routed manifest discovery
- trusted introduction

取捨：

- 匿名姿態較好
- peer graph 較窄，恢復較慢

## 7. Discovery Flow

最小 bounded discovery 流程如下：

1. 載入設定好的 bootstrap sources
2. 分類 candidate peer addresses
3. 依 active deployment profile 過濾
4. 嘗試建立 transport-compatible session
5. 驗證 wire compatibility 與必要 capabilities
6. 將 peer 標記為 `candidate` 或 `active`
7. 隨時間更新 ranking 與 retention state

## 8. Refresh and Rotation

Peer discovery 是持續性的，不是一次性動作。

建議 refresh 行為：

- 定期重試 degraded peers
- 對超出 policy 限制仍不可用的 peers 做 expire
- 從允許的 discovery sources 補入替代 peers
- 對關鍵角色節點保留一個小而穩定的 active set

## 9. Failure Cases

### 9.1 Stale Bootstrap Entry

- 標記為 degraded 或 expired
- 不能無限迴圈重試而不做 backoff

### 9.2 Transport Mismatch

- 對 active profile 拒絕該 peer
- 保留本地 reason code

### 9.3 Capability Mismatch

- 對該 role 不可升成 active
- 若對其他 role 有用，才可保留為 candidate

### 9.4 Manifest-expansion Abuse

- 不可自動信任所有 manifests 裡廣播的 peers
- 仍然要對這些新 peers 套 admission checks

## 10. Minimal First-client Rules

對第一個可互通 client，我建議：

- 一份明確的 bootstrap list
- 明確的 address-class filtering
- 不要把 peer advertisements 一律升為 active peer
- bounded active peer set
- 可見的 peer state（`candidate`、`active`、`degraded`、`expired`）

## 11. Open Questions

- 後續是否應把 peer-provided discovery 升成 normative wire feature，還是維持 deployment-local？
- 對 v0.1 clients 來說，topic compatibility 是否應變成 discovery admission 的必要條件？
- Tor-oriented discovery 與 restricted federation discovery 應維持分開 profiles，還是共享一個更窄的 base model？
