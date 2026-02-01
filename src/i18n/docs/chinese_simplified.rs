use std::collections::HashMap;
use std::sync::OnceLock;

use super::DocSection;

pub(super) fn chinese_simplified_docs() -> &'static HashMap<&'static str, DocSection> {
    static MAP: OnceLock<HashMap<&'static str, DocSection>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            (
                "general",
                DocSection {
                    title: "概述",
                    markdown: r#"# 概述

## 重要说明

Oxide Mongo 不是标准 mongo shell 的完整替代品，也不包含 JavaScript 解释器。
应用并不试图复刻整个 shell，而是模拟最常用的命令，让在 GUI 中使用 MongoDB 更方便。

如果需要完整的 JavaScript 环境或复杂脚本，mongo shell 仍是默认选择。
Oxide Mongo 专注于日常任务、数据浏览和快速操作数据库。

## 关于项目

Oxide Mongo 是一个轻量级、跨平台的 MongoDB GUI 客户端。
项目灵感来自优秀的 Robomongo（后来的 Robo3T），但该项目如今几乎不再维护。

目标是延续 Robomongo 的理念：
- 以简洁替代臃肿的界面
- 启动快速、资源占用低
- 不引入侵入式限制

Oxide Mongo 是一个开源且免费的工具，面向需要快速、清晰访问 MongoDB 的开发者与管理员。
"#,
                },
            ),
            (
                "quick-start",
                DocSection {
                    title: "快速开始",
                    markdown: r#"# 快速开始

## 首次使用

下面是首次使用的步骤示例，适用于已经有数据库并需要查找和编辑文档的场景。

- 打开“连接”菜单并点击“创建”。
- 填写连接参数：地址、端口、数据库。需要时启用认证和/或 SSH 隧道。
- 点击“测试”，确保验证成功。
- 点击“保存”，然后选择创建的连接并点击“连接”。
- 在左侧面板展开数据库和集合，打开一个标签页。
- 在查询编辑器输入查询，例如 `db.getCollection('my_collection').find({})`，然后点击“发送”。
- 在结果表中选择一个文档，并打开上下文菜单“编辑文档...”。
- 修改文档后点击“保存”。
- 如有需要，再次运行 `find(...)` 验证数据已更新。
"#,
                },
            ),
            (
                "supported-commands",
                DocSection {
                    title: "支持的命令",
                    markdown: r#"# 支持的命令列表：

## 集合相关：

    find(...)
    findOne(...)
    count(...)
    countDocuments(...)
    estimatedDocumentCount(...)
    distinct(...)
    aggregate(...)
    watch(...)
    insertOne(...)
    insertMany(...)
    bulkWrite(...)
    updateOne(...)
    updateMany(...)
    replaceOne(...)
    findOneAndUpdate(...)
    findOneAndReplace(...)
    findOneAndDelete(...)
    findAndModify(...)
    deleteOne(...)
    deleteMany(...)
    createIndex(...)
    createIndexes(...)
    dropIndex(...)
    dropIndexes(...)
    getIndexes()
    hideIndex(...)
    unhideIndex(...)

find(...) 支持以下方法：

    sort(...), hint(...), limit(...), skip(...), maxTimeMS(...), explain(), count(...), countDocuments(...), comment(...)

## 数据库相关

    db.stats(...)
    db.runCommand(...)
    db.adminCommand(...)
    db.watch(...)

## 副本集助手

    rs.status()
    rs.conf()
    rs.isMaster()
    rs.hello()
    rs.printReplicationInfo()
    rs.printSecondaryReplicationInfo()
    rs.initiate(...)
    rs.reconfig(...)
    rs.stepDown(...)
    rs.freeze(...)
    rs.add(...)
    rs.addArb(...)
    rs.remove(...)
    rs.syncFrom(...)
    rs.slaveOk()
"#,
                },
            ),
            (
                "change-stream",
                DocSection {
                    title: "变更流",
                    markdown: r#"# 变更流

## 工作方式

`watch(...)` 命令会启动变更流。查询不会一次性返回所有文档，而是等待新事件并在到达时追加到表格中。

## 结束条件

当接收到的元素数量达到 `limit` 值时，流会自动停止。此后查询视为完成，并显示执行时间。
"#,
                },
            ),
            (
                "hotkeys",
                DocSection {
                    title: "快捷键",
                    markdown: r#"# 快捷键

- F2 — 切换结果为表格视图
- F4 — 切换结果为文本视图
- Ctrl+Enter — 运行当前查询
- Ctrl+W — 关闭当前标签页
"#,
                },
            ),
        ])
    })
}
