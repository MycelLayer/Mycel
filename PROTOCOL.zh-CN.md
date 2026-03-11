# Mycel 协议 v0.1

语言：简体中文 | [English](./PROTOCOL.en.md) | [繁體中文](./PROTOCOL.zh-TW.md)

## 0. 定位

Mycel 是一种具备以下特性的文本协议：

- Git 式版本模型
- P2P 复制
- 签名验证
- 多分支共存
- 不要求全局单一共识

它不是区块链，也不是 Git 复制品；它是为文字与知识内容而设计的去中心化、可分叉、可验证历史的协议。

适用场景包括：

- 长期文本
- 评注
- 宣言文件
- 社区章程
- 规范文件
- 去中心化 wiki
- 不易被删除的知识网络

## 1. 设计目标

Mycel 的设计目标：

1. **可验证历史**：所有被接受的变更都必须可追溯、可重放验证。
2. **去中心存活**：在没有单一服务器时，内容仍然可以被保存和同步。
3. **分支合法**：分叉是一级合法状态，不是错误。
4. **合并可选**：社区可发布已签名的治理 View，而阅读客户端依固定 profile 规则导出 accepted head。
5. **匿名可用**：作者可使用假名密钥，并应最小化元数据暴露。
6. **文本优先（v0.1）**：在 v0.1 以 block / paragraph 为主要操作单位。

## 2. 协议概念

Mycel 把数据拆成 6 种核心概念：

- **Document**：一条可长期演化且可重放的对象历史，不必然是传统文字文件
- **Block**：段落/区块
- **Patch**：一次修改
- **Revision**：某个可验证状态
- **View**：用来用于导出 accepted head 的已签名治理信号
- **Snapshot**：某一时刻的快照包

## 3. 基本原则

### 3.1 逻辑 ID 与 canonical object ID

Mycel 使用两种不同的标识符类别：

- **逻辑 ID**：文件状态内的稳定参照，例如 `doc_id` 与 `block_id`
- **Canonical object ID**：可复制对象的内容寻址 ID，例如 `patch_id`、`revision_id`、`view_id`、`snapshot_id`

逻辑 ID 属于应用层状态，MUST NOT 被解释为内容哈希。
Canonical object ID 由 canonical bytes 导出：

```text
object_hash = HASH(canonical_serialization(object_without_derived_ids_or_signatures))
object_id = <type-prefix>:<object_hash>
```

在 v0.1：

- `doc_id` 与 `block_id` 是逻辑 ID
- `patch_id`、`revision_id`、`view_id`、`snapshot_id` 是 canonical object ID
- 导出 ID 字段本身与 `signature` 字段 MUST NOT 纳入 hash 输入

这个拆分可避免自我递归哈希，也让传输参照保持明确。

### 3.2 签名必须存在

所有作者产生的 Patch、Revision、View 都必须具备数字签名。
所有 v0.1 对象类型的签名要求，以第 6.4 节为规范性定义。

### 3.3 多个 head 合法

同一个文件可以有多个 heads。

### 3.4 Accepted Head 由 Profile 治理，而非全局真理

所谓“默认采用版本”只是某个治理 View profile 的输出，不是全网唯一版本。
不同合法 profile 可以并存，但合规的阅读客户端 MUST 以固定的 protocol-defined profile 输入导出 active accepted head，而不是依本地偏好自由裁量。
这表示「采用」是某个固定 profile 下可复现的 selector 结果，不是在宣称整个网络已收敛到唯一强制版本。

### 3.5 传输与接受分离

节点可以接收某个 object，但不让它进入 profile-governed accepted-head 路径。
对象只有在完成完整验证并符合固定 selector profile 的条件后，才会影响 accepted-head selection。

## 4. 对象模型

### 4.1 Document

Document 定义一份文本的身份与基础设置。

```json
{
  "type": "document",
  "version": "mycel/0.1",
  "doc_id": "doc:origin-text",
  "title": "Origin Text",
  "language": "zh-Hant",
  "content_model": "block-tree",
  "created_at": 1777777777,
  "created_by": "pk:authorA",
  "genesis_revision": "rev:0ab1"
}
```

字段：

- `doc_id`：文件固定逻辑 ID，不是内容哈希
- `title`：标题
- `language`：语言
- `content_model`：内容模型，v0.1 固定为 `block-tree`
- `genesis_revision`：初始 revision

### 4.2 Block

Block 是最小文本结构单位。

```json
{
  "type": "block",
  "block_id": "blk:001",
  "block_type": "paragraph",
  "content": "起初没有终稿，只有传递。",
  "attrs": {},
  "children": []
}
```

`block_type` 可用值：

- `title`
- `heading`
- `paragraph`
- `quote`
- `verse`
- `list`
- `annotation`
- `metadata`

`block_id` 是文件状态内的逻辑 block 参照，不是内容哈希。

### 4.3 Patch

Patch 表示一次对文件的修改。

```json
{
  "type": "patch",
  "version": "mycel/0.1",
  "patch_id": "patch:91ac",
  "doc_id": "doc:origin-text",
  "base_revision": "rev:0ab1",
  "author": "pk:authorA",
  "timestamp": 1777778888,
  "ops": [
    {
      "op": "replace_block",
      "block_id": "blk:001",
      "new_content": "起初没有终稿，只有传递与再写。"
    },
    {
      "op": "insert_block_after",
      "after_block_id": "blk:001",
      "new_block": {
        "block_id": "blk:002",
        "block_type": "paragraph",
        "content": "凡被写下者，皆可再写。",
        "attrs": {},
        "children": []
      }
    }
  ],
  "signature": "sig:..."
}
```

