# Dynamic Module Loading for Mycel Apps

狀態：design draft

這份筆記提議一種保守的方式，讓 Mycel-based app 可以在執行時載入可執行模組，並在缺少模組時即時下載。

核心建議是：

- 把可執行程式碼視為獨立建模、可簽章的 artifact
- 預設優先使用沙盒化的 `WASM` 模組，而不是 native binaries
- 先驗證模組身分與 policy，再允許執行
- 只授予明確 capabilities，而不是讓模組直接取得宿主環境的 ambient access

相關文件：

- `DESIGN-NOTES.signed-on-demand-runtime-substrate.*`：runtime-substrate 的整體 framing 與執行模型
- `DESIGN-NOTES.minimal-mycel-host-bootstrap.*`：承載這種模型所需的最小常駐 host
- `DESIGN-NOTES.future-software-ecosystem-on-mycel-runtime-substrate.*`：若這套模型擴大後的生態與產品後果
- `DESIGN-NOTES.app-signing-model.*`：object signing、release signing 與 execution-evidence signing 的分層差異
- `DESIGN-NOTES.mycel-app-layer.*`：app-layer behavior 應如何位於 protocol core 之上
- `DESIGN-NOTES.mycel-full-stack-map.*`：更完整的分層地圖

## 0. 這份文件在系列中的定位

這份文件回答的核心問題是：

- 「單一 runtime module 應如何被建模、抓取、驗證、核准與載入？」

它主要處理：

- module metadata 與 blob 的 artifact 拆分
- runtime fetch 與 load flow
- module-level capability 與 admission 邊界
- 為什麼預設應選 `WASM`，而不是 native dynamic loading

它刻意不主講：

- 整個 Mycel runtime substrate 的總體 framing
- resident host 應有哪些最小常駐元件
- 這套模型成為主流後的軟體生態變化

如果讀者在問的是：

- 「整體模型到底像什麼？」請先看 `DESIGN-NOTES.signed-on-demand-runtime-substrate.*`
- 「本地最小可信 host 到底要常駐哪些東西？」請看 `DESIGN-NOTES.minimal-mycel-host-bootstrap.*`
- 「這種模式若擴大，軟體市場與 app store 會怎麼變？」請看 `DESIGN-NOTES.future-software-ecosystem-on-mycel-runtime-substrate.*`

## 1. 目標

讓 Mycel app 可以：

- 發現所需的 runtime module 缺失
- 在需要時即時抓取缺少的模組
- 把它當成已簽章、content-addressed 的 artifact 來驗證
- 在受限制的 runtime 中執行

同時保留：

- object-level verifiability
- 可重現的 module identity
- 明確的 trust 與 capability 邊界
- 事後可審計到底跑了哪份程式碼

預設避免：

- 任意 native code execution
- 隱含的宿主權限
- 執行未驗證的 script fragments

## 2. 為什麼需要獨立模型

下載普通 Mycel content，和下載可執行邏輯，並不是同一件事。

可執行邏輯會改變 trust model，因為它可能：

- 影響本地 confidentiality
- 影響本地 integrity
- 改變 app 如何解讀狀態
- 產生非決定性的 side effects

因此系統應把 code 視為第一級的 signed artifact，並給它比一般 content objects 更嚴格的 admission 規則。

## 3. 建議的執行方向

大致上有三種實作選擇。

### 3.1 沙盒化 `WASM` 模組

建議作為預設方向。

好處：

- 跨平台攜帶性較好
- 比 native code 更容易做 sandbox
- 適合搭配 capability-based host API
- 驗證與 cache 模型比較清楚

取捨：

- 需要先定義 host API 與 runtime embedding layer

### 3.2 受限制的 Script Runtime

可作為較短期的起步方案。

例如：

- Lua
- Rhai
- 其他 embedded DSL

好處：

- 一開始的實作較小

取捨：

- 攜帶性保證較弱
- 更容易漂移成 host-specific behavior
- 對較豐富模組的上限較低

### 3.3 Native Binary 或 Dynamic Library Loading

不建議作為預設。

原因：

- supply-chain risk 最強
- sandboxing 最難
- 跨平台表現最差
- 最容易讓 ambient host compromise 發生

如果未來真的要支援 native modules，也應把它視為獨立、較高風險的部署模式，而不是 Mycel app 的預設基線。

## 4. 建議的 Artifact 拆分

不要把可執行 code 當成普通 content object 裡的無結構附件。

建議改成兩種明確 artifact。

### 4.1 `module` Metadata Object

這個 object 描述：

- module identity
- version
- runtime target
- entry points
- required capabilities
- expected code hash
- fetch hints
- author 或 release signature

### 4.2 `module_blob` Artifact

這個 artifact 包含：

- 真正的 `WASM` payload，或其他被允許的 bytecode 形式

app 應驗證：

- blob hash 是否與 metadata object 一致
- metadata object signature 是否有效
- runtime target 是否被本地支援

這樣的拆分有助於：

