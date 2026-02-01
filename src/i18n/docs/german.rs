use std::collections::HashMap;
use std::sync::OnceLock;

use super::DocSection;

pub(super) fn german_docs() -> &'static HashMap<&'static str, DocSection> {
    static MAP: OnceLock<HashMap<&'static str, DocSection>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            (
                "general",
                DocSection {
                    title: "Allgemeine Informationen",
                    markdown: r#"# Allgemeine Informationen

## Wichtiger Hinweis

Oxide Mongo ist kein vollständiger Ersatz für die Standard-mongo-Shell und enthält keinen JavaScript-Interpreter.
Die Anwendung versucht nicht, die gesamte Shell nachzubilden. Stattdessen emuliert sie die häufigsten Befehle und macht die Arbeit mit MongoDB in einer GUI bequem.

Wenn du eine vollständige JavaScript-Umgebung oder komplexe Skripte brauchst, bleibt die mongo-Shell die Standardwahl.
Oxide Mongo konzentriert sich auf alltägliche Aufgaben, das Durchsuchen von Daten und schnelles Arbeiten mit der Datenbank.

## Über das Projekt

Oxide Mongo ist ein plattformübergreifender, leichtgewichtiger GUI-Client für MongoDB.
Das Projekt ist vom hervorragenden Robomongo (später Robo3T) inspiriert, das heute praktisch nicht mehr gepflegt wird.

Ziel ist es, die Robomongo-Philosophie beizubehalten:
- Minimalismus statt einer überladenen Oberfläche
- schneller Start und geringer Ressourcenverbrauch
- keine aufdringlichen Einschränkungen

Oxide Mongo ist ein offenes und kostenloses Werkzeug für Entwickler und Administratoren, die schnellen und klaren Zugriff auf MongoDB ohne zusätzliche Komplexität benötigen.
"#,
                },
            ),
            (
                "quick-start",
                DocSection {
                    title: "Schnellstart",
                    markdown: r#"# Schnellstart

## Erster Start

Unten findest du ein Schritt-für-Schritt-Szenario für den ersten Start, wenn du bereits eine Datenbank hast und ein Dokument suchen und bearbeiten musst.

- Öffne das Menü "Verbindungen" und klicke auf "Erstellen".
- Fülle die Verbindungsparameter aus: Adresse, Port, Datenbank. Aktiviere bei Bedarf Authentifizierung und/oder SSH-Tunnel.
- Klicke auf "Testen" und stelle sicher, dass die Prüfung erfolgreich ist.
- Klicke auf "Speichern", wähle dann die erstellte Verbindung aus und klicke auf "Verbinden".
- Erweitere im linken Bereich die Datenbank und die Sammlung und öffne dann einen Tab.
- Gib im Abfrage-Editor eine Suche ein, z. B. `db.getCollection('my_collection').find({})`, und klicke auf "Senden".
- Wähle in der Ergebnistabelle ein Dokument aus und öffne das Kontextmenü "Dokument bearbeiten...".
- Ändere das Dokument und klicke auf "Speichern".
- Falls nötig, führe `find(...)` erneut aus, um zu prüfen, ob die Daten aktualisiert wurden.
"#,
                },
            ),
            (
                "supported-commands",
                DocSection {
                    title: "Unterstützte Befehle",
                    markdown: r#"# Liste der unterstützten Befehle:

## Für Sammlungen:

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

Für find(...), werden die folgenden Methoden unterstützt:

    sort(...), hint(...), limit(...), skip(...), maxTimeMS(...), explain(), count(...), countDocuments(...), comment(...)

## Für Datenbanken

    db.stats(...)
    db.runCommand(...)
    db.adminCommand(...)
    db.watch(...)

## Für Replica-Set-Helfer

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
                    title: "Änderungsstream",
                    markdown: r#"# Änderungsstream

## Funktionsweise

Der Befehl `watch(...)` startet einen Änderungsstream. Die Abfrage gibt nicht alle Dokumente auf einmal zurück. Stattdessen wartet sie auf neue Ereignisse und fügt sie bei Eintreffen der Tabelle hinzu.

## Wann er endet

Der Stream stoppt automatisch, wenn die Anzahl der empfangenen Elemente den Wert `limit` erreicht. Danach gilt die Abfrage als abgeschlossen und die Ausführungszeit wird angezeigt.
"#,
                },
            ),
            (
                "hotkeys",
                DocSection {
                    title: "Tastenkürzel",
                    markdown: r#"# Tastenkürzel

- F2 — Ergebnisse in die Tabellenansicht wechseln
- F4 — Ergebnisse in die Textansicht wechseln
- Strg+Enter — aktuelle Abfrage ausführen
- Strg+W — aktiven Tab schließen
"#,
                },
            ),
        ])
    })
}
