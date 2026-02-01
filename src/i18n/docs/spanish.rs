use std::collections::HashMap;
use std::sync::OnceLock;

use super::DocSection;

pub(super) fn spanish_docs() -> &'static HashMap<&'static str, DocSection> {
    static MAP: OnceLock<HashMap<&'static str, DocSection>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            (
                "general",
                DocSection {
                    title: "Información general",
                    markdown: r#"# Información general

## Nota importante

Oxide Mongo no es un reemplazo completo del mongo shell estándar y no incluye un intérprete de JavaScript.
La aplicación no busca replicar todo el shell. En su lugar, emula los comandos más comunes, haciendo el trabajo con MongoDB cómodo en una GUI.

Si necesitas un entorno completo de JavaScript o scripts complejos, mongo shell sigue siendo la opción por defecto.
Oxide Mongo se centra en tareas diarias, exploración de datos y trabajo rápido con la base de datos.

## Sobre el proyecto

Oxide Mongo es un cliente GUI multiplataforma y ligero para MongoDB.
El proyecto está inspirado en el excelente Robomongo (posteriormente Robo3T), que hoy prácticamente no se mantiene.

El objetivo es mantener la filosofía de Robomongo:
- minimalismo en lugar de una interfaz sobrecargada
- inicio rápido y bajo consumo de recursos
- sin limitaciones intrusivas

Oxide Mongo es una herramienta abierta y gratuita para desarrolladores y administradores que necesitan un acceso rápido y claro a MongoDB sin complejidad extra.
"#,
                },
            ),
            (
                "quick-start",
                DocSection {
                    title: "Inicio rápido",
                    markdown: r#"# Inicio rápido

## Primer inicio

A continuación se muestra un escenario paso a paso para el primer inicio cuando ya tienes una base de datos y necesitas buscar y editar un documento.

- Abre el menú "Conexiones" y haz clic en "Crear".
- Completa los parámetros de conexión: dirección, puerto, base de datos. Habilita autenticación y/o túnel SSH si es necesario.
- Haz clic en "Probar" y asegúrate de que la verificación sea exitosa.
- Haz clic en "Guardar", luego selecciona la conexión creada y haz clic en "Conectar".
- En el panel izquierdo expande la base y la colección, y abre una pestaña.
- En el editor de consultas introduce una búsqueda, por ejemplo `db.getCollection('my_collection').find({})`, y haz clic en "Enviar".
- En la tabla de resultados selecciona un documento y abre el menú contextual "Editar documento...".
- Modifica el documento y haz clic en "Guardar".
- Si es necesario, ejecuta `find(...)` nuevamente para verificar que los datos se actualizaron.
"#,
                },
            ),
            (
                "supported-commands",
                DocSection {
                    title: "Comandos compatibles",
                    markdown: r#"# Lista de comandos compatibles:

## Para colecciones:

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

Para find(...), se admiten los siguientes métodos:

    sort(...), hint(...), limit(...), skip(...), maxTimeMS(...), explain(), count(...), countDocuments(...), comment(...)

## Para bases de datos

    db.stats(...)
    db.runCommand(...)
    db.adminCommand(...)
    db.watch(...)

## Para helpers de replica set

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
                    title: "Flujo de cambios",
                    markdown: r#"# Flujo de cambios

## Cómo funciona

El comando `watch(...)` inicia un flujo de cambios. La consulta no devuelve todos los documentos de una vez. En su lugar, espera nuevos eventos y los agrega a la tabla a medida que llegan.

## Cuándo termina

El flujo se detiene automáticamente cuando la cantidad de elementos recibidos alcanza el valor `limit`. Después de eso, la consulta se considera completa y se muestra el tiempo de ejecución.
"#,
                },
            ),
            (
                "hotkeys",
                DocSection {
                    title: "Atajos de teclado",
                    markdown: r#"# Atajos de teclado

- F2 — cambiar resultados a vista Tabla
- F4 — cambiar resultados a vista Texto
- Ctrl+Enter — ejecutar la consulta actual
- Ctrl+W — cerrar la pestaña activa
"#,
                },
            ),
        ])
    })
}
