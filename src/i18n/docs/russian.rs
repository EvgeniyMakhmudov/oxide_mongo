use std::collections::HashMap;
use std::sync::OnceLock;

use super::DocSection;

pub(super) fn russian_docs() -> &'static HashMap<&'static str, DocSection> {
    static MAP: OnceLock<HashMap<&'static str, DocSection>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            (
                "general",
                DocSection {
                    title: "Общие сведения",
                    markdown: r#"# Общие сведения

## Важное замечание

Oxide Mongo не является полной заменой стандартного mongo shell и не содержит встроенного JavaScript-интерпретатора.
Приложение не стремится повторить весь функционал shell один в один — вместо этого Oxide Mongo эмулирует выполнение наиболее востребованных команд, делая работу с MongoDB удобной через графический интерфейс.

Если вам требуется полноценная JS-среда или сложные скрипты — mongo shell по-прежнему остаётся выбором по умолчанию.
Oxide Mongo ориентирован на повседневные задачи, просмотр данных и быструю работу с базой.

## О проекте

Oxide Mongo — это кроссплатформенный, простой и лёгкий GUI-клиент для MongoDB.
Проект вдохновлён великолепным Robomongo (позднее — Robo3T), который, к сожалению, сегодня фактически не развивается.

Цель — сохранить философию Robomongo:
- минимализм вместо перегруженного интерфейса
- быстрый запуск и низкое потребление ресурсов
- отсутствие навязчивых ограничений

Oxide Mongo — это открытый и бесплатный инструмент, подходящий как для разработчиков, так и для администраторов, которым нужен быстрый и понятный доступ к MongoDB без лишней сложности.
"#,
                },
            ),
            (
                "quick-start",
                DocSection {
                    title: "Быстрый старт",
                    markdown: r#"# Быстрый старт

## Первый запуск

Ниже — пошаговый сценарий для первого запуска, когда у вас уже есть база и нужно выполнить поиск и изменить документ.

- Откройте меню "Connections" и нажмите "Create".
- Заполните параметры подключения: адрес, порт, база данных. Если требуется — включите авторизацию и/или SSH туннель.
- Нажмите "Test" и убедитесь, что проверка прошла успешно.
- Нажмите "Save", затем выберите созданное соединение и нажмите "Connect".
- В левой панели раскройте нужную базу и коллекцию, откройте вкладку.
- В редакторе запросов введите поиск, например `db.getCollection('my_collection').find({})`, и нажмите "Send".
- В таблице результатов выберите документ и откройте контекстное меню "Edit Document...".
- Внесите изменения в документ и нажмите "Save".
- При необходимости выполните `find(...)` ещё раз, чтобы убедиться, что данные обновились.

"#,
                },
            ),
            (
                "supported-commands",
                DocSection {
                    title: "Поддерживаемые команды",
                    markdown: r#"# Список поддержанных команд:

## Для коллекции:

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

Для find(...) поддержаны команды:

    sort(...), hint(...), limit(...), skip(...), maxTimeMS(...), explain(), count(...), countDocuments(...)

## Для базы данных

    db.stats(...)
    db.runCommand(...)
    db.adminCommand(...)
    db.watch(...)

## Для работы с репликами

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
                    title: "Стрим изменений",
                    markdown: r#"# Change stream

## Как это работает

Команда `watch(...)` запускает стрим событий по изменениям. Запрос не возвращает все документы сразу — он ожидает новые события и добавляет их в таблицу по мере поступления.

## Когда завершается

Стрим автоматически останавливается, когда количество полученных элементов достигает значения `limit`. После этого запрос считается завершённым и отображается время выполнения.
"#,
                },
            ),
        ])
    })
}
