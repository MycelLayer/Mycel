# Canonical Text Profile

狀態：design draft

這份文件描述一個中性的文本 profile，用於在 Mycel 之上承載具結構、可引用、可註解的典範文本 corpus（文獻集合）。

核心設計原則是：

- profile 應能描述穩定可引用的 reference texts（參考文本），而不預設特定內容領域
- text structure 與 citation anchors（引用錨點）應明確表示
- commentary、translation 與 reference layers 應與 root text 分離
- 多個 witnesses（文本見證）與 accepted reading profiles 應可並存，而不強迫全域單一版本

## 0. Goal

讓 Mycel 能承載需要以下能力的長期 reference texts：

- 穩定引用
- 章節化閱讀
- 多 witnesses 或 editions（版本）
- translation layers（翻譯層）
- commentary layers（註解層）
- accepted reading profiles（採信閱讀設定）

這份文件不定義一個對所有 deployment 都強制適用的 canon model。

它定義的是一個中性的 baseline（基底），可再由特定領域的 extensions（擴充）細化。

## 1. Scope

這個 profile 適合的 corpus 類型包括：

- scriptural collections（經典集合）
- legal codes（法典）
- philosophical canons（哲學典籍）
- historical primary texts（歷史原典）
- 其他以 reference 為導向的文獻

這個 profile 不特別適合：

- 類聊天內容
- 快速頻繁改動的短文
- 不需要穩定引用的一般 wiki 頁面

## 2. Core Design Rule

Root text 必須與所有 secondary layers（次級層）分開。

這表示：

- commentary 不應靜默覆蓋 root text
- translation 不應被視為與 source text 相同的 object
- explanatory notes（說明性註記）應保持可追責
- accepted reading state 可依 profile 不同而改變，但不應改寫 underlying witnesses

## 3. Text Families

一個 canonical corpus 應建模成一組彼此相關的 document types。

### 3.1 Work Record

定義概念上的作品本身。

建議欄位：

- `work_id`
- `title`
- `language_family`
- `corpus_id`
- `reference_scheme`
- `root_witnesses`

用途：

- 在不綁定單一 edition 的前提下識別作品
- 提供穩定的最高層 reference target

### 3.2 Witness Document

表示一個具體的文本 witness。

例子：

- source-language witness（原語文本）
- translation witness（譯本）
- critical edition witness（校勘版）
- curated reading witness（整理版閱讀本）

建議欄位：

- `witness_id`
- `work_id`
- `witness_kind`
- `language`
- `source_description`
- `text_document_id`
- `lineage_ref`

### 3.3 Text Document

承載實際的結構化文本內容。

這一層應由對應明確 citation anchors 與 section hierarchy 的 blocks 組成。

### 3.4 Commentary Document

承載次級詮釋或說明。

它應透過引用 root text anchors 來附著，而不是取代 root text。

### 3.5 Alignment Document

承載兩個或多個 witnesses 之間的對齊關係。

例子：

- 原文對譯本
- 版本對版本
- 段落對段落的對應

### 3.6 Citation Set

承載供 commentary、Q&A 或其他上層使用的穩定引用集合。

## 4. Structural Model

這個 text profile 應把 hierarchy（階層）明確表示出來。

建議層次：

- `corpus`
- `work`
- `section`
- `subsection`
- `text_unit`
- 可選的 `line_unit`

這些名稱刻意保持中性。

Deployment 可以透過 extension 把它映射成特定領域用語。

例子：

- `section` 可以對應到某種章節層
- `text_unit` 可以對應到某種段落或句節層
- `line_unit` 則只在需要精細逐行引用時使用

## 5. Citation Anchors

穩定引用是這個 profile 的核心理由之一。

每個 text document 都應支援明確的 anchors。

建議欄位：

- `anchor_id`
- `work_id`
- `witness_id`
- `anchor_path`
- `anchor_kind`
- `block_ref`

建議性質：

- 若只是小幅編輯性清理，anchor 應盡量能存續
- anchors 應明確表示，不應只靠視覺格式推斷
- anchor paths 應同時方便人類引用與機器處理

## 6. Witness and Edition Model

這個 profile 不應假設某個 text witness 在全域上具有唯一權威。

相反地：

- 同一個 work 可以有多個 witnesses
- 某個 deployment 可以偏好某個 accepted reading
- 另一個 deployment 也可以偏好另一個 accepted reading

這種模型比強行壓成單一 universal edition 更符合 Mycel 的多分支與 accepted-head 形狀。

## 7. Alignment Model

典範文本常常需要跨 witness 比對。

Alignment document 應支援：

- 一對一對應
- 一對多對應
- 部分重疊
- 無法對上的片段

建議欄位：

- `alignment_id`
- `source_witness_id`
- `target_witness_id`
- `alignment_units`
- `alignment_method`
- `confidence`

這個 profile 應把「不完美對齊」視為常態。

## 8. Secondary Layers

以下層次應與 root text 分離：

- commentary
- translation
- glossary
- index
- Q&A
- teaching 或 explanatory summaries（教學或說明摘要）

每個次級層都應引用 root anchors 或 alignment units，而不是透過複製內容暗示自己也擁有相同權威。

## 9. Accepted Reading Model

Mycel 的 accepted-head 模型在 canonical text 閱讀上的適用方式應是：

- root witness set 的歷史要被保存
- deployment 可發布 accepted reading profiles
- reader client 預設顯示一個 accepted reading
- 其他 witnesses 與 branches 仍然可見

這樣可以避免假裝所有讀者都必須共享同一個 universal reading state。

## 10. Reader Expectations

實作這個 profile 的 client 至少應支援：

- 閱讀 accepted text
- 顯示具 anchor 的文本結構
- 查看 alternative witnesses
- 開啟綁定 anchors 的 commentary
- 開啟綁定 anchors 的 citations
- 顯示目前 accepted reading 是由哪個 profile 選出的

## 11. Governance Boundaries

這個 profile 應區分：

- root text maintenance
- witness publication
- commentary publication
- accepted reading selection

這些角色可以重疊，但不應被靜默混成同一種權限。

## 12. Extension Strategy

Core profile 應保持中性。

特定傳統或領域的細節，應透過 extensions 表達，例如：

- 額外的 hierarchy labels
- 領域特有的 citation conventions（引用慣例）
- 特化的 alignment metadata
- 特定領域的 commentary classes

除非互通性明確需要，extension 不應重新定義這個中性基底。

## 13. Minimal First Version

這個 profile 的最小第一版 deployment 應要求：

- 一個 `work record`
- 一個或多個 `witness documents`
- 每個 witness 對應一個結構化 `text document`
- 明確的 citation anchors
- 可選的 commentary documents
- 可選的 alignment documents
- 一個 accepted reading profile

它不應要求：

- 完美的跨 witness 對齊
- commentary 的自動語義 merge
- 對單一 edition 的全域一致認可

## 14. Recommended Next Step

這份設計稿之後最實際的下一步是定義：

- 一份最小 canonical text schema
- 一套 anchor syntax（錨點語法）
- 一個含兩個 witnesses 與一層 commentary 的 example corpus

這樣這份 profile 才會具體到足以支撐第一版 reader client。
