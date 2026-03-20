# Metadata Merge Boundaries

狀態：design draft

這份文件記錄 conservative merge-authoring flow 在 `M2` 階段目前已確立的
metadata merge 邊界。

它刻意保持很窄。目的不是重寫整個 merge profile，而是先把目前已落地的
metadata 行為整理成一份之後可以直接引用的短結論，方便接續
`content-variant` 工作或未來的 patch-model 擴充。

## 0. Scope

這份文件只涵蓋三種 metadata 情境：

1. 採用 non-primary metadata addition
2. 面對 non-primary metadata addition 時保留 primary variant
3. 在 resolved state 中移除 metadata

它不試圖重新設計完整 merge 規則。

## 1. Current Rule

目前 conservative metadata merge 的規則是：

1. 如果 resolved state 選擇 non-primary parent 的 metadata variant，回報
   `multi-variant`
2. 如果存在 non-primary 的 competing metadata alternative，但 resolved
   state 保留 primary metadata variant，也一樣回報 `multi-variant`
3. 如果 resolved state 移除了 primary metadata，回報
   `manual-curation-required`

這樣做的目的是：即使最後 resolved state 和 primary parent 一樣，metadata
上的 competing choice 仍然要保持可見。

## 2. Adopt Non-primary Add

情境：

- primary parent 沒有某個 metadata key
- non-primary parent 新增了該 key
- resolved state 採用了 non-primary 的值

目前處理：

- outcome: `multi-variant`
- materialization: 1 個 `set_metadata` op

原因：

- 這是一個真實的 competing parent choice
- 目前 patch model 可以直接表達這個結果

## 3. Keep Primary Over Add

情境：

- primary parent 沒有某個 metadata key
- non-primary parent 新增了該 key
- resolved state 保留 primary variant，讓該 key 繼續缺席

目前處理：

- outcome: `multi-variant`
- materialization: 0-op merge patch

原因：

- 這個 merge 依然做出了一個 competing parent choice
- 即使 resolved state 已經等於 primary parent，merge assessment 仍然應該把這個
  choice 顯示出來

這裡最重要的設計點是：`沒有產生 patch op` 不等於 `沒有 competing variant`。

## 4. Metadata Removal Boundary

情境：

- primary parent 含有某個 metadata key
- resolved state 把這個 key 移除

目前處理：

- outcome: `manual-curation-required`

原因：

- Mycel v0.1 目前只有 `set_metadata`，沒有 metadata deletion op
- replay 可以新增或覆寫 metadata key，但沒有正式的 deletion primitive
- 因為目前的 patch op set 不能忠實 materialize 這個 resolved state，所以這是
  patch-model boundary，而不只是單純還沒細分的 competing-variant case

## 5. Implication For `M2`

這讓目前的 metadata story 可以穩定地收斂成三條分支：

1. 採用 non-primary metadata add：已實作
2. 面對 non-primary metadata add 時保留 primary：已實作
3. 移除 metadata：目前明確不在 v0.1 的自動 materialization 範圍內

這也表示，後續 metadata follow-up 不應再把 removal 當成一般「尚未分類完整」
的 merge 缺口。

下一個真正需要決定的問題是：

1. 繼續讓 metadata deletion 保持在 v0.1 scope 之外
2. 或先新增一個狹窄的 metadata deletion op，再去擴大 removal cases
