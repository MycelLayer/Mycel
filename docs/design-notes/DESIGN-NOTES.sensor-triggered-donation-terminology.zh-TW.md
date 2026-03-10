# Sensor-triggered Donation 中文術語對照表

狀態：working glossary

這份對照表服務於下列文件的中文閱讀與撰寫一致性：

- `DESIGN-NOTES.sensor-triggered-donation.zh-TW.md`
- `PROJECT-INTENT.zh-TW.md`

目標不是重寫 protocol core 術語，而是讓這個 case 在中文討論時有較穩定的用語。

## 1. 基本原則

- 涉及 protocol core 的固定術語，優先保留 repo 既有正式寫法。
- 涉及 app/profile 架構的工作詞彙，優先給出一個推薦中文，再附上保留英文的情況。
- 欄位名、enum 值、物件型別名若已出現在 schema 或例子中，正文可譯，程式或資料欄位中仍保留英文。
- 若中文翻譯會造成語義誤導，保留英文並補一個短中文解釋。

## 2. 術語對照

| English term | 建議中文 | 補充說明 |
| --- | --- | --- |
| app layer | 應用層 | 指 profile 之上的應用語意與記錄模型，不是 protocol core。 |
| profile | profile / 規則組 | 若在正式技術語境，建議保留 `profile`；需要白話時可說「規則組」。 |
| fixed profile | 固定 profile | 強調選擇規則不可臨時改動。 |
| consent profile | consent profile / 同意規則組 | 這裡的 `consent` 不只是同意按鈕，而是一組預先授權條件。 |
| trigger | trigger / 觸發條件 | 若是一般敘述可譯成「觸發條件」；欄位名保留 `trigger_*`。 |
| trigger eligibility | 觸發資格 | 指事件是否達到可進入 funding 評估的門檻。 |
| accepted state | accepted state / 已接受狀態 | 不建議硬翻成「共識狀態」，因為這裡不是全網共識。 |
| accepted head | accepted head / 已接受 head | 正式文件維持 `accepted head`，必要時補中文。 |
| user-state event | 使用者狀態事件 | 指從 session 導出的高階事件，不是原始訊號。 |
| derived event | 導出事件 | 表示由 runtime 從 evidence 摘要出來的結果。 |
| session record | session 記錄 | `session` 在這個 case 比「場次」更穩，建議保留。 |
| meditation-session record | 靜坐 session 記錄 | 這是目前最直接對應 `meditation -> fund` 的說法。 |
| evidence summary | 證據摘要 | 指可驗證摘要，不是原始感測資料。 |
| raw signal | 原始訊號 | 指不應直接進入 Mycel replicated state 的資料。 |
| sensor runtime | sensor runtime / 感測 runtime | 以 `runtime` 保留系統語氣，避免誤譯成單一程式。 |
| payment runtime | payment runtime / 支付 runtime | 負責外部 payment side effects。 |
| effect layer | effect layer / 外部效果層 | 指外部觀測與支付動作，不在 core replay 內。 |
| donation pledge | donation pledge / 捐助承諾 | 還不是直接付款，而是承諾或待確認狀態。 |
| payment intent | payment intent / 支付意圖 | 表示系統準備進入支付流程。 |
| manual-confirmation | 人工確認 | 建議正文譯成「人工確認」，欄位值保留英文。 |
| pre-authorized-payment | 預先授權支付 | 指事先同意的支付模式。 |
| settlement receipt | 結算收據 | 用於表示外部 payment 或 settlement 結果。 |
| failure receipt | 失敗收據 | 表示外部執行失敗後的可審計記錄。 |
| dispute record | 爭議記錄 | 使用者提出異議的記錄。 |
| revoke | 撤回 | 通常用於撤回既有授權。 |
| pause | 暫停 | 與 revoke 不同，暫停後仍可能恢復。 |
| funding bounds | 資金邊界 | 包括金額上限、頻率限制、冷卻時間等。 |
| payout mode | 撥付模式 | 指 `pledge`、`manual-confirmation`、`pre-authorized-payment` 等模式。 |
| long-lived history container | 長期歷史容器 | 用來解釋 `document` 在這個 case 裡不是傳統文件。 |
| document family | document family / 文件家族 | 在這個 case 裡指一組長期記錄流，不一定是散文文件。 |
| auditability | 可稽核性 | 若是對外說明，也可寫成「可審計性」，但 repo 目前較常用 audit 語氣。 |
| replay | replay / 重播驗證 | 若要完整說明，可寫成「重播驗證」。 |
| side-effect free | 無 side effect | 指 replay 不應直接造成外部付款或裝置動作。 |
| pseudonymous user reference | 假名化使用者參照 | 比「匿名」更精確，表示仍可穩定關聯但不直接暴露真實身份。 |

## 3. 高風險易混術語

以下幾組詞最容易混淆，建議固定區分：

| 不要混在一起 | 建議區分 |
| --- | --- |
| consent 與 trigger | `consent` 是事前授權；`trigger` 是事後事件是否達標。 |
| event 與 payment | 事件只提供資格訊號，不直接等於付款。 |
| pledge 與 settlement | `pledge` 是承諾或待確認；`settlement` 是外部支付或結算結果。 |
| profile 與 policy | `profile` 是一組固定評估規則；`policy` 可是其中一部分的具體約束。 |
| document 與 prose file | `document` 在 Mycel 裡是長期歷史容器，不必然是文字稿。 |

## 4. 目前推薦寫法

若要在繁中正文裡直接描述這個 case，建議優先用：

- 靜坐 session 記錄
- 導出事件
- 固定 profile
- 觸發資格
- donation pledge / 捐助承諾
- 支付意圖
- 結算收據
- 長期歷史容器

避免一開始就寫得太口語，例如：

- 「靜坐就拿錢」
- 「打坐自動收款」
- 「感測到就直接付款」

這些說法會把 app/profile 條件、accepted state 與外部結算的邊界全部壓扁。
