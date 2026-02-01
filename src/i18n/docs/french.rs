use std::collections::HashMap;
use std::sync::OnceLock;

use super::DocSection;

pub(super) fn french_docs() -> &'static HashMap<&'static str, DocSection> {
    static MAP: OnceLock<HashMap<&'static str, DocSection>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            (
                "general",
                DocSection {
                    title: "Informations générales",
                    markdown: r#"# Informations générales

## Note importante

Oxide Mongo n'est pas un remplacement complet du mongo shell standard et n'inclut pas d'interpréteur JavaScript.
L'application ne cherche pas à reproduire tout le shell. Elle émule plutôt les commandes les plus courantes, rendant l'utilisation de MongoDB confortable dans une interface graphique.

Si vous avez besoin d'un environnement JavaScript complet ou de scripts complexes, mongo shell reste le choix par défaut.
Oxide Mongo se concentre sur les tâches quotidiennes, la consultation des données et le travail rapide avec la base de données.

## À propos du projet

Oxide Mongo est un client GUI multiplateforme et léger pour MongoDB.
Le projet est inspiré du remarquable Robomongo (ensuite Robo3T), qui n'est plus réellement maintenu aujourd'hui.

L'objectif est de conserver la philosophie de Robomongo :
- minimalisme plutôt qu'une interface surchargée
- démarrage rapide et faible consommation de ressources
- aucune limitation intrusive

Oxide Mongo est un outil libre et ouvert pour les développeurs et administrateurs qui ont besoin d'un accès rapide et clair à MongoDB sans complexité supplémentaire.
"#,
                },
            ),
            (
                "quick-start",
                DocSection {
                    title: "Démarrage rapide",
                    markdown: r#"# Démarrage rapide

## Premier lancement

Ci-dessous, un scénario pas à pas pour un premier lancement lorsque vous avez déjà une base de données et devez rechercher et modifier un document.

- Ouvrez le menu "Connexions" et cliquez sur "Créer".
- Renseignez les paramètres de connexion : adresse, port, base de données. Activez l'authentification et/ou le tunnel SSH si nécessaire.
- Cliquez sur "Tester" et assurez-vous que la vérification réussit.
- Cliquez sur "Enregistrer", puis sélectionnez la connexion créée et cliquez sur "Connecter".
- Dans le panneau de gauche, développez la base et la collection, puis ouvrez un onglet.
- Dans l'éditeur de requêtes, saisissez une recherche, par exemple `db.getCollection('my_collection').find({})`, et cliquez sur "Envoyer".
- Dans le tableau des résultats, sélectionnez un document et ouvrez le menu contextuel "Modifier le document...".
- Modifiez le document et cliquez sur "Enregistrer".
- Si nécessaire, exécutez `find(...)` à nouveau pour vérifier que les données ont été mises à jour.
"#,
                },
            ),
            (
                "supported-commands",
                DocSection {
                    title: "Commandes prises en charge",
                    markdown: r#"# Liste des commandes prises en charge :

## Pour les collections :

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

Pour find(...), les méthodes suivantes sont prises en charge :

    sort(...), hint(...), limit(...), skip(...), maxTimeMS(...), explain(), count(...), countDocuments(...), comment(...)

## Pour les bases de données

    db.stats(...)
    db.runCommand(...)
    db.adminCommand(...)
    db.watch(...)

## Pour les helpers de replica set

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
                    title: "Flux de changements",
                    markdown: r#"# Flux de changements

## Fonctionnement

La commande `watch(...)` démarre un flux de changements. La requête ne renvoie pas tous les documents d'un coup. Elle attend de nouveaux événements et les ajoute au tableau au fur et à mesure.

## Fin du flux

Le flux s'arrête automatiquement lorsque le nombre d'éléments reçus atteint la valeur `limit`. Après cela, la requête est considérée comme terminée et le temps d'exécution est affiché.
"#,
                },
            ),
            (
                "hotkeys",
                DocSection {
                    title: "Raccourcis clavier",
                    markdown: r#"# Raccourcis clavier

- F2 — basculer les résultats en vue Tableau
- F4 — basculer les résultats en vue Texte
- Ctrl+Enter — exécuter la requête actuelle
- Ctrl+W — fermer l'onglet actif
"#,
                },
            ),
        ])
    })
}
