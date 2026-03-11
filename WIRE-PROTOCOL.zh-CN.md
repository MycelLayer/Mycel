# Mycel 传输协议 v0.1（草案）

语言：简体中文 | [English](./WIRE-PROTOCOL.en.md) | [繁體中文](./WIRE-PROTOCOL.zh-TW.md)

## 0. 范围

本文件定义 Mycel 节点的传输层消息格式与最小同步流程。

v0.1 目标：

- 定义稳定的 wire 信封
- 定义 v0.1 同步消息集的规范性字段
- 保持实现中立、技术化、可互通

## 1. 兼容条件

节点若符合以下条件，即可视为 v0.1 wire 兼容：

1. 可产生与解析第 2 节 envelope
2. 实现 `HELLO`、`MANIFEST`、`HEADS`、`WANT`、`OBJECT`、`BYE`、`ERROR`
3. 在接受前验证 envelope 签名以及对象哈希/签名
4. 若宣告 `snapshot-sync`，则实现 `SNAPSHOT_OFFER`
5. 若宣告 `view-sync`，则实现 `VIEW_ANNOUNCE`

## 2. 消息信封

所有 wire 消息 MUST 使用以下信封：

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

必需字段：

- `type`：消息种类
- `version`：固定为 `mycel-wire/0.1`
- `msg_id`：唯一消息 ID
- `timestamp`：RFC 3339 时间戳
- `from`：发送端节点 ID
- `payload`：消息主体
- `sig`：对不含 `sig` 的 canonical envelope 做签名

每一种消息类型的 wire-message 签名规则，以第 3.1 节为规范性定义。
信封的 canonicalization MUST 依 `PROTOCOL.zh-CN.md` Appendix A 执行。

## 3. 消息类型

v0.1 定义以下消息种类：

- `HELLO`
- `MANIFEST`
- `HEADS`
- `WANT`
- `OBJECT`
- `SNAPSHOT_OFFER`
- `VIEW_ANNOUNCE`
- `BYE`
- `ERROR`

## 3.1 Wire Message Signature Matrix（规范）

所有 v0.1 wire 消息都需要 envelope signature。

| 消息类型 | Envelope `sig` | 签名 payload |
| --- | --- | --- |
| `HELLO` | required | 省略 `sig` 后的 canonical envelope |
| `MANIFEST` | required | 省略 `sig` 后的 canonical envelope |
| `HEADS` | required | 省略 `sig` 后的 canonical envelope |
| `WANT` | required | 省略 `sig` 后的 canonical envelope |
| `OBJECT` | required | 省略 `sig` 后的 canonical envelope |
| `SNAPSHOT_OFFER` | required | 省略 `sig` 后的 canonical envelope |
| `VIEW_ANNOUNCE` | required | 省略 `sig` 后的 canonical envelope |
| `BYE` | required | 省略 `sig` 后的 canonical envelope |
| `ERROR` | required | 省略 `sig` 后的 canonical envelope |

规则：

1. 接收端 MUST 拒绝任何缺少 `sig` 的 v0.1 wire 消息。
2. `from` 所对应的节点密钥 MUST 能验证对不含 `sig` 的 canonical envelope 所做的签名。
3. Envelope `sig` 只验证传输元数据；它不能取代 `OBJECT.body` 内部的 object-level signature。
4. `sig` 字段本身 MUST NOT 纳入签名 envelope payload。

## 4. HELLO

`HELLO` 用于启动连接并宣告能力。

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

必需 `payload` 字段：

- `node_id`
- `capabilities`
- `nonce`

## 5. MANIFEST

`MANIFEST` 用于宣告节点目前提供的同步表面。
它是 wire 消息摘要，不是内容寻址的协议对象。

