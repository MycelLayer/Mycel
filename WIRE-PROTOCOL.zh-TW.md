# Mycel Wire Protocol v0.1（草案）

語言：繁體中文 | [English](./WIRE-PROTOCOL.en.md)

## 0. 範圍

本文件定義 Mycel 節點的傳輸層訊息格式與最小同步流程。

v0.1 目標：

- 定義穩定的 wire envelope
- 定義 `HELLO`、`WANT`、`OBJECT` 的最小欄位
- 保持實作中立、技術化、可互通

## 1. 相容條件

節點若符合以下條件，即可視為 v0.1 wire 相容：

1. 可產生與解析第 2 節 envelope
2. 實作 `HELLO`、`WANT`、`OBJECT`
3. 在接受物件前驗證雜湊與簽章

## 2. 訊息信封

所有 wire 訊息 MUST 使用以下信封：

```json
{
  "type": "HELLO",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:5f0c...",
  "timestamp": "2026-03-08T20:00:00+08:00",
  "from": "node:alpha",
  "payload": {},
  "sig": "sig:..."
}
```

必要欄位：

- `type`：訊息種類
- `version`：固定為 `mycel-wire/0.1`
- `msg_id`：唯一訊息 ID
- `timestamp`：RFC 3339 時間戳
- `from`：發送端節點 ID
- `payload`：訊息主體
- `sig`：對不含 `sig` 的 canonical envelope 做簽章

每一種訊息型別的 wire-message 簽章規則，以第 3.1 節為規範性定義。

## 3. 訊息類型

v0.1 定義以下訊息種類：

- `HELLO`
- `MANIFEST`
- `HEADS`
- `WANT`
- `OBJECT`
- `SNAPSHOT_OFFER`
- `VIEW_ANNOUNCE`
- `BYE`
- `ERROR`

## 3.1 Wire Message Signature Matrix（規範）

所有 v0.1 wire 訊息都需要 envelope signature。

| 訊息型別 | Envelope `sig` | 簽章 payload |
| --- | --- | --- |
| `HELLO` | required | 省略 `sig` 後的 canonical envelope |
| `MANIFEST` | required | 省略 `sig` 後的 canonical envelope |
| `HEADS` | required | 省略 `sig` 後的 canonical envelope |
| `WANT` | required | 省略 `sig` 後的 canonical envelope |
| `OBJECT` | required | 省略 `sig` 後的 canonical envelope |
| `SNAPSHOT_OFFER` | required | 省略 `sig` 後的 canonical envelope |
| `VIEW_ANNOUNCE` | required | 省略 `sig` 後的 canonical envelope |
| `BYE` | required | 省略 `sig` 後的 canonical envelope |
| `ERROR` | required | 省略 `sig` 後的 canonical envelope |

規則：

1. 接收端 MUST 拒絕任何缺少 `sig` 的 v0.1 wire 訊息。
2. `from` 所對應的節點金鑰 MUST 能驗證對不含 `sig` 的 canonical envelope 所做的簽章。
3. Envelope `sig` 只驗證 transport metadata；它不能取代 `OBJECT.body` 內部的 object-level signature。
4. `sig` 欄位本身 MUST NOT 納入簽章 envelope payload。

## 4. HELLO

`HELLO` 用於啟動連線並宣告能力。

```json
{
  "type": "HELLO",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:hello-001",
  "timestamp": "2026-03-08T20:00:00+08:00",
  "from": "node:alpha",
  "payload": {
    "node_id": "node:alpha",
    "agent": "mycel-node/0.1",
    "capabilities": ["patch-sync", "snapshot-sync", "view-sync"],
    "topics": ["text/core", "text/commentary"],
    "nonce": "n:01f4..."
  },
  "sig": "sig:..."
}
```

必要 `payload` 欄位：

- `node_id`
- `capabilities`
- `nonce`

## 5. WANT

`WANT` 依 canonical object ID 請求缺少的物件。
在 v0.1，這些 ID 是帶型別前綴的內容定址 ID，例如 `rev:<object_hash>` 或 `patch:<object_hash>`。
像 `doc_id`、`block_id` 這類邏輯 ID 不是合法的 `WANT` 目標。

```json
{
  "type": "WANT",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:want-001",
  "timestamp": "2026-03-08T20:01:00+08:00",
  "from": "node:beta",
  "payload": {
    "objects": ["rev:merge001", "patch:91ac"],
    "max_items": 256
  },
  "sig": "sig:..."
}
```

必要 `payload` 欄位：

- `objects`：非空的 canonical object ID 列表

## 6. OBJECT

`OBJECT` 用於傳送單一物件內容。

```json
{
  "type": "OBJECT",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:obj-001",
  "timestamp": "2026-03-08T20:01:02+08:00",
  "from": "node:alpha",
  "payload": {
    "object_id": "patch:91ac",
    "object_type": "patch",
    "encoding": "json",
    "hash_alg": "blake3",
    "hash": "hash:...",
    "body": {"type": "patch", "...": "..."}
  },
  "sig": "sig:..."
}
```

必要 `payload` 欄位：

- `object_id`
- `object_type`
- `encoding`
- `hash_alg`
- `hash`
- `body`

欄位語義：

- `object_id`：canonical 型別化 object ID，以 `<object_type-prefix>:<hash>` 重建
- `hash`：canonicalized `body` 的原始摘要值
- `body`：未經 transport 包裝前的 canonical 物件內容

對 v0.1 的內容定址物件型別：

- `patch` 使用 `patch_id`
- `revision` 使用 `revision_id`
- `view` 使用 `view_id`
- `snapshot` 使用 `snapshot_id`

接收端 MUST：

1. 重算 `hash(body)` 並比對 `hash`
2. 依 `object_type` 與 `hash` 重建預期的 `object_id`，並與 `object_id` 比對
3. 若 `body` 含有該型別的導出 object-ID 欄位，必須驗證其與 `object_id` 一致
4. 依 `PROTOCOL.zh-TW.md` 中的規範性 object signature rules 驗證物件層簽章
5. 驗證通過才可入庫

## 7. 錯誤處理

解析或驗證失敗時，回傳 `ERROR`：

```json
{
  "type": "ERROR",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:err-001",
  "timestamp": "2026-03-08T20:01:03+08:00",
  "from": "node:beta",
  "payload": {
    "in_reply_to": "msg:obj-001",
    "code": "INVALID_HASH",
    "detail": "Hash mismatch for object patch:91ac"
  },
  "sig": "sig:..."
}
```

建議錯誤碼：

- `UNSUPPORTED_VERSION`
- `INVALID_SIGNATURE`
- `INVALID_HASH`
- `MALFORMED_MESSAGE`
- `OBJECT_NOT_FOUND`
- `RATE_LIMITED`

## 8. 最小同步流程

1. 交換 `HELLO`
2. 交換 `MANIFEST` / `HEADS`
3. 接收端以 `WANT` 請求缺失 ID
4. 發送端回傳一個或多個 `OBJECT`
5. 接收端驗證並入庫
6. 可選擇交換 `SNAPSHOT_OFFER` / `VIEW_ANNOUNCE`
7. 正常關閉時傳送 `BYE`

## 9. 安全備註

- envelope 簽章不能取代 object 層簽章檢查
- 依本地 policy 拒絕未簽章或簽章錯誤的控制訊息
- 對重複無效流量套用 rate limit
- 保持 transport 與 acceptance 決策分離

## 10. 後續延伸

後續版本可擴充：

1. 大物件串流/分塊傳輸
2. 壓縮能力協商
3. capability 範圍授權 token
4. replay 防護視窗與 nonce 規則