Patch 的签名输入至少要包含：

- `type`
- `version`
- `doc_id`
- `base_revision`
- `timestamp`
- `author`
- `ops`

`patch_id` 是导出的 canonical object ID，格式为 `patch:<object_hash>`。
它 MUST 由省略 `patch_id` 与 `signature` 后的 canonical Patch 内容计算而得。

对 v0.1 的 genesis-state Patch 对象，`base_revision` MUST 使用固定 sentinel 值 `rev:genesis-null`。

### 4.4 Patch Operations

v0.1 建议只定义少量基本操作：

- `insert_block`
- `insert_block_after`
- `delete_block`
- `replace_block`
- `move_block`
- `annotate_block`
- `set_metadata`

范例：删除

```json
{
  "op": "delete_block",
  "block_id": "blk:009"
}
```

范例：评注

```json
{
  "op": "annotate_block",
  "block_id": "blk:001",
  "annotation": {
    "block_id": "blk:ann01",
    "block_type": "annotation",
    "content": "此段为社区常用的维护版本。"
  }
}
```

### 4.4.1 Trivial Change（规范）

在 Mycel v0.1 中，`trivial change` 指的是只改动编辑表面形式，而不改变文件结构、参照目标、metadata 语义、或预期语义的变更。

只有在以下条件全部成立时，某个 Patch 才 MAY 被分类为 trivial：

1. 每个 operation 都作用在同一文件状态谱系中的既有 block
2. 每个 operation 只能是以下之一：
   - 对既有 block 的 `replace_block`
   - 不改变目标 block 自身内容的 `annotate_block`
3. 没有任何 operation 改变 block 顺序、block parentage、block identity、或 block type
4. 没有任何 operation 插入、删除、或移动 block
5. 没有任何 operation 改变 metadata keys 或 metadata values
6. 没有任何 operation 以可能改变解读的方式修改 identifiers、revision references、URLs、numeric values、或 date/time literals
7. 结果文本只打算用于修正或正规化表面形式

典型的 trivial changes 包含：

- 明显 typo 修正
- 空白正规化
- 标点正规化
- 在语义不变前提下的大小写正规化
- 不改变评注主张的 annotation formatting 清理

以下不属于 trivial changes：

- 任何结构变更
- 任何插入、删除、或移动
- 任何对 `block_id` 的修改
- 任何对 metadata 语义的修改
- 任何可能合理改变解读的 wording change

Trivial-change classification 只具 advisory 性质。
它 MUST NOT 绕过一般 Patch 验证、Revision 验证、签名检查、merge 规则、或 `state_hash` 重算。

### 4.5 Revision

Revision 表示某个状态节点。
它不是全文本本身，而是「parent + patch 集合」形成的可验证状态。

```json
{
  "type": "revision",
  "version": "mycel/0.1",
  "revision_id": "rev:8fd2",
  "doc_id": "doc:origin-text",
  "parents": ["rev:0ab1"],
  "patches": ["patch:91ac"],
  "state_hash": "hash:state001",
  "author": "pk:authorA",
  "timestamp": 1777778890,
  "signature": "sig:..."
}
```

merge revision 范例：

```json
{
  "type": "revision",
  "version": "mycel/0.1",
  "revision_id": "rev:c7d4",
  "doc_id": "doc:origin-text",
  "parents": ["rev:8fd2", "rev:b351"],
  "patches": ["patch:a12f"],
  "state_hash": "hash:merged-state",
  "author": "pk:curator1",
  "timestamp": 1777780000,
  "merge_strategy": "semantic-block-merge",
  "signature": "sig:..."
}
```

`revision_id` 是导出的 canonical object ID，格式为 `rev:<object_hash>`。
它 MUST 由省略 `revision_id` 与 `signature` 后的 canonical Revision 内容计算而得。

### 4.5.1 Revision State Construction（规范）

为了让 v0.1 的 `state_hash` 可重算，Revision 状态建构规则如下：

1. `parents` 是有顺序的数组。
2. Genesis revision MUST 使用 `parents: []`。
3. 非 merge revision MUST 刚好有一个 parent。
4. 多 parent revision MUST 将 `parents[0]` 视为唯一的执行基底状态。
5. `parents[1..]` 只记录被合并的 ancestry；它们 MUST NOT 自动把内容带入结果状态。
6. 任何从次要 parents 采纳的内容，都 MUST 明确实体化在列出的 `patches` 中。
7. `patches` 是有顺序的数组，且 MUST 依数组顺序逐一套用。
8. Revision 所引用的每个 Patch，其 `doc_id` MUST 与该 Revision 相同。
9. 非 genesis Revision 所引用的每个 Patch，其 `base_revision` MUST 等于 `parents[0]`。对 genesis revision，所有引用的 Patch 都 MUST 使用 `base_revision = rev:genesis-null`。
10. 若任何引用的 Patch 缺失、无效、或无法决定性套用，该 Revision 即为无效。