```json
{
  "type": "MANIFEST",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:manifest-001",
  "timestamp": "2026-03-08T20:00:10+08:00",
  "from": "node:alpha",
  "payload": {
    "node_id": "node:alpha",
    "capabilities": ["patch-sync", "snapshot-sync", "view-sync"],
    "topics": ["text/core", "text/commentary"],
    "heads": {
      "doc:origin-text": ["rev:c7d4", "rev:b351"]
    },
    "snapshots": ["snap:44cc"],
    "views": ["view:9aa0"]
  },
  "sig": "sig:..."
}
```

必需 `payload` 字段：

- `node_id`
- `capabilities`
- `heads`

字段规则：

- `heads` 是 `doc_id -> 非空 canonical revision ID 数组` 的 map
- 每个 head list MUST 只包含唯一 revision ID
- 每个 head list SHOULD 以按按字典序递增发送，方便稳定重放
- 若有 `snapshots`，其内容 MUST 是 canonical snapshot ID
- 若有 `views`，其内容 MUST 是 canonical view ID

## 6. HEADS

`HEADS` 用于宣告一个或多个文件目前的 heads。

```json
{
  "type": "HEADS",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:heads-001",
  "timestamp": "2026-03-08T20:00:30+08:00",
  "from": "node:alpha",
  "payload": {
    "documents": {
      "doc:origin-text": ["rev:c7d4", "rev:b351"],
      "doc:governance-rules": ["rev:91de"]
    },
    "replace": true
  },
  "sig": "sig:..."
}
```

必需 `payload` 字段：

- `documents`
- `replace`

字段规则：

- `documents` 是非空的 `doc_id -> 非空 canonical revision ID 数组` map
- 每个 head list MUST 只包含唯一 revision ID
- 每个 head list SHOULD 以按按字典序递增发送，方便稳定重放
- 若 `replace` 为 `true`，表示发送端宣告：对于这些列出的文件，其 head set 应取代先前的广播内容
- 若 `replace` 为 `false`，表示发送端宣告：这些 head set 只是增量提示

## 7. WANT

`WANT` 依 canonical object ID 请求缺少的对象。
在 v0.1，这些 ID 是带类型前缀的内容寻址 ID，例如 `rev:<object_hash>` 或 `patch:<object_hash>`。
像 `doc_id`、`block_id` 这类逻辑 ID 不是合法的 `WANT` 目标。

```json
{
  "type": "WANT",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:want-001",
  "timestamp": "2026-03-08T20:01:00+08:00",
  "from": "node:beta",
  "payload": {
    "objects": ["rev:c7d4", "patch:a12f"],
    "max_items": 256
  },
  "sig": "sig:..."
}
```

必需 `payload` 字段：

- `objects`：非空的 canonical object ID 列表

## 8. OBJECT

`OBJECT` 用于传送单一对象内容。

```json
{
  "type": "OBJECT",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:obj-001",
  "timestamp": "2026-03-08T20:01:02+08:00",
  "from": "node:alpha",
  "payload": {
    "object_id": "patch:a12f",
    "object_type": "patch",
    "encoding": "json",
    "hash_alg": "blake3",
    "hash": "hash:...",
    "body": {"type": "patch", "...": "..."}
  },
  "sig": "sig:..."
}
```

必需 `payload` 字段：

- `object_id`
- `object_type`
- `encoding`
- `hash_alg`
- `hash`
- `body`

字段含义：

- `object_id`：canonical 类型化 object ID，以 `<object_type-prefix>:<hash>` 重建
- `hash`：canonicalized `body` 的原始摘要值
- `body`：未经传输包装前的 canonical 对象内容

对 v0.1 的内容寻址对象类型：

- `patch` 使用 `patch_id`
- `revision` 使用 `revision_id`
- `view` 使用 `view_id`
- `snapshot` 使用 `snapshot_id`

接收端 MUST：

1. 重算 `hash(body)` 并比对 `hash`
2. 依 `object_type` 与 `hash` 重建预期的 `object_id`，并与 `object_id` 比对
3. 若 `body` 含有该类型的导出 object-ID 字段，必须验证其与 `object_id` 一致
4. 依 `PROTOCOL.zh-CN.md` 中的规范性 object signature rules 验证对象层签名
5. 验证通过才可入库

