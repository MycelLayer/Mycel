# Minimal Mycel Host Bootstrap

狀態：design draft

這份筆記描述一個最小可行的本地 bootstrap，它可以承載 Mycel as a signed, on-demand runtime substrate。

目標不是定義一個完整的 operating system distribution。

而是定義一個最小、常駐、可信的 host，讓它可以：

- 安全開機
- 建立 trust
- 抓取缺少的已簽章模組
- 驗證它們
- 在受限制的 runtime 中執行它們

相關文件：

- `DESIGN-NOTES.signed-on-demand-runtime-substrate.*`：較大的執行模型與整體 substrate framing
- `DESIGN-NOTES.dynamic-module-loading.*`：單一 signed module 的 artifact 與載入規則
- `DESIGN-NOTES.future-software-ecosystem-on-mycel-runtime-substrate.*`：這套 host 與 substrate 若成為主流後的生態後果
- `DESIGN-NOTES.app-signing-model.*`：artifact 與 execution trust layers 的分層

## 0. 這份文件在系列中的定位

這份文件回答的核心問題是：

- 「若要承載 signed on-demand runtime substrate，本地最小可信 host 必須常駐哪些能力？」

它主要處理：

- boot layer、resident host layer、on-demand module layer 的分層
- verifier、fetcher、cache manager、sandbox runtime 等最小常駐元件
- 從安全開機到 fetched module admission 的 host boot flow

它刻意不主講：

- 單一 module schema、metadata 欄位與 module cache key 細節
- substrate 作為整體軟體模型的長篇 framing
- 這套架構若普及後的市場、產品與治理結構

閱讀順序上，它位在兩篇之間：

- 先用 `DESIGN-NOTES.signed-on-demand-runtime-substrate.*` 理解整體模型
- 再用這篇收斂「哪些東西不能被按需抓取，必須常駐」
- 最後回到 `DESIGN-NOTES.dynamic-module-loading.*` 看單一 module 怎麼被驗證與載入

## 1. 目標

定義最小的本地常駐 host，同時仍讓 Mycel-based system 可以表現成 fetch-on-demand 的 runtime substrate。

保持常駐：

- 只保留開機、驗證、抓取、快取、沙盒化與執行所需的最小程式碼

允許在需要前不存在：

- application logic
- renderers
- transformers
- 高層 policy helpers
- 大部分 app-specific runtime behavior

## 2. 為什麼本地 Bootstrap 不可避免

如果本地完全是空的，就不可能安全地參與 signed on-demand execution model。

一定要先有某些 trusted code 常駐，來負責：

- 啟動機器
- 建立網路
- 驗證 signatures 與 hashes
- 執行本地 execution policy
- 提供 execution sandbox

所以正確的設計目標不是：

- 完全沒有 local runtime

而是：

- 最小、但仍可信且能安全接納遠端 artifacts 的 local runtime

## 3. 三層 Host 模型

這個 minimal host 可以拆成三層理解。

### 3.1 Boot Layer

責任：

- firmware handoff
- bootloader execution
- root-of-trust initialization

這層建立：

- 第一批本地受信任程式碼是什麼
- 哪些 public keys 或 trust anchors 內建在本地

### 3.2 Resident Host Layer

責任：

- networking
- verification
- fetch 與 cache
- sandbox runtime hosting
- module policy enforcement

這一層就是 minimal Mycel host bootstrap 的核心。

### 3.3 On-Demand Module Layer

責任：

- application-specific behavior
- rendering
- transformation
- optional extension logic

這一層應該可替換、可抓取，而且在不需要時大多可以不存在。

## 4. 最小可行的常駐元件

常駐 host 大概只需要六個永遠存在的元件。

### 4.1 Boot 與 Trust Anchor

用途：

- 安全啟動
- 內建 signer trust roots
- update chain continuity

至少應定義：

- trusted public keys
- version 或 rollback policy
- 若有的話，則包含 local host identity

### 4.2 Tiny Host Core

用途：

- process 或 task isolation
- memory management
- 本地裝置與 filesystem mediation
- network stack access

它不需要是完整功能的桌面 OS。

它只需要足夠承載 verifier、fetcher、cache 與 runtime 即可。

### 4.3 Verifier

用途：

- hash verification
- signature verification
- artifact metadata validation
- runtime-target compatibility checks

這部分必須常駐且可信。

如果 verifier 太早也變成 on-demand module，整個 trust boundary 就會崩掉。

### 4.4 Fetcher

用途：