这表示接收端不会为了重算 Revision 状态而重新执行 semantic merge 演算法。
接收端只会对执行基底状态重放有序的 Patch 操作。

### 4.6 View

View 是一个已签名的治理信号，用来声明某维护者在特定 policy body 下采用哪些 revisions。

```json
{
  "type": "view",
  "version": "mycel/0.1",
  "view_id": "view:9aa0",
  "maintainer": "pk:community-curator",
  "documents": {
    "doc:origin-text": "rev:c7d4",
    "doc:governance-rules": "rev:91de"
  },
  "policy": {
    "preferred_branches": ["community-mainline"],
    "accept_keys": ["pk:community-curator", "pk:reviewerB"],
    "merge_rule": "manual-reviewed"
  },
  "timestamp": 1777781000,
  "signature": "sig:..."
}
```

在 v0.1，View 不是终端使用者的偏好对象。
它是用来导出 profile-governed accepted head 的其中一个 selector 输入。

`view_id` 是导出的 canonical object ID，格式为 `view:<object_hash>`。
它 MUST 由省略 `view_id` 与 `signature` 后的 canonical View 内容计算而得。

### 4.7 Snapshot

Snapshot 用于快速同步。

```json
{
  "type": "snapshot",
  "version": "mycel/0.1",
  "snapshot_id": "snap:44cc",
  "documents": {
    "doc:origin-text": "rev:c7d4"
  },
  "included_objects": [
    "rev:c7d4",
    "patch:91ac",
    "patch:a12f"
  ],
  "root_hash": "hash:snapshot-root",
  "created_by": "pk:mirrorA",
  "timestamp": 1777782000,
  "signature": "sig:..."
}
```

`snapshot_id` 是导出的 canonical object ID，格式为 `snap:<object_hash>`。
它 MUST 由省略 `snapshot_id` 与 `signature` 后的 canonical Snapshot 内容计算而得。

## 5. 序列化与哈希

### 5.1 Canonical Serialization

在 hash 或签名之前，所有协议对象都 MUST 先转成 Appendix A 定义的 canonical JSON 形式。
同一套 canonicalization 规则也适用于 `state_hash` 计算所用的 state object，以及 [`WIRE-PROTOCOL.zh-CN.md`](./WIRE-PROTOCOL.zh-CN.md) 所引用的 wire envelope。

### 5.2 Hash

在 v0.1，同一个 network MUST 对 canonical object ID 与对象验证使用同一个固定哈希演算法。
预设建议为：

```text
hash = BLAKE3(canonical_bytes)
```

如果想保守，也可换成 SHA-256；但协议要固定，不可同网混用。

### 5.3 导出 ID 规则

对 v0.1 中任何内容寻址对象类型：

1. 对对象内容做 canonicalize
2. 省略导出 ID 字段（`patch_id`、`revision_id`、`view_id`、`snapshot_id`）
3. 省略 `signature`
4. 以网络固定的哈希演算法计算剩余 canonical bytes
5. 以 `<type-prefix>:<object_hash>` 重建导出 ID

接收端 MUST 拒绝任何内嵌导出 ID 与重算 canonical object ID 不一致的内容寻址对象。

### 5.4 State Hash Computation（规范）

在 v0.1，Revision 的 `state_hash` 依以下方式计算：

1. 解析执行基底状态：
   - 若 `parents` 为空，使用空状态 `{ "doc_id": <revision.doc_id>, "blocks": [] }`
   - 否则，载入 `parents[0]` 已完整验证的状态
2. 依数组顺序，把引用的 `patches` 重放到该执行基底状态上。
3. 产生结果文件状态，表示为 canonical state object：

```json
{
  "doc_id": "doc:origin-text",
  "blocks": [
    {
      "block_id": "blk:001",
      "block_type": "paragraph",
      "content": "...",
      "attrs": {},
      "children": []
    }
  ]
}
```

4. 以协议其他部分相同的 serialization 规则对该 state object 做 canonicalize。
5. 计算 `state_hash = HASH(canonical_state_bytes)`。

补充规则：

- 顶层 block 顺序 MUST 保留在结果 `blocks` 数组中。
- 子 block 顺序 MUST 保留在各自的 `children` 数组中。
- 已删除的 block MUST 不出现在结果状态中。
- 若要保留 multi-variant 结果，MUST 由套用后的 Patch 结果状态明确表达，而不能只从 parent ancestry 隐式推导。
- 接收端 MUST 拒绝任何宣告的 `state_hash` 与重算值不一致的 Revision。

## 6. 身分与签名

### 6.1 作者身份

Mycel 作者身份预设是**假名公钥身份**。

```text
author_id = pk:<public_key_fingerprint>
```

不是帐号，不是真名。

### 6.2 签名算法

v0.1 建议：

- 签名：Ed25519
- 密钥交换：X25519

### 6.3 身分模式

Mycel 支援 3 种：

- **Persistent pseudonym**：长期笔名
- **Rotating pseudonym**：定期换 key
- **One-time signer**：一次性作者

### 6.4 Object Signature Matrix（规范）

v0.1 的对象签名要求如下：

