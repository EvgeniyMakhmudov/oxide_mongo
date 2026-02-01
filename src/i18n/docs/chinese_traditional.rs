use std::collections::HashMap;
use std::sync::OnceLock;

use super::DocSection;

pub(super) fn chinese_traditional_docs() -> &'static HashMap<&'static str, DocSection> {
    static MAP: OnceLock<HashMap<&'static str, DocSection>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            (
                "general",
                DocSection {
                    title: "概述",
                    markdown: r#"# 概述

## 重要說明

Oxide Mongo 不是標準 mongo shell 的完整替代品，也不包含 JavaScript 直譯器。
應用程式不打算重現整個 shell，而是模擬最常用的命令，讓在 GUI 中使用 MongoDB 更方便。

如果需要完整的 JavaScript 環境或複雜腳本，mongo shell 仍是預設選擇。
Oxide Mongo 專注於日常任務、資料瀏覽與快速操作資料庫。

## 關於專案

Oxide Mongo 是一個輕量、跨平台的 MongoDB GUI 用戶端。
專案靈感來自優秀的 Robomongo（後來的 Robo3T），但該專案如今幾乎不再維護。

目標是延續 Robomongo 的理念：
- 以簡潔取代臃腫介面
- 啟動快速、資源占用低
- 不引入侵入式限制

Oxide Mongo 是一個開源且免費的工具，面向需要快速、清晰存取 MongoDB 的開發者與管理者。
"#,
                },
            ),
            (
                "quick-start",
                DocSection {
                    title: "快速開始",
                    markdown: r#"# 快速開始

## 首次使用

以下是首次使用的步驟示例，適用於已經有資料庫且需要查找與編輯文件的情境。

- 打開「連線」選單並點擊「建立」。
- 填寫連線參數：位址、埠、資料庫。需要時啟用驗證和/或 SSH 通道。
- 點擊「測試」，確保驗證成功。
- 點擊「儲存」，然後選擇建立的連線並點擊「連線」。
- 在左側面板展開資料庫與集合，開啟一個分頁。
- 在查詢編輯器輸入查詢，例如 `db.getCollection('my_collection').find({})`，然後點擊「送出」。
- 在結果表中選擇一個文件，並開啟右鍵選單「編輯文件...」。
- 修改文件後點擊「儲存」。
- 如有需要，再次執行 `find(...)` 以驗證資料已更新。
"#,
                },
            ),
            (
                "supported-commands",
                DocSection {
                    title: "支援的命令",
                    markdown: r#"# 支援的命令清單：

## 集合相關：

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

find(...) 支援以下方法：

    sort(...), hint(...), limit(...), skip(...), maxTimeMS(...), explain(), count(...), countDocuments(...), comment(...)

## 資料庫相關

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
                    title: "變更串流",
                    markdown: r#"# 變更串流

## 運作方式

`watch(...)` 命令會啟動變更串流。查詢不會一次回傳所有文件，而是等待新事件並在到達時追加到表格中。

## 結束條件

當接收到的元素數量達到 `limit` 值時，串流會自動停止。此後查詢視為完成，並顯示執行時間。
"#,
                },
            ),
            (
                "hotkeys",
                DocSection {
                    title: "快捷鍵",
                    markdown: r#"# 快捷鍵

- F2 — 切換結果為表格檢視
- F4 — 切換結果為文字檢視
- Ctrl+Enter — 執行目前查詢
- Ctrl+W — 關閉目前分頁
"#,
                },
            ),
        ])
    })
}
