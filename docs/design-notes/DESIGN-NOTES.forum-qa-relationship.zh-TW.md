# Forum 與 Q&A 的關係

狀態：design draft

這份筆記用來定義 Forum App Layer 與 Q&A App Layer 之間的關係。

## 0. 短答案

這兩份筆記彼此相關，但不是重複品。

- Forum App Layer 是較通用的 discussion model
- Q&A App Layer 是較窄的 resolution-oriented model
- 兩者在概念上都可以存在
- 實作上不需要同時往前推進兩條線

## 1. Forum App Layer 擅長什麼

Forum App Layer 是比較廣義的 discussion model。

它的重心是：

- board structure
- thread structure
- reply trees
- 顯式 moderation history
- 為 thread 與 board display 導出的 accepted reading

它更適合：

- 一般討論
- 長時間延續的 threads
- moderators 可見的 disputes
- 以 post 為單位的 visibility control
- 容許分叉的 conversation systems

## 2. Q&A App Layer 擅長什麼

Q&A App Layer 是比較窄的 resolution model。

它的重心是：

- 一個 question
- 多個 candidate answers
- 在 active profile 下導出一個 accepted answer
- 以 citations 為中心的 answer evaluation
- 顯式的 answer-selection traces

它更適合：

- knowledge-base workflows
- answer selection
- expert-response systems
- citation-heavy 的 answer review
- 比起開放討論，更在意單一 active result 的情境

## 3. 預設閱讀單位的差異

兩者最關鍵的差異，是 accepted reading 的單位不同。

對 Forum 來說：

- accepted reading 要回答的是：「這條 thread 或這個 board 現在應該怎麼顯示？」

對 Q&A 來說：

- accepted reading 要回答的是：「這個問題目前哪個答案被採用？」

光是這個差異，就足以讓兩份筆記同時成立。

## 4. 建議的專案解讀方式

建議的解讀方式是：

- Forum 應被視為較通用的 discussion app-layer example
- Q&A 應被視為較特化、偏向 resolution 的 app-layer example
- 未來 Q&A 可以實作成：
  - 和 Forum 並列的一組 specialized schema family，或
  - 建立在較廣義 Forum shape 之上的 constrained profile

目前這個 repo 還不需要先決定實作細節要採哪一種。

## 5. 近期建議

對近期的 design 與 fixture work，我建議：

1. 保留兩份筆記
2. 以 Forum 作為主要的 discussion example
3. 除非明確需要 accepted-answer model，否則先延後 Q&A 的 schema 或 fixtures

這樣可以維持設計空間清楚，同時避免太早展開兩條看似重疊的實作線。