| 对象类型 | 签名状态 | 签署者字段 | 签名 payload |
| --- | --- | --- | --- |
| `document` | forbidden | 无 | 无 |
| `block` | forbidden | 无 | 无 |
| `patch` | required | `author` | 省略 `signature` 后的 canonical Patch |
| `revision` | required | `author` | 省略 `signature` 后的 canonical Revision |
| `view` | required | `maintainer` | 省略 `signature` 后的 canonical View |
| `snapshot` | required | `created_by` | 省略 `signature` 后的 canonical Snapshot |

规则：

1. 接收端 MUST 拒绝任何缺少 `signature` 的 v0.1 `patch`、`revision`、`view`、`snapshot` 对象。
2. 接收端 MUST 拒绝任何带有顶层 `signature` 字段的 `document` 或 `block` 对象。
3. 签署者字段所指向的密钥 MUST 能验证对应 canonical payload 的签名。
4. 对内容寻址对象类型，内嵌的导出 ID MUST 先与重算出的 canonical object ID 一致，签名验证才可成立。
5. `signature` 字段本身 MUST NOT 纳入签名 payload。

### 6.5 Object Signature Inputs（规范）

每一种需签名的 v0.1 对象，其签名 payload 都是「只省略 `signature` 字段后」的 canonical serialization。

这表示：

- `patch` 的签名覆盖 `patch_id`、`doc_id`、`base_revision`、`author`、`timestamp`、`ops`
- `revision` 的签名覆盖 `revision_id`、`doc_id`、`parents`、`patches`、`state_hash`、`author`、`timestamp`，以及任何宣告的 merge 字段
- `view` 的签名覆盖 `view_id`、`maintainer`、`documents`、`policy`、`timestamp`
- `snapshot` 的签名覆盖 `snapshot_id`、`documents`、`included_objects`、`root_hash`、`created_by`、`timestamp`

## 7. 节点模型

Mycel 节点可同时兼任多种角色。
在 v0.1，与 maintainer 有关的角色拆分如下：

1. **Author Node**：产生一般 patch / revision 对象
2. **Editor-Maintainer Node**：发布维护者级 patch / revision 对象，可建立新的 candidate heads
3. **View-Maintainer Node**：发布会影响 accepted-head selection 的 View 对象
4. **Mirror Node**：保存与提供内容
5. **Relay Node**：转发元数据与对象
6. **Archivist Node**：保存完整历史

同一把 key 或同一个 node MAY 同时持有 editor-maintainer 与 view-maintainer 角色。
仅持有 editor-maintainer 身分，MUST NOT 自动取得 selector weight。

## 8. P2P 同步层

Mycel 不要求全节点同步全部数据，支援 partial replication。

### 8.1 节点宣告：Manifest

每个节点可公布 manifest：

```json
{
  "type": "manifest",
  "version": "mycel/0.1",
  "node_id": "node:alpha",
  "topics": ["text/core", "text/commentary"],
  "heads": {
    "doc:origin-text": ["rev:c7d4", "rev:b351"]
  },
  "snapshots": ["snap:44cc"],
  "capabilities": ["patch-sync", "snapshot-sync", "view-sync"]
}
```

### 8.2 同步流程

第一次加入：

1. 节点取得初始 peers
2. 取得 manifest
3. 拉最近 snapshot
4. 补差额 patch / revision
5. 为一个或多个固定 profiles 建立 accepted-head 索引

日常更新：

1. 收到 head announcement
2. 检查本地是否缺对象
3. 以 canonical object ID 拉取缺失
4. 验 hash、验签名
5. 存入本地储存
6. 依固定 profile 规则重算 accepted heads

### 8.3 交换消息类型

v0.1 最小消息集：

- `HELLO`
- `MANIFEST`
- `HEADS`
- `WANT`
- `OBJECT`
- `SNAPSHOT_OFFER`
- `VIEW_ANNOUNCE`
- `BYE`

这些消息的传输格式以 [`WIRE-PROTOCOL.zh-CN.md`](./WIRE-PROTOCOL.zh-CN.md) 为规范性定义。
本核心协议文件只描述概念性的同步流程与被复制对象的语义。

## 9. 冲突与合并

Mycel 不把冲突视为协议失败。

### 9.1 合法状态

以下都合法：

- 多个 heads
- 不同分支长期并存
- 同一段文本有多个地方版本

### 9.2 合并结果可分三类

- **Auto-merged**：自动合并成功
- **Multi-variant**：保留并列版本
- **Manual-curation-required**：需要人工整理

在 v0.1，任何以 Revision 发布的 merge 结果，都 MUST 已经被实体化成明确的 Patch 操作。
接收端是靠重放这些 Patches 验证结果状态，而不是根据 parent ancestry 重新计算 semantic merge（语义合并）。

### 9.3 Merge Generation Profile v0.1（规范）

Mycel v0.1 定义一个保守版 semantic merge generation profile（语义合并生成设置档）。
这个 profile 只用来产生候选 merge Patch 操作。
验证仍然只依赖最终产生的 Patch、Revision 与 `state_hash`。

#### 9.3.1 输入

一个 merge generator 的输入为：

- `base_revision`
- `left_revision`
- `right_revision`

三者都 MUST：

1. 属于同一个 `doc_id`
2. 是已完整验证的 revision
3. 在开始 merge generation 前先还原成 canonical document states

`base_revision` 是比对用的共同祖先状态。
`left_revision` 与 `right_revision` 是两个待整合的后代状态。

