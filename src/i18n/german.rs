use std::collections::HashMap;
use std::sync::OnceLock;

pub(crate) fn german_map() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            ("Expand Hierarchically", "Hierarchisch erweitern"),
            ("Collapse Hierarchically", "Hierarchisch reduzieren"),
            ("Expand All Hierarchically", "Alles hierarchisch erweitern"),
            ("Collapse All Hierarchically", "Alles hierarchisch reduzieren"),
            ("Copy JSON", "JSON kopieren"),
            ("Duplicate Tab", "Tab duplizieren"),
            ("Tab Color", "Tab-Farbe"),
            ("Reset Tab Color", "Tab-Farbe zurücksetzen"),
            ("View", "Ansicht"),
            ("Table", "Tabelle"),
            ("Text", "Text"),
            (
                "Text view is available only for document results",
                "Die Textansicht ist nur für Dokumentergebnisse verfügbar",
            ),
            ("Copy Key", "Schlüssel kopieren"),
            ("Copy Value", "Wert kopieren"),
            ("Copy Path", "Pfad kopieren"),
            ("Edit Value Only...", "Nur Wert bearbeiten..."),
            ("Delete Index", "Index löschen"),
            ("Hide Index", "Index ausblenden"),
            ("Unhide Index", "Index einblenden"),
            ("comment expects a value.", "comment erwartet einen Wert."),
            ("explain must be followed by find(...).", "explain muss von find(...) gefolgt werden."),
            ("finish does not take any arguments.", "finish akzeptiert keine Argumente."),
            (
                "No methods are supported after finish().",
                "Nach finish() werden keine Methoden unterstützt.",
            ),
            ("Edit Index...", "Index bearbeiten..."),
            ("Edit Document...", "Dokument bearbeiten..."),
            ("Cancel", "Abbrechen"),
            ("Save", "Speichern"),
            ("Field value will be modified", "Der Feldwert wird geändert"),
            ("Value Type", "Werttyp"),
            ("Saving value...", "Wert wird gespeichert..."),
            ("Name", "Name"),
            ("Address/Host/IP", "Adresse/Host/IP"),
            ("Port", "Port"),
            ("Add filter for system databases", "Filter für Systemdatenbanken hinzufügen"),
            ("Include", "Einschließen"),
            ("Exclude", "Ausschließen"),
            ("Testing...", "Wird getestet..."),
            ("Test", "Testen"),
            ("No connections", "Keine Verbindungen"),
            ("Create Database", "Datenbank erstellen"),
            ("Refresh", "Aktualisieren"),
            ("Server Status", "Serverstatus"),
            ("Close", "Schließen"),
            ("No databases", "Keine Datenbanken"),
            ("Statistics", "Statistiken"),
            ("Drop Database", "Datenbank löschen"),
            ("Loading collections...", "Sammlungen werden geladen..."),
            ("No collections", "Keine Sammlungen"),
            ("Open Empty Tab", "Leeren Tab öffnen"),
            ("View Documents", "Dokumente anzeigen"),
            ("Help", "Hilfe"),
            ("Documentation", "Dokumentation"),
            ("General information", "Allgemeine Informationen"),
            ("Quick start", "Schnellstart"),
            ("Supported commands", "Unterstützte Befehle"),
            ("Change stream", "Änderungsstream"),
            ("Search", "Suchen"),
            ("Matches", "Treffer"),
            ("Change Stream", "Änderungsstream"),
            ("Delete Documents...", "Dokumente löschen..."),
            ("Delete All Documents...", "Alle Dokumente löschen..."),
            ("Rename Collection...", "Sammlung umbenennen..."),
            ("Drop Collection...", "Sammlung löschen..."),
            ("Create Collection", "Sammlung erstellen"),
            ("Create Index", "Index erstellen"),
            ("Indexes", "Indizes"),
            ("About", "Über"),
            ("Licenses", "Lizenzen"),
            ("Primary licenses", "Hauptlizenzen"),
            ("Color schemes", "Farbschemata"),
            ("Fonts", "Schriftarten"),
            ("License", "Lizenz"),
            ("Link", "Link"),
            ("Unknown", "Unbekannt"),
            ("Version", "Version"),
            ("Homepage", "Homepage"),
            ("Project started", "Projektstart"),
            ("Author", "Autor"),
            (
                "MongoDB GUI client for browsing collections, running queries, and managing data.",
                "MongoDB-GUI-Client zum Durchsuchen von Sammlungen, Ausführen von Abfragen und Verwalten von Daten.",
            ),
            ("No tabs opened", "Keine Tabs geöffnet"),
            ("No active tab", "Kein aktiver Tab"),
            ("Connections", "Verbindungen"),
            ("Create", "Erstellen"),
            ("Edit", "Bearbeiten"),
            ("Delete", "Löschen"),
            ("Connect", "Verbinden"),
            ("Cancel", "Abbrechen"),
            ("Deleted", "Gelöscht"),
            ("Save error: ", "Speicherfehler: "),
            ("Delete \"{}\"?", "\"{}\" löschen?"),
            ("Yes", "Ja"),
            ("No", "Nein"),
            ("connection", "Verbindung"),
            ("Unknown client", "Unbekannter Client"),
            ("Connecting...", "Verbindung wird hergestellt..."),
            ("Ready", "Bereit"),
            ("Send", "Senden"),
            ("Executing...", "Wird ausgeführt..."),
            ("Canceling...", "Wird abgebrochen..."),
            ("Canceled", "Abgebrochen"),
            ("Completed", "Abgeschlossen"),
            ("No results", "Keine Ergebnisse"),
            ("{} ms", "{} ms"),
            ("{} documents", "{} Dokumente"),
            ("Error:", "Fehler:"),
            (
                "Query not executed yet. Compose a query and press Send.",
                "Die Abfrage wurde noch nicht ausgeführt. Abfrage erstellen und Senden drücken.",
            ),
            (
                "Connection inactive. Reconnect and run the query again.",
                "Verbindung inaktiv. Erneut verbinden und Abfrage erneut ausführen.",
            ),
            (
                "Query not yet executed. Compose a query and press Send.",
                "Die Abfrage wurde noch nicht ausgeführt. Abfrage erstellen und Senden drücken.",
            ),
            ("Loading serverStatus...", "serverStatus wird geladen..."),
            ("New connection", "Neue Verbindung"),
            ("Edit connection", "Verbindung bearbeiten"),
            ("Connection established", "Verbindung hergestellt"),
            ("Selected connection not found", "Ausgewählte Verbindung nicht gefunden"),
            ("Saved", "Gespeichert"),
            ("General", "Allgemein"),
            ("Database filter", "Datenbankfilter"),
            ("Authorization", "Autorisierung"),
            ("SSH tunnel", "SSH-Tunnel"),
            ("Use", "Verwenden"),
            ("Login", "Benutzername"),
            ("Password", "Passwort"),
            ("Private key", "Privater Schlüssel"),
            ("Text key", "Textschlüssel"),
            ("Passphrase", "Passphrase"),
            ("Prompt for password", "Passwort anfordern"),
            ("Store in file", "In Datei speichern"),
            ("Authentication mechanism", "Authentifizierungsmechanismus"),
            ("Authentication method", "Authentifizierungsmethode"),
            ("Database", "Datenbank"),
            ("Connection type", "Verbindungstyp"),
            ("Direct connection", "Direkte Verbindung"),
            ("ReplicaSet", "ReplicaSet"),
            ("Login cannot be empty", "Anmeldung darf nicht leer sein"),
            ("Database cannot be empty", "Datenbank darf nicht leer sein"),
            ("Password cannot be empty", "Passwort darf nicht leer sein"),
            ("Server address", "Serveradresse"),
            ("Server port", "Serverport"),
            ("Username", "Benutzername"),
            (
                "SSH port must be a number between 0 and 65535",
                "SSH-Port muss eine Zahl zwischen 0 und 65535 sein",
            ),
            (
                "SSH server address cannot be empty",
                "SSH-Serveradresse darf nicht leer sein",
            ),
            ("SSH username cannot be empty", "SSH-Benutzername darf nicht leer sein"),
            ("SSH password cannot be empty", "SSH-Passwort darf nicht leer sein"),
            (
                "SSH private key cannot be empty",
                "SSH-Privatschlüssel darf nicht leer sein",
            ),
            (
                "SSH private key file not found",
                "SSH-Privatschlüsseldatei nicht gefunden",
            ),
            (
                "SSH known_hosts file not found",
                "SSH known_hosts-Datei nicht gefunden",
            ),
            (
                "Failed to read SSH known_hosts",
                "SSH known_hosts konnte nicht gelesen werden",
            ),
            ("Failed to read SSH host key", "SSH-Hostschlüssel konnte nicht gelesen werden"),
            ("SSH host key mismatch", "SSH-Hostschlüssel stimmt nicht überein"),
            (
                "SSH host is not present in known_hosts",
                "SSH-Host ist nicht in known_hosts vorhanden",
            ),
            (
                "SSH known_hosts check failed",
                "SSH-known_hosts-Prüfung fehlgeschlagen",
            ),
            (
                "SSH private key text is not supported on this platform",
                "SSH-Privatschlüssel im Textformat wird auf dieser Plattform nicht unterstützt",
            ),
            (
                "SSH password authentication failed. Check username and password.",
                "SSH-Passwortauthentifizierung fehlgeschlagen. Benutzername und Passwort prüfen.",
            ),
            (
                "SSH private key passphrase is required.",
                "Eine Passphrase für den SSH-Privatschlüssel ist erforderlich.",
            ),
            (
                "SSH private key passphrase is incorrect.",
                "Die Passphrase für den SSH-Privatschlüssel ist falsch.",
            ),
            ("Database name", "Datenbankname"),
            ("First collection name", "Name der ersten Sammlung"),
            ("Collection Name", "Name der Sammlung"),
            ("Settings", "Einstellungen"),
            ("Settings Error", "Einstellungsfehler"),
            ("Failed to load settings:", "Einstellungen konnten nicht geladen werden:"),
            ("Failed to apply settings:", "Einstellungen konnten nicht angewendet werden:"),
            (
                "Unable to load the settings file. Use defaults or exit.",
                "Einstellungsdatei konnte nicht geladen werden. Standardwerte verwenden oder beenden.",
            ),
            ("Use Defaults", "Standardwerte verwenden"),
            ("Exit", "Beenden"),
            ("Behavior", "Verhalten"),
            ("Appearance", "Erscheinungsbild"),
            ("Color Theme", "Farbschema"),
            ("Expand first result item", "Erstes Ergebnis-Element erweitern"),
            ("Query timeout (seconds)", "Abfrage-Timeout (Sekunden)"),
            ("Seconds", "Sekunden"),
            ("Sort fields alphabetically", "Felder alphabetisch sortieren"),
            (
                "Sort index names alphabetically",
                "Indexnamen alphabetisch sortieren",
            ),
            (
                "Close related tabs when closing a database",
                "Zugehörige Tabs beim Schließen einer Datenbank schließen",
            ),
            ("Enable logging", "Protokollierung aktivieren"),
            ("Log level", "Protokollierungsgrad"),
            ("Log file path", "Protokolldateipfad"),
            ("Path", "Pfad"),
            ("Language", "Sprache"),
            ("English", "Englisch"),
            ("Russian", "Russisch"),
            ("Primary Font", "Primäre Schriftart"),
            ("Query Result Font", "Schriftart der Abfrageergebnisse"),
            ("Query Editor Font", "Schriftart des Abfrage-Editors"),
            ("Font Size", "Schriftgröße"),
            ("Theme", "Farbschema"),
            ("Widget Surfaces", "Widget-Oberflächen"),
            ("Widget Background", "Widget-Hintergrund"),
            ("Widget Border", "Widget-Rahmen"),
            ("Subtle Buttons", "Dezente Schaltflächen"),
            ("Primary Buttons", "Primäre Schaltflächen"),
            ("Table Rows", "Tabellenzeilen"),
            ("Even Row", "Gerade Zeile"),
            ("Odd Row", "Ungerade Zeile"),
            ("Header Background", "Kopfzeilenhintergrund"),
            ("Separator", "Trennlinie"),
            ("Menu Items", "Menüeinträge"),
            ("Menu Background", "Menühintergrund"),
            ("Menu Hover Background", "Menühintergrund bei Hover"),
            ("Menu Text", "Menütext"),
            ("Default Colors", "Standardfarben"),
            ("Active", "Aktiv"),
            ("Hover", "Hover"),
            ("Pressed", "Gedrückt"),
            ("Text", "Text"),
            ("Border", "Rahmen"),
            ("Apply", "Anwenden"),
            ("System Default", "Systemstandard"),
            ("Monospace", "Monospace"),
            ("Serif", "Serif"),
            ("System", "System"),
            ("Light", "Hell"),
            ("Dark", "Dunkel"),
            ("localhost", "localhost"),
            ("27017", "27017"),
            ("serverStatus", "serverStatus"),
            ("admin", "admin"),
            ("stats", "stats"),
            ("collStats", "collStats"),
            ("indexes", "indexes"),
            ("db.runCommand({ serverStatus: 1 })", "db.runCommand({ serverStatus: 1 })"),
            ("Delete All Documents", "Alle Dokumente löschen"),
            (
                "All documents from collection \"{}\" in database \"{}\" will be deleted. This action cannot be undone.",
                "Alle Dokumente aus der Sammlung \"{}\" in der Datenbank \"{}\" werden gelöscht. Diese Aktion kann nicht rückgängig gemacht werden.",
            ),
            (
                "Confirm deletion of all documents by entering the collection name \"{}\".",
                "Löschen aller Dokumente bestätigen, Sammlungsname \"{}\" eingeben.",
            ),
            ("Confirm Deletion", "Löschung bestätigen"),
            ("Delete Collection", "Sammlung löschen"),
            (
                "Collection \"{}\" in database \"{}\" will be deleted along with all documents. This action cannot be undone.",
                "Die Sammlung \"{}\" in der Datenbank \"{}\" wird zusammen mit allen Dokumenten gelöscht. Diese Aktion kann nicht rückgängig gemacht werden.",
            ),
            (
                "Confirm deletion of the collection by entering its name \"{}\".",
                "Löschen der Sammlung bestätigen, Namen \"{}\" eingeben.",
            ),
            ("Rename Collection", "Sammlung umbenennen"),
            (
                "Enter a new name for collection \"{}\" in database \"{}\".",
                "Neuen Namen für die Sammlung \"{}\" in der Datenbank \"{}\" eingeben.",
            ),
            (
                "Enter a name for the new collection in database \"{}\".",
                "Namen für die neue Sammlung in der Datenbank \"{}\" eingeben.",
            ),
            ("New Collection Name", "Neuer Sammlungsname"),
            ("Rename", "Umbenennen"),
            ("Delete Index", "Index löschen"),
            (
                "Index \"{}\" of collection \"{}\" in database \"{}\" will be deleted. This action cannot be undone.",
                "Der Index \"{}\" der Sammlung \"{}\" in der Datenbank \"{}\" wird gelöscht. Diese Aktion kann nicht rückgängig gemacht werden.",
            ),
            (
                "Confirm index deletion by entering its name \"{}\".",
                "Löschen des Index bestätigen, Namen \"{}\" eingeben.",
            ),
            (
                "updateOne expects a filter, an update, and an optional options object.",
                "updateOne erwartet einen Filter, ein Update und ein optionales Options-Objekt.",
            ),
            (
                "updateMany expects a filter, an update, and an optional options object.",
                "updateMany erwartet einen Filter, ein Update und ein optionales Options-Objekt.",
            ),
            (
                "replaceOne expects a filter, a replacement document, and an optional options object.",
                "replaceOne erwartet einen Filter, ein Ersatzdokument und ein optionales Options-Objekt.",
            ),
            (
                "findOneAndUpdate expects a filter, an update, and an optional options object.",
                "findOneAndUpdate erwartet einen Filter, ein Update und ein optionales Options-Objekt.",
            ),
            (
                "findOneAndReplace expects a filter, a replacement document, and an optional options object.",
                "findOneAndReplace erwartet einen Filter, ein Ersatzdokument und ein optionales Options-Objekt.",
            ),
            (
                "findOneAndDelete expects a filter and an optional options object.",
                "findOneAndDelete erwartet einen Filter und ein optionales Options-Objekt.",
            ),
            (
                "deleteOne requires a filter as the first argument.",
                "deleteOne erfordert einen Filter als erstes Argument.",
            ),
            (
                "deleteOne accepts a filter and an optional options object.",
                "deleteOne akzeptiert einen Filter und ein optionales Options-Objekt.",
            ),
            (
                "deleteMany requires a filter as the first argument.",
                "deleteMany erfordert einen Filter als erstes Argument.",
            ),
            (
                "deleteMany accepts a filter and an optional options object.",
                "deleteMany akzeptiert einen Filter und ein optionales Options-Objekt.",
            ),
            (
                "Method {} is not supported. Available methods: find, watch, findOne, count, countDocuments, estimatedDocumentCount, distinct, aggregate, insertOne, insertMany, updateOne, updateMany, replaceOne, findOneAndUpdate, findOneAndReplace, findOneAndDelete, deleteOne, deleteMany, createIndex, createIndexes, dropIndex, dropIndexes, getIndexes, hideIndex, unhideIndex.",
                "Methode {} wird nicht unterstützt. Verfügbare Methoden: find, watch, findOne, count, countDocuments, estimatedDocumentCount, distinct, aggregate, insertOne, insertMany, updateOne, updateMany, replaceOne, findOneAndUpdate, findOneAndReplace, findOneAndDelete, deleteOne, deleteMany, createIndex, createIndexes, dropIndex, dropIndexes, getIndexes, hideIndex, unhideIndex.",
            ),
            (
                "watch accepts at most one argument (the pipeline array).",
                "watch akzeptiert höchstens ein Argument (das Pipeline-Array).",
            ),
            (
                "watch pipeline element at index {} must be an object.",
                "Das watch-Pipeline-Element am Index {} muss ein Objekt sein.",
            ),
            (
                "watch pipeline must be an array of stages or a single stage object.",
                "Die watch-Pipeline muss ein Array von Stages oder ein einzelnes Stage-Objekt sein.",
            ),
            (
                "watch supports at most two arguments: pipeline and options.",
                "watch unterstützt höchstens zwei Argumente: pipeline und options.",
            ),
            (
                "watch options must be a JSON object.",
                "watch-Optionen müssen ein JSON-Objekt sein.",
            ),
            (
                "Invalid character in the collection name:",
                "Ungültiges Zeichen im Sammlungsnamen:",
            ),
            (
                "Query must start with db.<collection>, db.getCollection('<collection>'), rs.<method>, or a supported database method.",
                "Die Abfrage muss mit db.<collection>, db.getCollection('<collection>'), rs.<method> oder einer unterstützten Datenbankmethode beginnen.",
            ),
            (
                "Query must start with db.<collection>, db.getCollection('<collection>'), rs.<method>, or a supported method.",
                "Die Abfrage muss mit db.<collection>, db.getCollection('<collection>'), rs.<method> oder einer unterstützten Methode beginnen.",
            ),
            (
                "Only one method call is supported after specifying the replica set helper.",
                "Nach Angabe des Replica-Set-Helpers wird nur ein Methodenaufruf unterstützt.",
            ),
            (
                "Method rs.{} is not supported. Available methods: status, conf, isMaster, hello, printReplicationInfo, printSecondaryReplicationInfo, initiate, reconfig, stepDown, freeze, add, addArb, remove, syncFrom, slaveOk.",
                "Methode rs.{} wird nicht unterstützt. Verfügbare Methoden: status, conf, isMaster, hello, printReplicationInfo, printSecondaryReplicationInfo, initiate, reconfig, stepDown, freeze, add, addArb, remove, syncFrom, slaveOk.",
            ),
            (
                "Method rs.{} does not accept arguments.",
                "Methode rs.{} akzeptiert keine Argumente.",
            ),
            (
                "rs.initiate expects no arguments or a config document.",
                "rs.initiate erwartet keine Argumente oder ein Konfigurationsdokument.",
            ),
            (
                "rs.reconfig expects a config document and an optional options document.",
                "rs.reconfig erwartet ein Konfigurationsdokument und ein optionales Options-Dokument.",
            ),
            (
                "rs.stepDown expects an optional number of seconds and an optional secondary catch-up period.",
                "rs.stepDown erwartet eine optionale Anzahl von Sekunden und eine optionale Aufholphase der Sekundären.",
            ),
            (
                "rs.freeze expects a number of seconds.",
                "rs.freeze erwartet eine Anzahl von Sekunden.",
            ),
            (
                "rs.add expects a host string or a member document.",
                "rs.add erwartet eine Host-Zeichenkette oder ein Mitgliedsdokument.",
            ),
            (
                "rs.addArb expects a host string or a member document.",
                "rs.addArb erwartet eine Host-Zeichenkette oder ein Mitgliedsdokument.",
            ),
            (
                "rs.remove expects a host string.",
                "rs.remove erwartet eine Host-Zeichenkette.",
            ),
            (
                "rs.syncFrom expects a host string.",
                "rs.syncFrom erwartet eine Host-Zeichenkette.",
            ),
            (
                "Replica set config response does not contain a config document.",
                "Die Replica-Set-Konfigurationsantwort enthält kein Konfigurationsdokument.",
            ),
            (
                "Replica set config must contain a members array of documents.",
                "Die Replica-Set-Konfiguration muss ein members-Array von Dokumenten enthalten.",
            ),
            (
                "Replica set member must include a host string.",
                "Das Replica-Set-Mitglied muss eine Host-Zeichenkette enthalten.",
            ),
            (
                "Replica set member with host '{}' already exists.",
                "Ein Replica-Set-Mitglied mit Host '{}' existiert bereits.",
            ),
            (
                "Replica set member with host '{}' not found.",
                "Replica-Set-Mitglied mit Host '{}' nicht gefunden.",
            ),
            (
                "Replica set config version must be a number.",
                "Die Version der Replica-Set-Konfiguration muss eine Zahl sein.",
            ),
            (
                "Oplog stats are unavailable.",
                "Oplog-Statistiken sind nicht verfügbar.",
            ),
            (
                "Oplog is empty; cannot compute replication info.",
                "Oplog ist leer; Replikationsinformationen können nicht berechnet werden.",
            ),
            (
                "Oplog entry does not contain a timestamp.",
                "Der Oplog-Eintrag enthält keinen Timestamp.",
            ),
            (
                "Replica set status does not contain members.",
                "Der Replica-Set-Status enthält keine members.",
            ),
            (
                "Primary member optime is not available.",
                "Optime des primären Mitglieds ist nicht verfügbar.",
            ),
            ("slaveOk has no effect in this client.", "slaveOk hat in diesem Client keine Wirkung."),
            ("unknown", "unbekannt"),
            (
                "Single-quoted string contains an unfinished escape sequence.",
                "Die Zeichenkette in einfachen Anführungszeichen enthält eine unvollständige Escape-Sequenz.",
            ),
            (
                "The \\x sequence must contain two hex digits.",
                "Die Sequenz \\x muss zwei Hex-Ziffern enthalten.",
            ),
            (
                "The \\u sequence must contain four hex digits.",
                "Die Sequenz \\u muss vier Hex-Ziffern enthalten.",
            ),
            (
                "Enter the exact database name to confirm.",
                "Genauen Datenbanknamen zur Bestätigung eingeben.",
            ),
            (
                "Enter the name of the first collection for the new database.",
                "Namen der ersten Sammlung für die neue Datenbank eingeben.",
            ),
            (
                "A database with this name already exists.",
                "Eine Datenbank mit diesem Namen existiert bereits.",
            ),
            (
                "Document not found. It may have been deleted or the change was not applied.",
                "Dokument nicht gefunden. Es wurde möglicherweise gelöscht oder die Änderung wurde nicht angewendet.",
            ),
            (
                "Index document must contain a string field named name.",
                "Das Indexdokument muss ein String-Feld namens name enthalten.",
            ),
            (
                "Index name cannot be changed via collMod.",
                "Der Indexname kann nicht über collMod geändert werden.",
            ),
            (
                "Failed to refresh database list:",
                "Datenbankliste konnte nicht aktualisiert werden:",
            ),
            ("Failed to delete index", "Index konnte nicht gelöscht werden"),
            (
                "Database \"{}\" will be deleted along with all collections and documents. This action cannot be undone.",
                "Die Datenbank \"{}\" wird zusammen mit allen Sammlungen und Dokumenten gelöscht. Diese Aktion kann nicht rückgängig gemacht werden.",
            ),
            (
                "Confirm deletion of all data by entering the database name \"{}\".",
                "Löschen aller Daten bestätigen, Datenbankname \"{}\" eingeben.",
            ),
            ("Delete Database", "Datenbank löschen"),
            ("Processing...", "Wird verarbeitet..."),
            (
                "MongoDB creates a database only when the first collection is created. Provide the database name and the first collection to create immediately.",
                "MongoDB erstellt eine Datenbank erst, wenn die erste Sammlung erstellt wird. Datenbanknamen und erste Sammlung angeben, die sofort erstellt werden soll.",
            ),
            ("Creating database...", "Datenbank wird erstellt..."),
            ("Edit Document", "Dokument bearbeiten"),
            (
                "Edit the JSON representation of the document. The document will be fully replaced on save.",
                "JSON-Darstellung des Dokuments bearbeiten. Das Dokument wird beim Speichern vollständig ersetzt.",
            ),
            ("Saving document...", "Dokument wird gespeichert..."),
            ("Edit TTL Index", "TTL-Index bearbeiten"),
            (
                "Only the \"expireAfterSeconds\" field value can be changed. Other parameters will be ignored.",
                "Nur der Feldwert \"expireAfterSeconds\" kann geändert werden. Andere Parameter werden ignoriert.",
            ),
            ("Saving index...", "Index wird gespeichert..."),
            ("Expected a document, got", "Dokument erwartet, erhalten"),
            ("Expected an array, got", "Array erwartet, erhalten"),
            ("Expected binary data, got", "Binärdaten erwartet, erhalten"),
            ("Expected JavaScript code, got", "JavaScript-Code erwartet, erhalten"),
            (
                "Expected a regular expression, got",
                "Regulärer Ausdruck erwartet, erhalten",
            ),
            (
                "Expected JavaScript code with scope, got",
                "JavaScript-Code mit scope erwartet, erhalten",
            ),
            ("Expected a Timestamp, got", "Timestamp erwartet, erhalten"),
            ("Expected a DBRef, got", "DBRef erwartet, erhalten"),
            ("Expected a MinKey, got", "MinKey erwartet, erhalten"),
            ("Expected a MaxKey, got", "MaxKey erwartet, erhalten"),
            ("Expected undefined, got", "undefined erwartet, erhalten"),
            ("Duration:", "Dauer:"),
            ("Element at index", "Element an Index"),
            (
                "in insertMany must be a JSON object.",
                "in insertMany muss ein JSON-Objekt sein.",
            ),
            (
                "Value must be an integer in the Int32 range.",
                "Der Wert muss eine ganze Zahl im Int32-Bereich sein.",
            ),
            (
                "Value must be an integer in the Int64 range.",
                "Der Wert muss eine ganze Zahl im Int64-Bereich sein.",
            ),
            ("Value must be a Double.", "Der Wert muss ein Double sein."),
            (
                "Value must be a valid Decimal128.",
                "Der Wert muss ein gültiges Decimal128 sein.",
            ),
            (
                "BinData expects a base64 string as the second argument.",
                "BinData erwartet eine Base64-Zeichenkette als zweites Argument.",
            ),
            (
                "BinData expects two arguments: a subtype and a base64 string.",
                "BinData erwartet zwei Argumente: einen Untertyp und eine Base64-Zeichenkette.",
            ),
            (
                "DBRef expects an ObjectId as the second argument.",
                "DBRef erwartet ein ObjectId als zweites Argument.",
            ),
            (
                "DBRef expects two or three arguments: collection, _id, and an optional database name.",
                "DBRef erwartet zwei oder drei Argumente: collection, _id und optional einen Datenbanknamen.",
            ),
            (
                "Hex string must contain an even number of characters.",
                "Die Hex-Zeichenkette muss eine gerade Anzahl von Zeichen enthalten.",
            ),
            (
                "HexData expects two arguments: a subtype and a hex string.",
                "HexData erwartet zwei Argumente: einen Untertyp und eine Hex-Zeichenkette.",
            ),
            (
                "HexData expects a string as the second argument.",
                "HexData erwartet eine Zeichenkette als zweites Argument.",
            ),
            (
                "NumberDecimal expects a valid decimal value.",
                "NumberDecimal erwartet einen gültigen Dezimalwert.",
            ),
            ("NumberInt expects an integer.", "NumberInt erwartet eine ganze Zahl."),
            ("NumberLong expects an integer.", "NumberLong erwartet eine ganze Zahl."),
            (
                "Object expects a JSON object, but received a value of type {}.",
                "Object erwartet ein JSON-Objekt, erhielt jedoch einen Wert vom Typ {}.",
            ),
            (
                "ObjectId accepts either zero or one string argument.",
                "ObjectId akzeptiert entweder null oder ein String-Argument.",
            ),
            (
                "ObjectId requires a 24-character hex string or no arguments.",
                "ObjectId erfordert eine 24-stellige Hex-Zeichenkette oder keine Argumente.",
            ),
            ("ObjectId.fromDate expects a single argument.", "ObjectId.fromDate erwartet ein einzelnes Argument."),
            ("RegExp expects a string pattern.", "RegExp erwartet ein Zeichenkettenmuster."),
            (
                "RegExp expects a pattern and optional options.",
                "RegExp erwartet ein Muster und optionale Optionen.",
            ),
            (
                "Timestamp expects two arguments: time and increment.",
                "Timestamp erwartet zwei Argumente: time und increment.",
            ),
            (
                "UUID expects a string in the format xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.",
                "UUID erwartet eine Zeichenkette im Format xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.",
            ),
            (
                "arrayFilters must be an array of objects.",
                "arrayFilters muss ein Array von Objekten sein.",
            ),
            (
                "arrayFilters must contain at least one filter object.",
                "arrayFilters muss mindestens ein Filterobjekt enthalten.",
            ),
            ("collation must be a JSON object.", "collation muss ein JSON-Objekt sein."),
            (
                "db.runCommand expects a document describing the command.",
                "db.runCommand erwartet ein Dokument, das den Befehl beschreibt.",
            ),
            (
                "db.runCommand supports only one argument (the command document).",
                "db.runCommand unterstützt nur ein Argument (das Befehlsdokument).",
            ),
            (
                "db.adminCommand expects a document describing the command.",
                "db.adminCommand erwartet ein Dokument, das den Befehl beschreibt.",
            ),
            (
                "db.adminCommand supports only one argument (the command document).",
                "db.adminCommand unterstützt nur ein Argument (das Befehlsdokument).",
            ),
            (
                "aggregate supports at most two arguments: pipeline and options.",
                "aggregate unterstützt höchstens zwei Argumente: pipeline und options.",
            ),
            (
                "aggregate options must be a JSON object.",
                "aggregate-Optionen müssen ein JSON-Objekt sein.",
            ),
            (
                "aggregate cursor options must be a JSON object.",
                "aggregate-Cursor-Optionen müssen ein JSON-Objekt sein.",
            ),
            (
                "distinct supports at most three arguments: field, filter, and options.",
                "distinct unterstützt höchstens drei Argumente: field, filter und options.",
            ),
            (
                "distinct options must be a JSON object.",
                "distinct-Optionen müssen ein JSON-Objekt sein.",
            ),
            (
                "findOneAndModify expects a JSON object.",
                "findOneAndModify erwartet ein JSON-Objekt.",
            ),
            (
                "findOneAndModify requires a JSON object with parameters.",
                "findOneAndModify erfordert ein JSON-Objekt mit Parametern.",
            ),
            (
                "findOneAndModify requires an 'update' parameter when remove=false.",
                "findOneAndModify erfordert den Parameter 'update', wenn remove=false.",
            ),
            (
                "hint must be a string or a JSON object with index specification.",
                "hint muss eine Zeichenkette oder ein JSON-Objekt mit Indexspezifikation sein.",
            ),
            (
                "returnDocument must be the string 'before' or 'after'.",
                "returnDocument muss die Zeichenkette 'before' oder 'after' sein.",
            ),
            (
                "writeConcern must be a JSON object.",
                "writeConcern muss ein JSON-Objekt sein.",
            ),
            (
                "writeConcern.j must be a boolean value.",
                "writeConcern.j muss ein boolescher Wert sein.",
            ),
            (
                "writeConcern.w must be a non-negative integer.",
                "writeConcern.w muss eine nicht negative ganze Zahl sein.",
            ),
            (
                "writeConcern.w must be a string or a number.",
                "writeConcern.w muss eine Zeichenkette oder eine Zahl sein.",
            ),
            (
                "writeConcern.w must not exceed the maximum allowed value.",
                "writeConcern.w darf den maximal zulässigen Wert nicht überschreiten.",
            ),
            (
                "writeConcern.wtimeout must be a non-negative integer.",
                "writeConcern.wtimeout muss eine nicht negative ganze Zahl sein.",
            ),
            (
                "db.stats expects a number or an options object.",
                "db.stats erwartet eine Zahl oder ein Options-Objekt.",
            ),
            ("{}::{} must be a positive integer.", "{}::{} muss eine positive ganze Zahl sein."),
            (
                "{}::{} must be a number, received {}.",
                "{}::{} muss eine Zahl sein, erhalten {}.",
            ),
            ("{}::{} must fit into u32.", "{}::{} muss in u32 passen."),
            ("Argument must be a JSON object.", "Argument muss ein JSON-Objekt sein."),
            (
                "Argument must be a string or a number.",
                "Argument muss eine Zeichenkette oder eine Zahl sein.",
            ),
            (
                "Index argument must be a string with the index name or an object with keys.",
                "Index-Argument muss eine Zeichenkette mit dem Indexnamen oder ein Objekt mit Schlüsseln sein.",
            ),
            (
                "Update argument must be an object with operators or an array of stages.",
                "Update-Argument muss ein Objekt mit Operatoren oder ein Array von Stages sein.",
            ),
            (
                "The second argument to Code must be an object.",
                "Das zweite Argument von Code muss ein Objekt sein.",
            ),
            (
                "Enter the exact index name to confirm.",
                "Genauen Indexnamen zur Bestätigung eingeben.",
            ),
            (
                "Enter the exact collection name to confirm.",
                "Genauen Sammlungsnamen zur Bestätigung eingeben.",
            ),
            ("Document must be a JSON object.", "Dokument muss ein JSON-Objekt sein."),
            (
                "NumberInt value is out of the Int32 range.",
                "NumberInt-Wert liegt außerhalb des Int32-Bereichs.",
            ),
            (
                "NumberLong value exceeds the i64 range.",
                "NumberLong-Wert überschreitet den i64-Bereich.",
            ),
            (
                "Timestamp time value must fit into u32.",
                "Timestamp-Zeitwert muss in u32 passen.",
            ),
            (
                "Value must be boolean, numeric, or a string equal to true/false.",
                "Der Wert muss boolesch, numerisch oder eine Zeichenkette gleich true/false sein.",
            ),
            (
                "Value must be a number or a string.",
                "Der Wert muss eine Zahl oder eine Zeichenkette sein.",
            ),
            (
                "Collection name in getCollection must be a quoted string.",
                "Der Sammlungsname in getCollection muss eine Zeichenkette in Anführungszeichen sein.",
            ),
            (
                "The first argument to db.adminCommand must be a document.",
                "Das erste Argument von db.adminCommand muss ein Dokument sein.",
            ),
            (
                "Code point 0x{} is not a valid character.",
                "Codepunkt 0x{} ist kein gültiges Zeichen.",
            ),
            ("Constructor '{}' is not supported.", "Konstruktor '{}' wird nicht unterstützt."),
            (
                "Method db.{} is not supported. Available methods: stats, runCommand, adminCommand, watch.",
                "Methode db.{} wird nicht unterstützt. Verfügbare Methoden: stats, runCommand, adminCommand, watch.",
            ),
            ("Collection filters configured", "Sammlungsfilter konfiguriert"),
            (
                "Failed to determine the tab to refresh indexes.",
                "Tab zum Aktualisieren der Indizes konnte nicht bestimmt werden.",
            ),
            (
                "Failed to convert Decimal128 to a number.",
                "Decimal128 konnte nicht in eine Zahl konvertiert werden.",
            ),
            (
                "Failed to convert string to date.",
                "Zeichenkette konnte nicht in ein Datum konvertiert werden.",
            ),
            (
                "Unable to decode the BinData base64 string.",
                "Die BinData-Base64-Zeichenkette konnte nicht dekodiert werden.",
            ),
            (
                "Unable to construct a date with the specified components.",
                "Das Datum konnte mit den angegebenen Komponenten nicht erstellt werden.",
            ),
            (
                "Cannot convert value of type {other:?} to a date.",
                "Wert vom Typ {other:?} kann nicht in ein Datum konvertiert werden.",
            ),
            (
                "Invalid character '{}' in the method name.",
                "Ungültiges Zeichen '{}' im Methodennamen.",
            ),
            (
                "Invalid hex character '{}' in escape sequence.",
                "Ungültiges Hex-Zeichen '{}' in Escape-Sequenz.",
            ),
            ("No active connection", "Keine aktive Verbindung"),
            ("No active connection.", "Keine aktive Verbindung."),
            (
                "New collection name must differ from the current one.",
                "Der neue Sammlungsname muss sich vom aktuellen unterscheiden.",
            ),
            (
                "New collection name cannot be empty.",
                "Der neue Sammlungsname darf nicht leer sein.",
            ),
            (
                "Collection name cannot be empty.",
                "Der Sammlungsname darf nicht leer sein.",
            ),
            (
                "A collection with this name already exists.",
                "Eine Sammlung mit diesem Namen existiert bereits.",
            ),
            ("RegExp options must be a string.", "RegExp-Optionen müssen eine Zeichenkette sein."),
            (
                "countDocuments options must be a JSON object.",
                "countDocuments-Optionen müssen ein JSON-Objekt sein.",
            ),
            (
                "deleteOne/deleteMany options must be a JSON object.",
                "deleteOne/deleteMany-Optionen müssen ein JSON-Objekt sein.",
            ),
            (
                "estimatedDocumentCount options must be a JSON object.",
                "estimatedDocumentCount-Optionen müssen ein JSON-Objekt sein.",
            ),
            (
                "findOneAndDelete options must be a JSON object.",
                "findOneAndDelete-Optionen müssen ein JSON-Objekt sein.",
            ),
            (
                "findOneAndReplace options must be a JSON object.",
                "findOneAndReplace-Optionen müssen ein JSON-Objekt sein.",
            ),
            (
                "findOneAndUpdate options must be a JSON object.",
                "findOneAndUpdate-Optionen müssen ein JSON-Objekt sein.",
            ),
            (
                "insertMany options must be a JSON object.",
                "insertMany-Optionen müssen ein JSON-Objekt sein.",
            ),
            (
                "insertOne options must be a JSON object.",
                "insertOne-Optionen müssen ein JSON-Objekt sein.",
            ),
            ("replace options must be a JSON object.", "replace-Optionen müssen ein JSON-Objekt sein."),
            ("update options must be a JSON object.", "update-Optionen müssen ein JSON-Objekt sein."),
            (
                "Parameter 'arrayFilters' is not supported when remove=true.",
                "Parameter 'arrayFilters' wird nicht unterstützt, wenn remove=true.",
            ),
            (
                "Parameter 'bypassDocumentValidation' is not supported when remove=true.",
                "Parameter 'bypassDocumentValidation' wird nicht unterstützt, wenn remove=true.",
            ),
            (
                "Parameter 'hint' must be a string or a JSON object.",
                "Parameter 'hint' muss eine Zeichenkette oder ein JSON-Objekt sein.",
            ),
            (
                "Parameter 'new' must be a boolean.",
                "Parameter 'new' muss ein boolescher Wert sein.",
            ),
            (
                "Parameter 'ordered' in insertMany options must be a boolean.",
                "Parameter 'ordered' in insertMany-Optionen muss ein boolescher Wert sein.",
            ),
            (
                "Parameter 'remove' must be a boolean.",
                "Parameter 'remove' muss ein boolescher Wert sein.",
            ),
            (
                "Parameter 'returnOriginal' must be a boolean.",
                "Parameter 'returnOriginal' muss ein boolescher Wert sein.",
            ),
            (
                "Parameter 'update' must not be set together with remove=true.",
                "Parameter 'update' darf nicht zusammen mit remove=true gesetzt werden.",
            ),
            (
                "Parameter 'upsert' is not supported when remove=true.",
                "Parameter 'upsert' wird nicht unterstützt, wenn remove=true.",
            ),
            ("Parameter '{}' must be a JSON object.", "Parameter '{}' muss ein JSON-Objekt sein."),
            (
                "Parameter '{}' must be a boolean value (true/false).",
                "Parameter '{}' muss ein boolescher Wert (true/false) sein.",
            ),
            ("Parameter '{}' must be a string.", "Parameter '{}' muss eine Zeichenkette sein."),
            (
                "Parameter '{}' must be a non-negative integer.",
                "Parameter '{}' muss eine nicht negative ganze Zahl sein.",
            ),
            ("Parameter '{}' must be a timestamp.", "Parameter '{}' muss ein Timestamp sein."),
            (
                "Parameter '{}' has an unsupported value '{}'.",
                "Parameter '{}' hat einen nicht unterstützten Wert '{}'.",
            ),
            ("Parameter '{}' must fit into u32.", "Parameter '{}' muss in u32 passen."),
            (
                "Parameter '{}' is not supported in findOneAndModify.",
                "Parameter '{}' wird in findOneAndModify nicht unterstützt.",
            ),
            (
                "Parameter '{}' is not supported in countDocuments options. Allowed: limit, skip, hint, maxTimeMS.",
                "Parameter '{}' wird in countDocuments-Optionen nicht unterstützt. Zulässig: limit, skip, hint, maxTimeMS.",
            ),
            (
                "Parameter '{}' is not supported in watch options. Allowed: fullDocument, fullDocumentBeforeChange, maxAwaitTimeMS, batchSize, collation, showExpandedEvents, comment, startAtOperationTime.",
                "Parameter '{}' wird in watch-Optionen nicht unterstützt. Zulässig: fullDocument, fullDocumentBeforeChange, maxAwaitTimeMS, batchSize, collation, showExpandedEvents, comment, startAtOperationTime.",
            ),
            (
                "Parameter '{}' is not supported in watch options. Resume tokens are not supported.",
                "Parameter '{}' wird in watch-Optionen nicht unterstützt. Resume-Tokens werden nicht unterstützt.",
            ),
            (
                "Parameter '{}' is not supported in aggregate options. Allowed: allowDiskUse, batchSize, bypassDocumentValidation, collation, comment, hint, let, maxTimeMS, cursor.",
                "Parameter '{}' wird in aggregate-Optionen nicht unterstützt. Zulässig: allowDiskUse, batchSize, bypassDocumentValidation, collation, comment, hint, let, maxTimeMS, cursor.",
            ),
            (
                "Parameter '{}' is not supported in aggregate cursor options. Allowed: batchSize.",
                "Parameter '{}' wird in aggregate-Cursor-Optionen nicht unterstützt. Zulässig: batchSize.",
            ),
            (
                "Parameter '{}' is not supported in distinct options. Allowed: maxTimeMS, collation.",
                "Parameter '{}' wird in distinct-Optionen nicht unterstützt. Zulässig: maxTimeMS, collation.",
            ),
            (
                "Parameter '{}' is not supported in deleteOne/deleteMany options. Allowed: writeConcern, collation, hint.",
                "Parameter '{}' wird in deleteOne/deleteMany-Optionen nicht unterstützt. Zulässig: writeConcern, collation, hint.",
            ),
            (
                "Parameter '{}' is not supported in estimatedDocumentCount options. Only maxTimeMS is allowed.",
                "Parameter '{}' wird in estimatedDocumentCount-Optionen nicht unterstützt. Nur maxTimeMS ist zulässig.",
            ),
            (
                "Parameter '{}' is not supported in findOneAndDelete options. Allowed: writeConcern, maxTimeMS, projection, sort, collation, hint, let, comment.",
                "Parameter '{}' wird in findOneAndDelete-Optionen nicht unterstützt. Zulässig: writeConcern, maxTimeMS, projection, sort, collation, hint, let, comment.",
            ),
            (
                "Parameter '{}' is not supported in findOneAndReplace options. Allowed: writeConcern, upsert, bypassDocumentValidation, maxTimeMS, projection, returnDocument, sort, collation, hint, let, comment.",
                "Parameter '{}' wird in findOneAndReplace-Optionen nicht unterstützt. Zulässig: writeConcern, upsert, bypassDocumentValidation, maxTimeMS, projection, returnDocument, sort, collation, hint, let, comment.",
            ),
            (
                "Parameter '{}' is not supported in findOneAndUpdate options. Allowed: writeConcern, upsert, arrayFilters, bypassDocumentValidation, maxTimeMS, projection, returnDocument, sort, collation, hint, let, comment.",
                "Parameter '{}' wird in findOneAndUpdate-Optionen nicht unterstützt. Zulässig: writeConcern, upsert, arrayFilters, bypassDocumentValidation, maxTimeMS, projection, returnDocument, sort, collation, hint, let, comment.",
            ),
            (
                "Parameter '{}' is not supported in insertMany options. Allowed: writeConcern, ordered.",
                "Parameter '{}' wird in insertMany-Optionen nicht unterstützt. Zulässig: writeConcern, ordered.",
            ),
            (
                "Parameter '{}' is not supported in insertOne options. Allowed: writeConcern.",
                "Parameter '{}' wird in insertOne-Optionen nicht unterstützt. Zulässig: writeConcern.",
            ),
            (
                "Parameter '{}' is not supported in replaceOne options. Allowed: writeConcern, upsert, collation, hint, bypassDocumentValidation, let, comment, sort.",
                "Parameter '{}' wird in replaceOne-Optionen nicht unterstützt. Zulässig: writeConcern, upsert, collation, hint, bypassDocumentValidation, let, comment, sort.",
            ),
            (
                "Parameter '{}' is not supported in updateOne/updateMany options. Allowed: writeConcern, upsert, arrayFilters, collation, hint, bypassDocumentValidation, let, comment, sort.",
                "Parameter '{}' wird in updateOne/updateMany-Optionen nicht unterstützt. Zulässig: writeConcern, upsert, arrayFilters, collation, hint, bypassDocumentValidation, let, comment, sort.",
            ),
            (
                "Parameter '{}' is not supported inside writeConcern. Allowed: w, j, wtimeout.",
                "Parameter '{}' wird innerhalb von writeConcern nicht unterstützt. Zulässig: w, j, wtimeout.",
            ),
            (
                "Parameters 'fields' and 'projection' cannot be set at the same time.",
                "Die Parameter 'fields' und 'projection' können nicht gleichzeitig gesetzt werden.",
            ),
            (
                "Parameters 'new' and 'returnOriginal' conflict.",
                "Die Parameter 'new' und 'returnOriginal' stehen in Konflikt.",
            ),
            (
                "Document return options are not supported when remove=true.",
                "Optionen zur Dokumentrückgabe werden nicht unterstützt, wenn remove=true.",
            ),
            (
                "The first argument to Timestamp must be a number or a date; received {}.",
                "Das erste Argument von Timestamp muss eine Zahl oder ein Datum sein; erhalten {}.",
            ),
            (
                "The first argument to db.runCommand must be a document.",
                "Das erste Argument von db.runCommand muss ein Dokument sein.",
            ),
            (
                "Only one method call is supported after specifying the database.",
                "Nach Angabe der Datenbank wird nur ein Methodenaufruf unterstützt.",
            ),
            (
                "Only one method call is supported after specifying the collection.",
                "Nach Angabe der Sammlung wird nur ein Methodenaufruf unterstützt.",
            ),
            (
                "BinData subtype must be a number or a hex string.",
                "BinData-Untertyp muss eine Zahl oder eine Hex-Zeichenkette sein.",
            ),
            (
                "BinData subtype must be a number from 0 to 255.",
                "BinData-Untertyp muss eine Zahl von 0 bis 255 sein.",
            ),
            ("BinData subtype must be a number.", "BinData-Untertyp muss eine Zahl sein."),
            (
                "An empty update array is not supported. Add at least one stage.",
                "Ein leeres Update-Array wird nicht unterstützt. Mindestens eine Stage hinzufügen.",
            ),
            (
                "Regular expression is not terminated with '/'.",
                "Der reguläre Ausdruck ist nicht mit '/' abgeschlossen.",
            ),
            (
                "Call parenthesis for {} is not closed.",
                "Die Aufrufklammer für {} ist nicht geschlossen.",
            ),
            ("No saved connections", "Keine gespeicherten Verbindungen"),
            ("Auth", "Auth"),
            ("SSH", "SSH"),
            (
                "Double-quoted string is not closed.",
                "Die Zeichenkette in doppelten Anführungszeichen ist nicht geschlossen.",
            ),
            (
                "Single-quoted string is not closed.",
                "Die Zeichenkette in einfachen Anführungszeichen ist nicht geschlossen.",
            ),
            ("String must be true or false.", "Die Zeichenkette muss true oder false sein."),
            (
                "String value in Timestamp must be a number or an ISO date.",
                "Der Zeichenkettenwert in Timestamp muss eine Zahl oder ein ISO-Datum sein.",
            ),
            (
                "Failed to convert string value to number.",
                "Zeichenkettenwert konnte nicht in eine Zahl konvertiert werden.",
            ),
            ("Provide a database name.", "Datenbanknamen angeben."),
            ("No filters configured", "Keine Filter konfiguriert"),
            (
                "Function is missing a closing brace.",
                "Der Funktion fehlt eine schließende Klammer.",
            ),
            (
                "arrayFilters element at index {} must be a JSON object.",
                "Das arrayFilters-Element am Index {} muss ein JSON-Objekt sein.",
            ),
            (
                "Pipeline element at index {} must be a JSON object.",
                "Das Pipeline-Element am Index {} muss ein JSON-Objekt sein.",
            ),
        ])
    })
}
