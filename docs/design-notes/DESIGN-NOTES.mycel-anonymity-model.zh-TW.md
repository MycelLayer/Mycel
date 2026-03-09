# Mycel Anonymity Model

狀態：design draft

這份文件描述 Mycel 系統應如何把 anonymity（匿名性）理解成一個多層次屬性，而不是單一 transport setting（傳輸設定）。

核心設計原則是：

- transport anonymity（傳輸匿名）是必要但不充分的
- protocol metadata（協議 metadata）必須被刻意最小化
- client 與 runtime 的行為仍然可能使使用者去匿名化
- anonymity 與 auditability（可審計性）之間必須明確平衡

## 0. Goal

讓 Mycel 部署能在不假裝「單一工具就能提供完整匿名」的前提下，降低 network-identity leakage（網路身分洩漏）、metadata correlation（中繼資料關聯）與 local attribution risk（本地歸因風險）。

這份文件不為所有部署定義一個強制匿名模式。

它主要定義：

- 匿名性的層次
- 威脅面
- 建議控制方式
- 現實限制

## 1. Anonymity Is Multi-layer

對 Mycel 來說，匿名性至少要從四層來看。

### 1.1 Transport Layer

這一層處理封包如何在 peers 之間移動。

例子：

- Tor
- onion services（洋蔥服務）
- transport relays（傳輸中繼）
- VPN 或其他網路繞送機制

傳輸匿名可以幫助隱藏來源 IP，但它本身無法消除內容、時間模式或長期識別子的洩漏。

### 1.2 Protocol Metadata Layer

這一層處理 Mycel 訊息與物件本身暴露了什麼。

例子：

- sender identifiers（發送者識別）
- 穩定的 node IDs
- timestamps
- document references
- signer references
- profile references

若協議 metadata 高度可關聯，僅有傳輸匿名仍然很弱。

### 1.3 Client and Runtime Layer

這一層處理本地軟體本身如何運作。

例子：

- caching
- local account binding（本地帳號綁定）
- logging
- device identifiers
- runtime receipts
- effect execution patterns（副作用執行模式）

即使 transport 已匿名化，client 與 runtime 的行為仍可能暴露長期身分。

### 1.4 Replication and Governance Layer

這一層處理可複製歷史與 accepted state 如何隨時間被關聯。

例子：

- 同一把 key 重複簽章
- 穩定的 maintainer identities
- 固定的 signer-set membership
- 重複出現的同步時間模式

Mycel 的可驗證歷史很有價值，但它也會形成長期關聯風險。

## 2. Threat Model

一個重視匿名性的 Mycel 部署至少應考慮以下風險：

- network observers（網路觀察者）關聯來源 IP 與時間模式
- peers 關聯穩定 sender identifiers 的長期行為
- client 透過 logs、cache 或 secrets 洩漏身分
- runtime 把 accepted events 與外部 payment 或 effect 系統綁在一起
- 重複簽章將某個使用者或 signer 與長期活動關聯
- 根據同步頻率或抓物件模式進行 traffic analysis（流量分析）

## 3. Tor Is Helpful but Not Sufficient

Tor 應被視為傳輸層匿名工具，而不是完整的匿名模型。

Tor 可以幫助：

- 隱藏直接的 IP 層來源資訊
- 降低直接 peer-to-peer 位址暴露
- 讓流量來源更難被直接觀察

但 Tor 不能單獨解決：

- 穩定的協議識別子
- 可關聯的治理金鑰
- timing correlation（時間關聯）
- 本地 logging 或 runtime 洩漏
- 應用層帳號綁定

對 Mycel 來說，正確模型應該是：

- Tor 可以是一種 transport option
- 匿名性仍然取決於 metadata discipline（metadata 紀律）與本地行為

## 4. Metadata Minimization

若部署想要更強的匿名性，就應降低不必要的 metadata 暴露。

建議控制方式：

- 除非必要，避免暴露長期穩定的 node identifiers
- 當 active profile 不需要時，避免暴露 sender fields
- 在不影響必要功能時，降低 timestamp precision（時間戳精度）
- 避免把真實世界帳號參照塞進可複製物件
- 將 public object references 與本地 session identifiers 分開

核心原則是：

- 只複製驗證、路由或治理真正需要的 metadata