#### 9.3.2 逐 Block 分类

对任何出现在三个状态之一中的逻辑 `block_id`，都要将其分类为：

- unchanged
- inserted
- deleted
- replaced
- moved
- annotated
- metadata-changed

分类一律相对于 `base_revision` 进行。

#### 9.3.3 Auto-Merge 规则

只有当所有受影响 block 都能依以下规则解决时，merge generator MAY 产生 `Auto-merged`：

1. 若只有一侧修改某 block，而另一侧保持不变，则采用有修改的一侧。
2. 若两侧对同一 block 做出 byte-identical 的修改，则采用该共同结果。
3. 若两侧在不同位置插入不同的新 block，则两个 insert 都保留，且以决定性顺序排列：
   1. 较小的 parent position index
   2. 当 parent position 相同时，left-side insert 先于 right-side insert
   3. 字典序较小的新增 `block_id`
4. 若一侧对 block 做 annotation，而另一侧修改其内容但未删除该 block，则同时保留内容修改与 annotation。
5. 若两侧修改的是不同 metadata keys，则合并这些 key 更新。

若任一受影响 block 不属于以上规则，generator MUST NOT 输出 `Auto-merged`。

#### 9.3.4 强制非自动情况

遇到以下任一情况，merge generator MUST 输出 `Multi-variant` 或 `Manual-curation-required`：

1. 两侧对同一 block 做不同内容的 replace
2. 一侧删除某 block，而另一侧对其做 replace、move、或 annotate
3. 两侧把同一 block move 到不同目的地
4. 两侧对同一 metadata key 设置不同值
5. 任一侧改变 block structure，而另一侧对同一 subtree 做不兼容修改

#### 9.3.5 Multi-Variant 输出规则

若冲突仅限于同一逻辑 block 的替代性存活内容，generator SHOULD 优先输出 `Multi-variant`。
最终 merge Patch MUST 在合并后状态中明确实体化这些并存 alternatives。

#### 9.3.6 Manual Curation 规则

若冲突影响到 structure、ordering、deletion semantics、或 metadata，且无法安全表达成平行并存 variant，generator MUST 输出 `Manual-curation-required`。

#### 9.3.7 输出形式

产生的结果 MUST 被实体化成一般 Patch 操作。
Generator MUST NOT 依赖隐藏的 merge metadata 来让结果状态成立。

若 generator 输出 `Auto-merged`，其 Patch 操作 MUST 足以让任一接收端从 `parents[0]` 决定性重放出同样结果。

### 9.4 多版本 block 范例

```json
{
  "type": "variant_block",
  "block_id": "blk:001",
  "variants": [
    {
      "from_revision": "rev:8fd2",
      "content": "起初没有终稿，只有传递。"
    },
    {
      "from_revision": "rev:b351",
      "content": "起初没有终稿，只有传递与再写。"
    }
  ]
}
```

## 10. View 与采用

Mycel 不定义全局唯一 accepted head。
同一组文件可同时存在多个固定的 View profiles。
这个设计正是 Mycel 与 blockchain 的大差异。

### 10.0 Reader Client 合规要求（规范）

为了在保留 multi-view 的前提下，尽量降低 client 的自由裁量影响：

1. 合规的阅读客户端 MUST 将每个显示中的文件家族绑定到一个固定的 View profile。
2. 在 v0.1，accepted-head selection 的 profile 识别值为 `policy_hash`。
3. 合规的阅读客户端 MUST 只依已验证的协议对象与该固定 profile 导出 active accepted head。
4. 合规的阅读客户端 MUST NOT 提供会改变 active accepted head 的自由裁量本地 policy controls。
5. 合规的阅读客户端 MAY 为了审计而显示 raw heads、branch graphs、或其他 profile 的结果，但除非另有有效固定 profile 治理该结果，否则 MUST NOT 将其显示为 active accepted head。

### 10.0.1 两种 Maintainer 角色（规范）

对 v0.1 的 governed multi-view selection：

1. `editor-maintainer` MAY 发布可建立新 candidate heads 的 Patch 与 Revision 对象。
2. `view-maintainer` MAY 发布会贡献 governance signals 的 View 对象，用于 accepted-head selection。
3. 同一把 key MAY 同时持有两种角色。
4. 仅持有 editor-maintainer 身分，MUST NOT 自动取得 selector weight。
5. Selector weight 与 accepted-head governance MUST 只由 view-maintainer 行为导出，除非未来某个 profile 明确定义其他 signal source。
6. 一个完整验证过的 Revision，即使尚未得到 accepted-head 支持，仍 MAY 作为合法 head 存在。

### 10.1 决定性 Head 选择（规范）

为了降低 client 端分歧，head 选择必须由协议规范驱动：

1. client MUST 先解析一个固定的 `profile_id`，并以 `profile_id` 与 `doc_id` 发出请求，且 MAY 附带 selection-time boundary（选择时间边界）。
2. client MUST NOT 强制指定 `head_id`。
3. node MUST 依请求的固定 profile，从 eligible heads 即时计算 `selected_head`。
4. 对同一组已验证对象集合、固定 profile 参数、以及有效 selection time（选择时间），选择器 MUST 产生决定性结果。
5. 回应 MUST 包含 `selected_head` 与可机器解析的 decision trace（决策轨迹）。

