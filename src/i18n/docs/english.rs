use std::collections::HashMap;
use std::sync::OnceLock;

use super::DocSection;

pub(super) fn english_docs() -> &'static HashMap<&'static str, DocSection> {
    static MAP: OnceLock<HashMap<&'static str, DocSection>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            (
                "general",
                DocSection {
                    title: "General information",
                    markdown: r#"# General information

## Important note

Oxide Mongo is not a full replacement for the standard mongo shell and does not include a JavaScript interpreter.
The application does not aim to replicate the entire shell. Instead it emulates the most common commands, making MongoDB work convenient in a GUI.

If you need a full JavaScript environment or complex scripts, mongo shell is still the default choice.
Oxide Mongo focuses on day-to-day tasks, data browsing, and fast work with the database.

## About the project

Oxide Mongo is a cross-platform, lightweight GUI client for MongoDB.
The project is inspired by the excellent Robomongo (later Robo3T), which is effectively not maintained today.

The goal is to keep the Robomongo philosophy:
- minimalism instead of an overloaded interface
- fast startup and low resource usage
- no intrusive limitations

Oxide Mongo is an open and free tool for developers and administrators who need fast and clear access to MongoDB without extra complexity.
"#,
                },
            ),
            (
                "quick-start",
                DocSection {
                    title: "Quick start",
                    markdown: r#"# Quick start

## First launch

Below is a step-by-step scenario for the first launch when you already have a database and need to search and edit a document.

- Open the "Connections" menu and click "Create".
- Fill in the connection parameters: address, port, database. Enable authentication and/or SSH tunnel if needed.
- Click "Test" and make sure the check succeeds.
- Click "Save", then select the created connection and click "Connect".
- In the left panel expand the database and collection, then open a tab.
- In the query editor enter a search, for example `db.getCollection('my_collection').find({})`, and click "Send".
- In the results table select a document and open the context menu "Edit Document...".
- Modify the document and click "Save".
- If needed, run `find(...)` again to verify that the data was updated.
"#,
                },
            ),
            (
                "supported-commands",
                DocSection {
                    title: "Supported commands",
                    markdown: r#"# Supported commands list:

## For collections:

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

For find(...), the following methods are supported:

    sort(...), hint(...), limit(...), skip(...), maxTimeMS(...), explain(), count(...), countDocuments(...)

## For databases

    db.stats(...)
    db.runCommand(...)
    db.adminCommand(...)
    db.watch(...)

## For replica set helpers

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
                    title: "Change stream",
                    markdown: r#"# Change stream

## How it works

The `watch(...)` command starts a change stream. The query does not return all documents at once. Instead it waits for new events and appends them to the table as they arrive.

## When it ends

The stream stops automatically when the number of received elements reaches the `limit` value. After that the query is considered complete and the execution time is shown.
"#,
                },
            ),
            (
                "hotkeys",
                DocSection {
                    title: "Hotkeys",
                    markdown: r#"# Hotkeys

- F2 — switch results to Table view
- F4 — switch results to Text view
- Ctrl+Enter — run the current query
- Ctrl+W — close the active tab
"#,
                },
            ),
        ])
    })
}