## 5. Role Separation

部署不應假設同一個 identity 必須扮演所有角色。

建議分離：

- reader identity 與 maintainer identity 分開
- maintainer identity 與 signer identity 分開
- signer identity 與 effect runtime identity 分開
- anonymous reading 與 public governance participation 分開

這可以降低跨層關聯。

取捨：

- 分離越強，匿名性越好
- 分離越強，操作複雜度越高

## 6. Client Hardening

client 行為很容易破壞網路匿名性。

建議控制方式：

- 限制持久化本地 logs
- 為不同 identities 分離本地 profiles
- 避免把 wallet、payment 或 sensor identifiers 塞入可複製 records
- 避免在 accepted deployment profile 之外發送 application telemetry（應用遙測）
- 支援本地 cache 清理或 compartmentalized storage（分艙式儲存）

若 client 把 anonymous reading 與 authenticated payment 或 governance operations 混在同一個本地 identity context，匿名性就會下降。

## 7. Runtime Hardening

runtime 是重大匿名風險來源，因為它會和外部系統互動。

建議控制方式：

- 將 external effect execution 與 reader identities 分開
- 避免發布不必要的 executor 細節
- 在 receipts 中最小化 runtime-specific identifiers
- 盡可能分離 payment、sensor 與 governance runtimes
- 避免把外部服務帳號 ID 與 Mycel identities 直接綁在一起

runtime 應只暴露審計真正需要的最小執行證據。

## 8. Replication Strategy

Replication 可以增加耐久性，但也可能提高可見性。

建議控制方式：

- 讓 reader nodes 只抓取其需要的 object families
- 支援 role-specific replication policies（角色特定複製策略）
- 不要強迫所有 peers 都鏡像所有敏感 app-layer records
- 保持 accepted-state verification，而不要求所有本地 artifacts 都普遍可見

部署可以選擇：

- 較廣的 replication 以換取 audit
- 較窄的 replication 以換取 anonymity

這個取捨應被明確化。

## 9. Governance Tradeoffs

Mycel 的 governance 與 audit 模型天然會產生 attribution pressure（歸因壓力）。

例子：

- signed governance signals
- 固定 signer sets
- accepted-head traces
- disbursement receipts

部署必須決定以下兩者中更偏好哪一邊：

- public accountability（公開可追責）
- operational anonymity（運作匿名）

這兩者並不完全相容。

建議做法：

- 依 role 與 profile 定義匿名期待
- 不要假裝 governance-signing identities 預設就是匿名的

## 10. Deployment Tiers

我建議三個實際可行的匿名部署層級。

### 10.1 Basic Anonymous Transport

特徵：

- 流量透過 Tor 或同等 transport indirection（傳輸繞送）
- 幾乎沒有 protocol-level metadata reduction（協議層 metadata 縮減）

取捨：

- 最容易部署
- 對 metadata correlation 抵抗較弱

### 10.2 Metadata-aware Anonymous Deployment

特徵：

- 匿名 transport
- 最小化 sender metadata
- 降低 timestamp precision
- 更強的 role separation

取捨：

- 匿名性更好
- 部署複雜度更高

### 10.3 Hardened Anonymous Deployment

特徵：

- 匿名 transport
- metadata minimization
- compartmentalized clients
- 分離 runtimes
- role-specific replication
- 明確的 local-hardening 規則

取捨：

- 最強的實務匿名性
- 最高的操作成本

## 11. Minimal First-client Rules

對第一個具匿名意識的 client，我建議：

- 支援可代理的 transport，例如 Tor
- 將 anonymous reading 與 signer 或 payment operations 分開
- 避免不必要的長期本地識別子
- 預設不發布 executor details
- 清楚說明哪些動作會破壞匿名假設

## 12. Non-goals

這份文件不主張：

- 完美匿名
- 能抵抗全域流量分析
- public governance signers 預設匿名
- 當使用者混用匿名與具名工作流時仍能保證匿名

## 13. Open Questions

- Mycel 是否應定義一個明確的 anonymous deployment profile，還是把匿名策略留給各部署自行決定？
- 哪些 metadata 欄位在所有可互通 profiles 中是真正必要的？
- 在 fund 與 governance workflows 中，我們應該用多少匿名性去換 audit 可見性？