- 鏡像 blobs
- 以 content hash 做快取
- 讓多個 metadata objects 重用相同 code
- 清楚審計「核准的是什麼」與「實際執行的是什麼」

## 5. 建議的 Metadata 形狀

可參考的欄位：

- `module_id`
- `name`
- `version`
- `runtime`
- `entry`
- `code_hash`
- `capabilities`
- `resource_limits`
- `fetch_uris`
- `signature`

可考慮的 resource-limit 欄位：

- `max_memory_mb`
- `max_fuel`
- `max_output_bytes`
- `network_policy`
- `filesystem_policy`

一開始不一定要把完整 schema 放進 protocol core。

它可以先作為 app-layer schema，由已簽章的 Mycel objects 承載。

## 6. Runtime Fetch 與 Load Flow

建議的執行流程：

1. 某個 document、view、profile 或 app manifest 參照一個 `module_id`
2. app 檢查本地 cache 是否已經有對應 module blob
3. 若缺少，app 便從被允許的 Mycel 或外部位置抓取 metadata object 與 blob
4. app 驗證 metadata signature、code hash、runtime 相容性，以及本地 policy admission
5. app 把驗證過的 blob 放進 module cache
6. app 在沙盒 runtime 中 instantiate 該模組
7. app 記錄 execution metadata，供之後 audit

重要規則：

- app 可以即時下載缺少的 module，但必須以完整、已簽章的 module artifact 為單位
- 在完整性驗證完成前，不應先執行任何 partial fragments

## 7. Segment 與 Chunk Fetching

如果未來大型 blobs 需要 chunked transfer，chunking 應維持在 transport 層，而不是 execution identity。

建議規則：

- chunks 可以逐步抓取
- execution identity 仍然是完整驗證過的 module blob
- 在最終 hash 與 signature 檢查通過之前，app 不得執行模組

這樣可以避免把「code segment download」變成定義不完整的 partial-execution 模型。

## 8. Capability 模型

模組不應直接取得不受限的宿主權限。

相反地，宿主應只提供窄版 capability API。

例如：

- `read_document`
- `read_revision_history`
- `read_view_state`
- `write_render_output`
- `write_local_cache`
- `emit_diagnostics`
- `request_network_fetch`

建議的 default-deny 行為：

- 不提供任意 filesystem access
- 不提供任意 subprocess execution
- 不提供任意 outbound network access
- 不允許再動態載入其他 native libraries

## 9. Trust 與 Governance 邊界

Mycel 的 verifiability 本身，不等於某個 module 就應該被信任來執行。

因此 app 應分開做兩層檢查：

1. integrity check
   - module object 與 blob 是否真實且未被竄改？
2. execution-admission check
   - 這個 signer、module family 與 capability request 是否符合本地或 profile policy？

可行的 admission 模型：

- 本地 signer allowlist
- 綁在 profile 上的 signer policy
- governance recommendation 加上本地最終核准

建議的短期基線：

- 本地 allowlist，加上明確 capability grant

## 10. Cache 與 Versioning

本地 app 應維護：

- `module_id -> metadata`
- `code_hash -> blob path`
- 以 module hash 與 runtime version 為 key 的 compiled cache

建議的 version 規則：

- 參照時 pin exact version
- 不要自動升級 major versions
- 只要 active state 還有參照，就保留舊版 cached blobs

cache 應是 content-addressed，而不只是 name-addressed。

## 10. Audit 與 Replay

如果 module 會影響 rendering、transformation 或外部行為，app 至少應記錄：

- `module_id`
- `version`
- `code_hash`
- runtime identity
- granted capabilities
- 相關 input object IDs
- 若適用，則記錄 output artifact hash

這不保證能做到完美 deterministic replay。

但它可以讓後續 review 與 incident analysis 強得多。

## 11. 第一版建議的 Non-goals

第一版應避免：

- 任意 shell script execution
- 自動執行下載下來的 native binaries
- 沒有明確 policy 的 module-to-module 遞迴依賴載入
- 在完整 blob 驗證前就開始 partial execution
- 完全由 governance 驅動、沒有本地 admission 的自動執行

## 12. 建議的 MVP

實務上第一版可以先限制在：

1. 單一 sandbox runtime，最好是 `WASM`
2. 單一 signed module metadata object type
3. 單一 content-addressed blob type
4. 一小組 capability surface
5. 本地 module cache
6. 對 module execution 的 audit logging

這樣就足以支援 app-layer renderer、transformer 或 policy helper，而不用讓 Mycel 一開始就承諾 native plugin loading。

## 13. 開放問題

- module metadata 應完全停留在 app-layer schema，還是未來要形成 protocol-level conventions？
- profile policy 應能推薦具體 modules，還是只應核准某些 signer classes？
- 對會影響 accepted-state derivation 的 modules，Mycel 應要求多高程度的 determinism？
- 模組啟動後，是否允許外部 network fetch，還是只能透過 host-mediated fetch requests？