#### 10.1.1 Selector Inputs

Selector 的输入 tuple（输入组）为：

- `profile_id`
- `doc_id`
- `effective_selection_time`

在 v0.1，`profile_id` 就是 active View profile 的固定 `policy_hash`。
若 client 支援多个固定 profiles，MUST 以明确列举方式提供；它 MUST NOT 为 active accepted-head 路径临时构造 ad hoc 本地 policies。

`effective_selection_time` 定义如下：

- 若 client 有提供 boundary，则使用该值
- 否则使用 node 处理请求时的本地时间

若 client 省略 boundary，node MUST 在 decision trace（决策轨迹）中输出解析后的 `effective_selection_time`。

Selector 只可使用 `policy_hash` 等于 `profile_id` 的完整验证 View 对象。

#### 10.1.2 Eligible Heads

对某个 `doc_id` 而言，Revision 若要成为 eligible head，必须同时符合以下条件：

1. 该 Revision 已依所有 object、hash、signature、state 规则完整验证
2. 该 Revision 的 `doc_id` 与请求的 `doc_id` 相同
3. 该 Revision 的 timestamp 小于或等于 `effective_selection_time`
4. 不存在另一个同文件、同样已完整验证且 timestamp 小于或等于 `effective_selection_time` 的 descendant Revision

若不存在 eligible heads，选择必须失败，并回传像 `NO_ELIGIBLE_HEAD` 这类可机器解析的原因。

#### 10.1.3 Maintainer Signals

对每个已准入的 view-maintainer key `k`，selector 在 selector epoch（选择器 epoch）中最多导出一个 signal（讯号）：

1. 依第 10.2 节规则决定 selector epoch
2. 收集所有完整验证过的 View 对象，且需符合：
   - `maintainer == k`
   - `timestamp` 落在 selector epoch 内
   - `timestamp <= effective_selection_time`
   - `HASH(canonical_serialization(view.policy)) == profile_id`
3. 依以下顺序选出其中最新的一个 View：
   1. 较新的 `timestamp`
   2. 字典序较小的 `view_id`
4. 若该 View 含有 `documents[doc_id]`，且其值正好是某个 eligible head，则 `k` 对该 head 贡献一个 support signal
5. 否则 `k` 对该 `doc_id` 不贡献 signal

对任一 `(profile_id, doc_id, selector_epoch)`，每个 admitted view-maintainer 最多只能对一个 eligible head 贡献 signal。

#### 10.1.4 Selector Score

对每个 eligible head `h`：

```text
weighted_support(h) = sum(effective_weight(k)) for all view-maintainers k signaling to h
supporter_count(h) = count(k) for all view-maintainers k signaling to h
selector_score(h) = weighted_support(h)
```

被选中的 head，是 ordered tuple 最大的 eligible head：

```text
(selector_score, revision_timestamp, inverse_lexicographic_priority)
```

Tie-break 顺序 MUST 固定为：

1. 较高 `selector_score`
2. 较新 `revision_timestamp`
3. 字典序较小的 `revision_id`

Raw supporter count MAY 出现在 trace 中以利审计，但 MUST NOT 高于 `selector_score`。

#### 10.1.5 Decision Trace Schema

Decision trace（决策轨迹）MUST 可机器解析，且至少包含：

```json
{
  "profile_id": "hash:...",
  "doc_id": "doc:origin-text",
  "effective_selection_time": 1777781000,
  "selector_epoch": 587,
  "eligible_heads": [
    {
      "revision_id": "rev:0ab1",
      "revision_timestamp": 1777780000,
      "weighted_support": 7,
      "supporter_count": 3,
      "selector_score": 7
    }
  ],
  "selected_head": "rev:0ab1",
  "tie_break_reason": "higher_selector_score"
}
```

对同一组已验证对象集合、固定 profile 参数、以及 effective selection time（选择时间），此 trace MUST 可重现。

### 10.2 View Profile 参数 + View-Maintainer 准入（规范）

Mycel 对 accepted-head selection 采用假名、身份盲的治理。
View-maintainers 以 key 识别，不要求真实身份，也不要求彼此相识。

准入与加权规则：

1. View-maintainer 候选资格 MUST 只依可验证的协议行为评估，不依声称的真实身份。
2. 提供 accepted-head results 的 node MUST 保存并公布其固定 profile 参数，以便审计。
3. 固定 profile 参数至少 MUST 包含：
   - `epoch_seconds`
   - `epoch_zero_timestamp`
   - `admission_window_epochs`
   - `min_valid_views_for_admission`
   - `min_valid_views_per_epoch`
   - `weight_cap_per_key`
4. `epoch_seconds` MUST 是正整数。
5. Selector epoch 为：

```text
selector_epoch = floor((effective_selection_time - epoch_zero_timestamp) / epoch_seconds)
```

6. 对每个 view-maintainer key `k` 与 epoch `e`，定义：
   - `valid_view_count(e, k)`：在 epoch `e` 中，由 `k` 发布且 policy hash 等于 selector `profile_id` 的完整验证 View 对象数量
   - `critical_violation_count(e, k)`：在 epoch `e` 中可验证归因于 `k` 的重大违规数量