- 解析 module identities
- 下載缺少的 artifacts
- retry 與 mirror logic
- 本地 artifact staging

fetcher 可以很小，但必須可靠，且要理解 policy。

### 4.5 Cache Manager

用途：

- content-addressed blob storage
- offline reuse
- eviction of unused modules
- pinning of critical modules

這能讓系統比較像 verified execution cache，而不是傳統軟體安裝。

### 4.6 Sandboxed Runtime

用途：

- 載入已驗證的 portable modules
- 執行 resource limits
- 透過窄版 host API 暴露 capabilities

建議第一版 runtime 仍是：

- `WASM`

## 5. 建議的第一個 Runtime 形狀

第一版 host 最好只支援：

- 一種 portable module format
- 一個 runtime engine
- 一個 capability boundary

這意味著：

- 一開始不要同時支援多種 execution formats
- 一開始不要支援 native binary plugins
- 一開始不要支援無限制的 scripting environments

單一 `WASM` runtime 搭配嚴格 host API，仍是最乾淨的起點。

## 6. 建議的 Boot Flow

完整 host 啟動流程可以是：

1. firmware 把控制權交給 bootloader
2. bootloader 驗證並載入 resident host image
3. resident host 初始化 trust anchors、networking 與 local storage
4. host 載入本地 pinned policy 或 bootstrap manifest
5. host 判斷需要哪些 module identities
6. 缺少的 modules 被抓取並暫存
7. verifier 檢查 signature、hash、runtime target 與 policy
8. 通過核准的 modules 被快取並在 sandbox runtime 中啟動

這樣可讓 execution admission 從第一個 fetched artifact 出現時就保持明確。

## 7. 最小 Capability Surface

第一版 host 應只暴露很少的 capabilities。

例如：

- `read_document`
- `read_view_state`
- `write_render_output`
- `write_local_cache`
- `request_network_fetch`
- `emit_diagnostics`

第一版 host 應避免暴露：

- 任意 filesystem access
- 任意 subprocess creation
- module 內任意 outbound network access
- 直接 native library loading

## 8. Host 不該做什麼

這個 minimal bootstrap 不應試圖一次解決所有 systems 問題。

它不應：

- 第一天就試圖取代完整 general-purpose OS
- 執行未簽章 native code
- 讓 partial code fragments 在完整驗證前就開始執行
- 把 network fetch 視為自動 execution approval
- 把 verifier logic 與不可信 fetched code 混在一起

它的任務比較窄：

- 成為最小、但安全的 admission 與 execution host

## 9. 儲存模型

本地 storage model 應優先採用：

- content-addressed blobs
- pinned critical modules
- 可 eviction 的 non-critical cached modules
- persistent audit logs

在這個模型裡，installation 不再是主要單位。

主要單位會變成：

- trusted host image
- signed module metadata
- signed 或 hash-bound 的 module blobs
- local cache entries

## 9. Offline 與 Recovery 行為

實務 host 應明確定義三種狀態。

### 9.1 Online and Warm

host 可以抓取缺少 modules，也可以重用 cached modules。

### 9.2 Offline but Warm

host 雖然不能抓新 modules，但仍能執行已經在本地 pin 或 cache 的 artifacts。

### 9.3 Offline and Cold

host 本地沒有所需 module 的 cache。

在這種狀態下，execution 應安全失敗，而不是退回未簽章行為。

## 10. 最好的第一個 MVP

最小且現實的 MVP，可能還不是新的硬體層 OS image。

更可能是：

1. 一個跑在 Linux 上的 `mycel-host` process
2. 內建 verifier
3. 內建 fetcher 與 content-addressed cache
4. 內建 `WASM` runtime
5. 窄版 capability API

這樣已經能拿到大部分架構收益，同時避免太早承擔完整 custom mini-OS 的操作成本。

## 11. 後續演進路徑

如果這個模型被證明有價值，可以分三步走。

### Step 1

Linux-hosted minimal Mycel runtime process

### Step 2

帶有更小 trusted host stack 的 dedicated appliance image

### Step 3

更專門化的 mini-OS 或 unikernel-style deployment

這個順序能在保留長期方向的同時降低風險。

## 12. 開放問題

- host image 的哪些部分必須可更新，哪些部分應該被 pin 住？
- trust anchors 應該是 device-local、profile-local，還是 deployment-local？
- 哪些 modules 必須被 pin 住，才能保證 offline continuity？
- network stack 有多少應屬於 resident host，而不是 admitted system module？
- host core 最小可以縮到什麼程度，才不會讓 debugging 與 recovery 變得不實用？