## 9. SNAPSHOT_OFFER

`SNAPSHOT_OFFER` 用于宣告某个 snapshot 可通过 `WANT` 取得。

```json
{
  "type": "SNAPSHOT_OFFER",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:snap-001",
  "timestamp": "2026-03-08T20:02:00+08:00",
  "from": "node:alpha",
  "payload": {
    "snapshot_id": "snap:44cc",
    "root_hash": "hash:snapshot-root",
    "documents": ["doc:origin-text"],
    "object_count": 3912,
    "size_bytes": 1048576
  },
  "sig": "sig:..."
}
```

必需 `payload` 字段：

- `snapshot_id`
- `root_hash`
- `documents`

字段规则：

- `snapshot_id` MUST 是 canonical snapshot ID
- `documents` MUST 是非空的 `doc_id` 数组
- 若有 `object_count`，其值 MUST 是非负整数
- 若有 `size_bytes`，其值 MUST 是非负整数
- 当接收端后续取得对应的 Snapshot 对象时，其 `snapshot_id` 与 `root_hash` MUST 与此 offer 一致

## 10. VIEW_ANNOUNCE

`VIEW_ANNOUNCE` 用于宣告某个已签名的 View 对象可通过 `WANT` 取得。

```json
{
  "type": "VIEW_ANNOUNCE",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:view-001",
  "timestamp": "2026-03-08T20:02:05+08:00",
  "from": "node:alpha",
  "payload": {
    "view_id": "view:9aa0",
    "maintainer": "pk:community-curator",
    "documents": {
      "doc:origin-text": "rev:c7d4"
    }
  },
  "sig": "sig:..."
}
```

必需 `payload` 字段：

- `view_id`
- `maintainer`
- `documents`

字段规则：

- `view_id` MUST 是 canonical view ID
- `documents` MUST 是非空的 `doc_id -> canonical revision ID` map
- 取得到的 View 对象之 `view_id`、`maintainer`、`documents` MUST 与此 announcement 一致

## 11. BYE

`BYE` 用于正常关闭连接 session。

```json
{
  "type": "BYE",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:bye-001",
  "timestamp": "2026-03-08T20:02:10+08:00",
  "from": "node:alpha",
  "payload": {
    "reason": "normal-close"
  },
  "sig": "sig:..."
}
```

必需 `payload` 字段：

- `reason`

建议 `reason` 值：

- `normal-close`
- `shutdown`
- `idle-timeout`
- `policy-reject`

## 12. 错误处理

解析或验证失败时，回传 `ERROR`：

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
    "detail": "Hash mismatch for object patch:a12f"
  },
  "sig": "sig:..."
}
```

必需 `payload` 字段：

- `in_reply_to`
- `code`

建议错误码：

- `UNSUPPORTED_VERSION`
- `INVALID_SIGNATURE`
- `INVALID_HASH`
- `MALFORMED_MESSAGE`
- `OBJECT_NOT_FOUND`
- `RATE_LIMITED`

## 13. 最小同步流程

1. 交换 `HELLO`
2. 交换 `MANIFEST` / `HEADS`
3. 接收端以 `WANT` 请求缺少的 ID
4. 发送端回传一个或多个 `OBJECT`
5. 接收端验证并入库
6. 可选择交换 `SNAPSHOT_OFFER` / `VIEW_ANNOUNCE`
7. 正常关闭时传送 `BYE`

## 14. 安全备注

- envelope 签名不能取代 object 层签名检查
- 依本地 policy 拒绝未签名或签名错误的控制消息
- 对重复无效流量套用 rate limit
- 保持传输与 acceptance 决策分离

## 15. 后续延伸

后续版本可扩充：

1. 大对象串流 / 分块传输
2. 压缩能力协商
3. capability 范围授权 token
4. replay 防护视窗与 nonce 规则