7. 若某个 view-maintainer key 在前 `admission_window_epochs` 个已完成 epoch 中同时满足：
   - `valid_view_count` 总和至少为 `min_valid_views_for_admission`
   - `critical_violation_count` 总和为零
   则该 key 在 epoch `e` 中视为 admitted。
8. 非 admitted key 的 effective weight MUST 为 `0`。
9. Admitted key 第一次取得的 weight 为 `1`。
10. 之后每个 epoch 的 effective weight 更新规则如下：

```text
delta(e, k) =
  -1 if critical_violation_count(e-1, k) > 0
  +1 if critical_violation_count(e-1, k) == 0
       and valid_view_count(e-1, k) >= min_valid_views_per_epoch
   0 otherwise

effective_weight(e, k) =
  clamp(effective_weight(e-1, k) + delta(e, k), 0, weight_cap_per_key)
```

11. `clamp(x, lo, hi)` 的意义是：若 `x < lo` 回传 `lo`，若 `x > hi` 回传 `hi`，否则回传 `x`。
12. 若某 key 在 epoch `e-1` 有一个或多个重大违规，则它在 epoch `e` MUST 至少失去一个 weight unit。
13. 合规的阅读客户端 MUST NOT 套用会改变 active accepted-head 路径的自由裁量 per-installation 隔离或移除规则。
14. head 选择 MUST 使用 `effective_weight(e, k)`，且 MUST NOT 单独依赖原始 hit count。

### 10.3 Editor-Maintainer 发布政策（规范）

Editor-maintainer policy 与 accepted-head governance 分离：

1. 某个 profile MAY 定义 editor-maintainer 身分是否为受限的发布类别。
2. 若某个 profile 定义 editor-maintainer 准入，该准入规则 MUST 可审计且明确。
3. Editor-maintainer 身分 MAY 决定哪些 revisions 会在 reader 或 curator tooling 中被标成正式 candidate heads。
4. Editor-maintainer 身分本身 MUST NOT 影响 `effective_weight`。
5. 同时具备 editor-maintainer 与 view-maintainer 身分的 key，MUST 分别满足两条规则路径。
6. 除非某个 profile 明确收窄集合，否则 v0.1 的 eligible heads 仍是第 10.1.2 节中所有完整验证的 heads，而不只限 editor-maintainer 发布的 heads。

## 11. 匿名与安全预设

### 11.1 传输匿名

Mycel 建议预设跑在匿名传输上，例如：

- Tor onion services
- 或其他匿名 mesh transport

### 11.2 内容安全

每个 object 都需通过：

- hash 验证
- signature 验证
- context 验证

### 11.3 Metadata 最小化

建议节点：

- 批次转发
- 随机延迟
- 不公开真实现者身份
- topic 名称可 capability 化

### 11.4 本地传输与安全策略

每个节点仍可定义：

- 接受哪些作者 key
- 接受哪些 curator key
- 是否接受匿名 key
- 新 key 是否先隔离

这些本地策略 MAY 影响 storage、relay、moderation、或 private inspection。
但对合规的阅读客户端而言，它们 MUST NOT 改变第 10 节所定义的 fixed-profile active accepted head。

## 12. 本地储存模型

本地储存分成：

### 12.1 Object Store

用 `object_id` 储存所有对象。

### 12.2 Index Store

建立索引：

- `doc_id -> revisions`
- `revision -> parents`
- `block_id -> latest states`
- `author -> patches`
- `view_id -> governance signal contents`
- `profile_id -> current accepted-head map`

### 12.3 Policy Store

将本地传输、安全、与 moderation 规则，与 fixed-profile accepted-head 路径分开保存。

## 13. URI / 命名格式

v0.1 可用这种命名：

- `mycel://doc/origin-text`
- `mycel://rev/c7d4`
- `mycel://patch/91ac`
- `mycel://view/9aa0`
- `mycel://snap/44cc`

## 14. CLI 雏形

未来工具可包含：

```bash
mycel init
mycel create-doc origin-text
mycel patch origin-text
mycel commit origin-text
mycel branch create community-mainline
mycel merge rev:8fd2 rev:b351
mycel view create community-curation-v3
mycel sync
mycel serve
mycel verify
```

## 15. 最小实现架构

一个 Mycel 客户端最少要有：

### 15.1 Core

- object serializer
- hash engine
- signature engine
- patch applier
- revision builder

### 15.2 Store

- object store
- index store
- local transport/safety policy store
- accepted-head profile index

### 15.3 Network

- peer 传输
- manifest exchange
- want/object exchange
- snapshot sync

### 15.4 UI

- CLI
- wiki-like 阅读／编辑介面
- diff viewer
- branch/view browser

## 16. 典型流程示例

### 16.1 建立文件

1. 作者 A 建立 origin-text
2. 建立 genesis blocks
3. 建立 genesis revision
4. 签名
5. 发布给 peers

### 16.2 修改文件

作者 B 想改一段：

1. 下载最新 revision
2. 建立 patch
3. 用自己的 key 签名
4. 产生新 revision
5. 发布到网络

### 16.3 分支

作者 C 不同意主线：

1. 以同一 `base_revision` 建 patch
2. 发表不同 revision
3. 网络形成第二个 head

### 16.4 合并

Curator D 想把两边整合：

1. 取得两个 heads
2. 试 semantic block merge
3. 成功则产生 merge revision
4. 用自己 key 发表新的 view

## 17. 协议精神

