# 專案命名規則

狀態：目前有效的對外命名使用指引

這份文件用來定義 `Mycel` 與 `MycelLayer` 在 repo 與公開表面中的使用方式。

## 1. 短規則

- `Mycel` 保留為正式的專案、協定、repo 與實作名稱。
- `MycelLayer` 作為對外公開定位名稱，用在需要向新讀者快速說明 Mycel 是什麼的場景。

建議的第一句標準說法是：

> MycelLayer 是 Mycel 的對外名稱，用來表達它是一層面向可驗證文本歷史、受 profile 治理的 accepted reading，以及去中心化複製的協定層。

## 2. 為什麼要這樣拆

- `Mycel` 已經是 repo、Rust workspace、協定文件與程式碼識別字裡穩定使用的內部名稱。
- `MycelLayer` 能降低公開表面的撞名風險，也更直接說清楚這是一個「協定層」。
- 這種拆法能保留實作與規格上的連續性，同時讓對外敘事更清楚。

## 3. 正式名稱與對外名稱

### 3.1 何時用 `Mycel`

以下情境使用 `Mycel`：

- repo 名稱與 GitHub URL
- crate 名稱、package 名稱、binary 名稱與程式碼識別字
- protocol、wire、profile、fixture、simulator 文件
- 實作或規格導向的 issue 標題
- CLI 輸出與測試名稱

建議例子：

- `Mycel is a Rust-based protocol stack...`
- `Mycel object verification`
- `Mycel protocol v0.1`

### 3.2 何時用 `MycelLayer`

以下情境使用 `MycelLayer`：

- 首頁 hero 文案
- 面向新讀者的 README 開頭定位句
- repo description、social preview 文案與簡短公開介紹
- grant note、support page 與對外說明資料
- 主要在說明 Mycel 想補上的空缺，而不是協定細節的 explainer

建議例子：

- `MycelLayer is the public-facing name for Mycel.`
- `MycelLayer is a protocol layer for verifiable text history...`

## 4. 混合使用時的寫法

如果同一份文件同時出現兩個名稱：

1. 只有在該文件屬於對外表面時，才先引入 `MycelLayer`
2. 立刻把它接回 `Mycel`
3. 後續技術細節一律優先回到 `Mycel`，除非整段都仍是公開定位文案

建議寫法：

- `MycelLayer is the public-facing name for Mycel.`
- `Mycel is a Rust-based protocol stack...`

這樣可以避免讀者誤以為 `MycelLayer` 與 `Mycel` 是兩個不同系統。

## 5. 不要這樣做

- 不要把 repo、crate、binary 或協定文件標題從 `Mycel` 改成 `MycelLayer`
- 不要把規格文字改成 `MycelLayer protocol`、`MycelLayer wire format` 或 `MycelLayer object`
- 不要在 protocol、wire、schema、fixture 或 design-note 內部深處到處換成 `MycelLayer`，除非那段明確是在講對外命名
- 不要把 `MycelLayer` 包裝成和 `Mycel` 分離的另一個產品線、公司或網路

## 6. 各表面指引

### 6.1 README 與 landing pages

- 標題可以維持 `Mycel`，保留既有連續性
- 副標或第一句可以使用 `MycelLayer`，前提是該段是面向外部新讀者
- 第一屏就應把兩者關係說清楚

建議格式：

- 標題：`Mycel`
- 副標或第一句：`MycelLayer is the public-facing name for Mycel...`

### 6.2 協定與設計文件

- 除了少數在說明公開定位的註記外，全文維持 `Mycel`
- 技術精確性優先於品牌文案

### 6.3 Grant 與 support 文件

- 開頭摘要、support asks 與對外說明優先使用 `MycelLayer`
- 當需要指向 repo artifact、協定範圍或實作狀態時，再切回 `Mycel`

### 6.4 社群與分享表面

以下項目優先使用 `MycelLayer`：

- GitHub repo description
- social preview 標題或副標
- 公開 profile bio
- 提供連結預覽用的 page metadata

## 7. 翻譯指引

- `zh-TW`：`MycelLayer` 不翻譯，說明時寫成 `Mycel 的對外名稱` 或 `Mycel 的公開定位名稱`
- `zh-CN`：`MycelLayer` 不翻譯，說明時寫成 `Mycel 的对外名称` 或 `Mycel 的公开定位名称`
- 不要自行發明 `菌層`、`菌絲層` 或其他在地化品牌替代名

## 8. 推進順序

建議的落地順序：

1. 先建立這份命名規則文件
2. 再更新 README 開頭、首頁 hero 文案與 support page
3. 視需要更新 repo description 與 social preview 文案
4. 協定與實作識別字維持不變

## 9. 目前預設

在公開文案尚未全面更新前，請把這份文件視為目前的決策邊界：

- `Mycel` 仍是所有技術面上的正式名稱
- `MycelLayer` 已核准用於公開定位文案
