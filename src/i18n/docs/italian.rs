use std::collections::HashMap;
use std::sync::OnceLock;

use super::DocSection;

pub(super) fn italian_docs() -> &'static HashMap<&'static str, DocSection> {
    static MAP: OnceLock<HashMap<&'static str, DocSection>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            (
                "general",
                DocSection {
                    title: "Informazioni generali",
                    markdown: r#"# Informazioni generali

## Nota importante

Oxide Mongo non è un sostituto completo del mongo shell standard e non include un interprete JavaScript.
L'applicazione non mira a replicare l'intera shell. Invece emula i comandi più comuni, rendendo comodo lavorare con MongoDB in una GUI.

Se hai bisogno di un ambiente JavaScript completo o di script complessi, mongo shell rimane la scelta predefinita.
Oxide Mongo si concentra su attività quotidiane, esplorazione dei dati e lavoro rapido con il database.

## Sul progetto

Oxide Mongo è un client GUI multipiattaforma e leggero per MongoDB.
Il progetto è ispirato all'eccellente Robomongo (in seguito Robo3T), che oggi è praticamente non mantenuto.

L'obiettivo è mantenere la filosofia di Robomongo:
- minimalismo invece di un'interfaccia sovraccarica
- avvio rapido e basso consumo di risorse
- senza limitazioni intrusive

Oxide Mongo è uno strumento aperto e gratuito per sviluppatori e amministratori che necessitano di un accesso rapido e chiaro a MongoDB senza complessità aggiuntiva.
"#,
                },
            ),
            (
                "quick-start",
                DocSection {
                    title: "Avvio rapido",
                    markdown: r#"# Avvio rapido

## Primo avvio

Di seguito uno scenario passo-passo per il primo avvio quando hai già un database e devi cercare e modificare un documento.

- Apri il menu "Connessioni" e fai clic su "Crea".
- Compila i parametri di connessione: indirizzo, porta, database. Abilita autenticazione e/o tunnel SSH se necessario.
- Fai clic su "Test" e assicurati che la verifica abbia successo.
- Fai clic su "Salva", quindi seleziona la connessione creata e fai clic su "Connetti".
- Nel pannello sinistro espandi il database e la collezione, quindi apri una scheda.
- Nell'editor delle query inserisci una ricerca, ad esempio `db.getCollection('my_collection').find({})`, e fai clic su "Invia".
- Nella tabella dei risultati seleziona un documento e apri il menu contestuale "Modifica documento...".
- Modifica il documento e fai clic su "Salva".
- Se necessario, esegui di nuovo `find(...)` per verificare che i dati siano stati aggiornati.
"#,
                },
            ),
            (
                "supported-commands",
                DocSection {
                    title: "Comandi supportati",
                    markdown: r#"# Elenco comandi supportati:

## Per collezioni:

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

Per find(...), sono supportati i seguenti metodi:

    sort(...), hint(...), limit(...), skip(...), maxTimeMS(...), explain(), count(...), countDocuments(...), comment(...)

## Per database

    db.stats(...)
    db.runCommand(...)
    db.adminCommand(...)
    db.watch(...)

## Per helper replica set

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

## Come funziona

Il comando `watch(...)` avvia un change stream. La query non restituisce tutti i documenti in una volta, ma attende nuovi eventi e li aggiunge alla tabella quando arrivano.

## Quando termina

Lo stream si interrompe automaticamente quando il numero di elementi ricevuti raggiunge il valore `limit`. Dopo di che la query è considerata completa e viene mostrato il tempo di esecuzione.
"#,
                },
            ),
            (
                "hotkeys",
                DocSection {
                    title: "Scorciatoie da tastiera",
                    markdown: r#"# Scorciatoie da tastiera

- F2 — passa i risultati alla vista Tabella
- F4 — passa i risultati alla vista Testo
- Ctrl+Enter — esegui la query corrente
- Ctrl+W — chiudi la scheda attiva
"#,
                },
            ),
        ])
    })
}