Mycel 的核心不是唯一真理，而是：

> 文可改，史可验，支可分，网可散。

英文可写成：

> Write locally. Sign changes. Replicate freely. Merge socially.

## 18. 协议特色总结

若用一句话定义 Mycel 与其他系统的差异：

- 不是 Git：因为它天生 P2P、天生多 view、天生匿名可用
- 不是 blockchain：因为它不追求全局唯一共识
- 不是 torrent：因为它不是只传档案包，而是传可验证变更历史
- 不是普通 wiki：因为版本不是附属功能，而是核心结构

## 19. 建议下一版

目前这版已包含：

1. **Wire protocol**：规范性的同步消息 schema
2. **Canonical serialization appendix**：决定性的哈希与签名规则
3. **Conservative merge generation profile**：可安全重放的合并输出规则

下一步最有价值的是：

1. **Implementation checklist（实现检查清单）**：把规格整理成可落地的实现 profile（设置档）
2. **Consistency audit（一致性稽核）**：把所有文件中的例子、术语、范围对齐
3. **Governance simplification review（治理简化检查）**：在视 v0.1 为稳定前，先收敛 selector / governance 的可选复杂度

## Appendix A. Canonical Serialization（规范）

Mycel v0.1 以下情境都使用 canonical JSON bytes：

- 内容寻址 object ID
- object signatures
- `state_hash` 计算
- wire-envelope signatures

### A.1 编码

1. Canonical bytes MUST 是 UTF-8 编码的 JSON text。
2. JSON text MUST NOT 包含 byte order mark。
3. 字串值以外的 insignificant whitespace 一律禁止。

### A.2 数据类型

v0.1 canonical payload 可使用的 JSON 值类型：

- object
- array
- string
- integer number
- `true`
- `false`

以下在 canonical payload 中视为无效：

- `null`
- 浮点数
- 指数记号
- 重复的 object keys

### A.3 Object 规则

1. Object keys MUST 唯一。
2. Object keys MUST 依原始 Unicode code point 的字典序递增序列化。
3. Object members MUST 以 `"key":value` 形式序列化，不得加入额外空白。

Key 排序范例：

```json
{"author":"pk:a","doc_id":"doc:x","type":"patch","version":"mycel/0.1"}
```

### A.4 Array 规则

1. Arrays MUST 保留协议定义的顺序。
2. Arrays MUST 以逗号分隔序列化，不得加入额外空白。
3. Canonicalization 过程 MUST NOT 对 arrays 重新排序。

这表示：

- `parents` 保留宣告顺序
- `patches` 保留宣告顺序
- `blocks` 保留文件结构顺序
- wire `WANT` 的 `objects` 保留发送端请求顺序

### A.5 String 规则

1. Strings MUST 使用 JSON 双引号字串语法序列化。
2. Strings MUST 精确保留原始 code points；实现 MUST NOT 自动做 Unicode normalization。
3. 双引号（`"`）与反斜线（`\`）MUST 转义。
4. U+0000 到 U+001F 的控制字元 MUST 使用小写 `\u00xx` 转义。
5. `/` MUST NOT 被转义，除非更高层 transport 在 canonicalization 之外另有要求。
6. 非 ASCII 字元 MAY 直接以 UTF-8 出现，且除非它是控制字元，MUST NOT 被改写成 `\u` escape。

### A.6 Integer 规则

1. v0.1 canonical payload 中的数字 MUST 是十进位整数。
2. 零 MUST 序列化为 `0`。
3. 正整数 MUST NOT 带有前置 `+`。
4. 整数 MUST NOT 含有前导零。
5. 负整数只有在字段定义明确允许时才可使用。

### A.7 Boolean

Boolean 值 MUST 序列化成小写 `true` 或 `false`。

### A.8 字段省略

1. 不存在的 optional fields MUST 直接省略。
2. 实现 MUST NOT 用 `null` 表示「缺省」。
3. 导出 ID 字段与 `signature` 只有在特定 hashing 或 signing 规则明确要求时，才可省略。

### A.9 Canonicalization Procedure

Canonicalize 一个 payload 的步骤：

1. 验证 payload 只使用允许的 JSON 类型。
2. 拒绝重复 keys。
3. 拒绝禁止的数字格式与 `null`。
4. 依 A.3 规则递归排序所有 object keys。
5. 保留所有 array 顺序。
6. 以 UTF-8 JSON 并且不含 insignificant whitespace 的形式序列化。

### A.10 Canonical State Object

计算 `state_hash` 时，结果 state object MUST 使用以下形状：

```json
{
  "doc_id": "doc:origin-text",
  "blocks": [
    {
      "block_id": "blk:001",
      "block_type": "paragraph",
      "content": "Example text",
      "attrs": {},
      "children": []
    }
  ]
}
```

补充规则：

1. State serialization 中的每个 block object 都 MUST 包含 `block_id`、`block_type`、`content`、`attrs`、`children`。
2. `attrs` MUST 是 object；若为空，MUST 序列化成 `{}`。
3. `children` MUST 是 array；若为空，MUST 序列化成 `[]`。

### A.11 Canonical Envelope Serialization

Wire envelopes 使用同一套 canonical JSON 规则。
计算 envelope signature 时，必须在 canonicalization 之前先省略 `sig` 字段。
