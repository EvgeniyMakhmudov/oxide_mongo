use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    English,
    Russian,
}

static CURRENT_LANGUAGE: OnceLock<RwLock<Language>> = OnceLock::new();

pub const ALL_LANGUAGES: &[Language] = &[Language::English, Language::Russian];

impl Language {
    pub fn label(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Russian => "Russian",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(tr(self.label()))
    }
}

fn language_lock() -> &'static RwLock<Language> {
    CURRENT_LANGUAGE.get_or_init(|| RwLock::new(Language::Russian))
}

pub fn init_language(language: Language) {
    if CURRENT_LANGUAGE.set(RwLock::new(language)).is_err() {
        set_language(language);
    }
}

pub fn set_language(language: Language) {
    let mut guard = language_lock().write().expect("language write lock poisoned");
    *guard = language;
}

fn current_language() -> Language {
    *language_lock().read().expect("language read lock poisoned")
}

fn russian_map() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            ("Expand Hierarchically", "Развернуть иерархично"),
            ("Collapse Hierarchically", "Свернуть иерархично"),
            ("Expand All Hierarchically", "Развернуть иерархично всё"),
            ("Collapse All Hierarchically", "Свернуть иерархично всё"),
            ("Copy JSON", "Копировать JSON"),
            ("View", "Вид"),
            ("Table", "Таблица"),
            ("Text", "Текст"),
            ("Text view is available only for document results", "Текстовый режим доступен только для результатов в виде документов"),
            ("Copy Key", "Копировать ключ"),
            ("Copy Value", "Копировать значение"),
            ("Copy Path", "Копировать путь"),
            ("Edit Value Only...", "Изменить только значение..."),
            ("Delete Index", "Удалить индекс"),
            ("Hide Index", "Спрятать индекс"),
            ("Unhide Index", "Не прятать индекс"),
            ("Edit Index...", "Изменить индекс..."),
            ("Edit Document...", "Изменить документ..."),
            ("Cancel", "Отмена"),
            ("Save", "Сохранить"),
            ("Field value will be modified", "Будет изменено значение поля"),
            ("Value Type", "Тип значения"),
            ("Saving value...", "Сохранение значения..."),
            ("Name", "Название"),
            ("Address/Host/IP", "Адрес/Хост/IP"),
            ("Port", "Порт"),
            ("Add filter for system databases", "Добавить фильтр системные базы данных"),
            ("Include", "Включить"),
            ("Exclude", "Исключить"),
            ("Testing...", "Тестирование..."),
            ("Test", "Тестировать"),
            ("No connections", "Соединений нет"),
            ("Create Database", "Создать базу данных"),
            ("Refresh", "Обновить"),
            ("Server Status", "Статус сервера"),
            ("Close", "Закрыть"),
            ("No databases", "Нет баз данных"),
            ("Statistics", "Статистика"),
            ("Drop Database", "Удалить БД"),
            ("Loading collections...", "Загрузка коллекций..."),
            ("No collections", "Нет коллекций"),
            ("Open Empty Tab", "Открыть пустую вкладку"),
            ("View Documents", "Посмотреть документы"),
            ("Help", "Справка"),
            ("Documentation", "Документация"),
            ("Change Stream", "Стрим изменений"),
            ("Delete Documents...", "Удалить документы..."),
            ("Delete All Documents...", "Удалить все документы..."),
            ("Rename Collection...", "Переименовать коллекцию..."),
            ("Drop Collection...", "Удалить коллекцию..."),
            ("Create Collection", "Создать коллекцию"),
            ("Create Index", "Создать индекс"),
            ("Indexes", "Индексы"),
            ("About", "О проекте"),
            ("Homepage", "Домашняя страница"),
            ("Project started", "Год начала проекта"),
            ("Author", "Автор"),
            (
                "MongoDB GUI client for browsing collections, running queries, and managing data.",
                "GUI-клиент MongoDB для просмотра коллекций, выполнения запросов и управления данными.",
            ),
            ("No tabs opened", "Вкладки не открыты"),
            ("No active tab", "Нет активной вкладки"),
            ("Connections", "Соединения"),
            ("Create", "Создать"),
            ("Edit", "Редактировать"),
            ("Delete", "Удалить"),
            ("Connect", "Соединить"),
            ("Cancel", "Отменить"),
            ("Deleted", "Удалено"),
            ("Save error: ", "Ошибка сохранения: "),
            ("Delete \"{}\"?", "Удалить \"{}\"?"),
            ("Yes", "Да"),
            ("No", "Нет"),
            ("connection", "соединение"),
            ("Unknown client", "Неизвестный клиент"),
            ("Connecting...", "Подключение..."),
            ("Ready", "Готово"),
            ("Send", "Отправить"),
            ("Executing...", "Выполняется..."),
            ("Canceling...", "Отмена..."),
            ("Canceled", "Отменено"),
            ("Completed", "Завершено"),
            ("No results", "Нет результатов"),
            ("{} ms", "{} мс"),
            ("{} documents", "{} документов"),
            ("Error:", "Ошибка:"),
            (
                "Query not executed yet. Compose a query and press Send.",
                "Запрос пока не выполнен. Сформируйте запрос и нажмите Send.",
            ),
            (
                "Connection inactive. Reconnect and run the query again.",
                "Соединение не активно. Повторите подключение, затем выполните запрос.",
            ),
            (
                "Query not yet executed. Compose a query and press Send.",
                "Запрос пока не выполнен. Сформируйте запрос и нажмите Send.",
            ),
            ("Loading serverStatus...", "Загрузка serverStatus..."),
            ("New connection", "Новое соединение"),
            ("Edit connection", "Редактирование соединения"),
            ("Connection established", "Соединение установлено"),
            ("Selected connection not found", "Выбранное соединение не найдено"),
            ("Saved", "Сохранено"),
            ("General", "Общее"),
            ("Database filter", "Фильтр баз данных"),
            ("Authorization", "Авторизация"),
            ("SSH tunnel", "SSH туннель"),
            ("Use", "Использовать"),
            ("Login", "Логин"),
            ("Password", "Пароль"),
            ("Private key", "Приватный ключ"),
            ("Text key", "Текстовый ключ"),
            ("Passphrase", "Фраза"),
            ("Prompt for password", "Запрашивать"),
            ("Store in file", "Хранить в файле"),
            ("Authentication mechanism", "Механизм авторизации"),
            ("Authentication method", "Метод аутентификации"),
            ("Database", "База данных"),
            ("Connection type", "Тип соединения"),
            ("Direct connection", "Прямое соединение"),
            ("ReplicaSet", "ReplicaSet"),
            ("Login cannot be empty", "Логин не может быть пустым"),
            ("Database cannot be empty", "База данных не может быть пустой"),
            ("Password cannot be empty", "Пароль не может быть пустым"),
            ("Server address", "Адрес сервера"),
            ("Server port", "Порт сервера"),
            ("Username", "Имя пользователя"),
            (
                "SSH port must be a number between 0 and 65535",
                "Порт SSH должен быть числом от 0 до 65535",
            ),
            (
                "SSH server address cannot be empty",
                "Адрес SSH сервера не может быть пустым",
            ),
            ("SSH username cannot be empty", "Имя пользователя SSH не может быть пустым"),
            ("SSH password cannot be empty", "Пароль SSH не может быть пустым"),
            (
                "SSH private key cannot be empty",
                "Приватный ключ SSH не может быть пустым",
            ),
            (
                "SSH private key file not found",
                "Файл приватного ключа SSH не найден",
            ),
            (
                "SSH known_hosts file not found",
                "Файл known_hosts для SSH не найден",
            ),
            (
                "Failed to read SSH known_hosts",
                "Не удалось прочитать known_hosts для SSH",
            ),
            (
                "Failed to read SSH host key",
                "Не удалось прочитать ключ хоста SSH",
            ),
            ("SSH host key mismatch", "Ключ хоста SSH не совпадает"),
            (
                "SSH host is not present in known_hosts",
                "Хост SSH отсутствует в known_hosts",
            ),
            (
                "SSH known_hosts check failed",
                "Проверка known_hosts для SSH не удалась",
            ),
            (
                "SSH private key text is not supported on this platform",
                "Текстовый приватный ключ SSH не поддерживается на этой платформе",
            ),
            (
                "SSH password authentication failed. Check username and password.",
                "Не удалось пройти SSH-аутентификацию по паролю. Проверьте имя пользователя и пароль.",
            ),
            (
                "SSH private key passphrase is required.",
                "Для приватного ключа SSH требуется парольная фраза.",
            ),
            (
                "SSH private key passphrase is incorrect.",
                "Неверная парольная фраза приватного ключа SSH.",
            ),
            ("Database name", "Название базы данных"),
            ("First collection name", "Имя первой коллекции"),
            ("Collection Name", "Имя коллекции"),
            ("Settings", "Настройки"),
            ("Settings Error", "Ошибка настроек"),
            ("Failed to load settings:", "Не удалось загрузить настройки:"),
            ("Failed to apply settings:", "Не удалось применить настройки:"),
            (
                "Invalid numeric value for \"{}\".",
                "Некорректное числовое значение поля \"{}\".",
            ),
            ("Font size must be greater than zero", "Размер шрифта должен быть больше нуля"),
            (
                "Continue with default settings",
                "Продолжить с настройками по умолчанию",
            ),
            ("Exit", "Выйти"),
            ("Behavior", "Поведение"),
            ("Appearance", "Внешний вид"),
            ("Color Theme", "Цветовое оформление"),
            ("Expand first result item", "Раскрывать первый элемент результата"),
            ("Query timeout (seconds)", "Таймаут запроса (секунды)"),
            ("Seconds", "Секунды"),
            ("Sort fields alphabetically", "Сортировать поля по алфавиту"),
            (
                "Sort index names alphabetically",
                "Сортировать имена индексов по алфавиту",
            ),
            ("Language", "Язык"),
            ("English", "Английский"),
            ("Russian", "Русский"),
            ("Primary Font", "Основной шрифт"),
            ("Query Result Font", "Шрифт результатов запросов"),
            ("Font Size", "Размер шрифта"),
            ("Theme", "Тема"),
            ("Widget Surfaces", "Поверхности виджетов"),
            ("Widget Background", "Фон виджетов"),
            ("Widget Border", "Граница виджетов"),
            ("Subtle Buttons", "Ненавязчивые кнопки"),
            ("Primary Buttons", "Основные кнопки"),
            ("Table Rows", "Строки таблицы"),
            ("Even Row", "Чётная строка"),
            ("Odd Row", "Нечётная строка"),
            ("Header Background", "Фон заголовка"),
            ("Separator", "Разделитель"),
            ("Menu Items", "Пункты меню"),
            ("Menu Background", "Фон меню"),
            ("Menu Hover Background", "Фон меню при наведении"),
            ("Menu Text", "Текст меню"),
            ("Default Colors", "Цвета по умолчанию"),
            ("Active", "Активное состояние"),
            ("Hover", "Наведение"),
            ("Pressed", "Нажатое состояние"),
            ("Text", "Текст"),
            ("Border", "Граница"),
            ("Apply", "Применить"),
            ("System Default", "Системный"),
            ("Monospace", "Моноширинный"),
            ("Serif", "С засечками"),
            ("System", "Система"),
            ("Light", "Светлая"),
            ("Dark", "Тёмная"),
            ("localhost", "localhost"),
            ("27017", "27017"),
            ("serverStatus", "serverStatus"),
            ("admin", "admin"),
            ("stats", "stats"),
            ("collStats", "collStats"),
            ("indexes", "indexes"),
            ("db.runCommand({ serverStatus: 1 })", "db.runCommand({ serverStatus: 1 })"),
            ("Delete All Documents", "Удаление всех документов"),
            (
                "All documents from collection \"{}\" in database \"{}\" will be deleted. This action cannot be undone.",
                "Будут удалены все документы из коллекции \"{}\" базы \"{}\". Это действие нельзя отменить.",
            ),
            (
                "Confirm deletion of all documents by entering the collection name \"{}\".",
                "Подтвердите удаление всех документов введя название коллекции \"{}\".",
            ),
            ("Confirm Deletion", "Подтвердить удаление"),
            ("Delete Collection", "Удаление коллекции"),
            (
                "Collection \"{}\" in database \"{}\" will be deleted along with all documents. This action cannot be undone.",
                "Коллекция \"{}\" в базе \"{}\" будет удалена вместе со всеми документами. Это действие нельзя отменить.",
            ),
            (
                "Confirm deletion of the collection by entering its name \"{}\".",
                "Подтвердите удаление коллекции введя её название \"{}\".",
            ),
            ("Rename Collection", "Переименовать коллекцию"),
            (
                "Enter a new name for collection \"{}\" in database \"{}\".",
                "Введите новое имя для коллекции \"{}\" в базе \"{}\".",
            ),
            (
                "Enter a name for the new collection in database \"{}\".",
                "Введите имя новой коллекции в базе данных \"{}\".",
            ),
            ("New Collection Name", "Новое имя коллекции"),
            ("Rename", "Переименовать"),
            ("Delete Index", "Удаление индекса"),
            (
                "Index \"{}\" of collection \"{}\" in database \"{}\" will be deleted. This action cannot be undone.",
                "Индекс \"{}\" коллекции \"{}\" базы \"{}\" будет удалён. Это действие нельзя отменить.",
            ),
            (
                "Confirm index deletion by entering its name \"{}\".",
                "Подтвердите удаление индекса введя его имя \"{}\".",
            ),
            (
                "updateOne expects a filter, an update, and an optional options object.",
                "updateOne принимает фильтр, обновление и необязательный объект options.",
            ),
            (
                "updateMany expects a filter, an update, and an optional options object.",
                "updateMany принимает фильтр, обновление и необязательный объект options.",
            ),
            (
                "replaceOne expects a filter, a replacement document, and an optional options object.",
                "replaceOne принимает фильтр, документ замену и необязательный объект options.",
            ),
            (
                "findOneAndUpdate expects a filter, an update, and an optional options object.",
                "findOneAndUpdate принимает фильтр, обновление и необязательный объект options.",
            ),
            (
                "findOneAndReplace expects a filter, a replacement document, and an optional options object.",
                "findOneAndReplace принимает фильтр, документ замены и необязательный объект options.",
            ),
            (
                "findOneAndDelete expects a filter and an optional options object.",
                "findOneAndDelete принимает фильтр и необязательный объект options.",
            ),
            (
                "deleteOne requires a filter as the first argument.",
                "deleteOne требует фильтр в качестве первого аргумента.",
            ),
            (
                "deleteOne accepts a filter and an optional options object.",
                "deleteOne принимает фильтр и необязательный объект options.",
            ),
            (
                "deleteMany requires a filter as the first argument.",
                "deleteMany требует фильтр в качестве первого аргумента.",
            ),
            (
                "deleteMany accepts a filter and an optional options object.",
                "deleteMany принимает фильтр и необязательный объект options.",
            ),
            (
                "Method {} is not supported. Available methods: find, watch, findOne, count, countDocuments, estimatedDocumentCount, distinct, aggregate, insertOne, insertMany, updateOne, updateMany, replaceOne, findOneAndUpdate, findOneAndReplace, findOneAndDelete, deleteOne, deleteMany, createIndex, createIndexes, dropIndex, dropIndexes, getIndexes, hideIndex, unhideIndex.",
                "Метод {} не поддерживается. Доступны: find, watch, findOne, count, countDocuments, estimatedDocumentCount, distinct, aggregate, insertOne, insertMany, updateOne, updateMany, replaceOne, findOneAndUpdate, findOneAndReplace, findOneAndDelete, deleteOne, deleteMany, createIndex, createIndexes, dropIndex, dropIndexes, getIndexes, hideIndex, unhideIndex.",
            ),
            (
                "watch accepts at most one argument (the pipeline array).",
                "watch принимает не более одного аргумента (массив pipeline).",
            ),
            (
                "watch pipeline element at index {} must be an object.",
                "Элемент pipeline watch с индексом {} должен быть объектом.",
            ),
            (
                "watch pipeline must be an array of stages or a single stage object.",
                "Pipeline watch должен быть массивом стадий или одним объектом стадии.",
            ),
            (
                "Invalid character in the collection name:",
                "Недопустимый символ в имени коллекции:",
            ),
            (
                "Query must start with db.<collection>, db.getCollection('<collection>'), rs.<method>, or a supported database method.",
                "Запрос должен начинаться с db.<collection>, db.getCollection('<collection>'), rs.<method> или поддерживаемого метода базы.",
            ),
            (
                "Query must start with db.<collection>, db.getCollection('<collection>'), rs.<method>, or a supported method.",
                "Запрос должен начинаться с db.<collection>, db.getCollection('<collection>'), rs.<method> или поддерживаемого метода.",
            ),
            (
                "Only one method call is supported after specifying the replica set helper.",
                "После указания помощника реплика-сета поддерживается только один вызов метода.",
            ),
            (
                "Method rs.{} is not supported. Available methods: status, conf, isMaster, hello, printReplicationInfo, printSecondaryReplicationInfo, initiate, reconfig, stepDown, freeze, add, addArb, remove, syncFrom, slaveOk.",
                "Метод rs.{} не поддерживается. Доступны: status, conf, isMaster, hello, printReplicationInfo, printSecondaryReplicationInfo, initiate, reconfig, stepDown, freeze, add, addArb, remove, syncFrom, slaveOk.",
            ),
            (
                "Method rs.{} does not accept arguments.",
                "Метод rs.{} не принимает аргументы.",
            ),
            (
                "rs.initiate expects no arguments or a config document.",
                "rs.initiate ожидает отсутствие аргументов или документ конфигурации.",
            ),
            (
                "rs.reconfig expects a config document and an optional options document.",
                "rs.reconfig ожидает документ конфигурации и необязательный документ options.",
            ),
            (
                "rs.stepDown expects an optional number of seconds and an optional force flag.",
                "rs.stepDown ожидает необязательное число секунд и необязательный флаг force.",
            ),
            (
                "rs.freeze expects a number of seconds.",
                "rs.freeze ожидает число секунд.",
            ),
            (
                "rs.add expects a host string or a member document.",
                "rs.add ожидает строку с адресом хоста или документ члена.",
            ),
            (
                "rs.addArb expects a host string or a member document.",
                "rs.addArb ожидает строку с адресом хоста или документ члена.",
            ),
            (
                "rs.remove expects a host string.",
                "rs.remove ожидает строку с адресом хоста.",
            ),
            (
                "rs.syncFrom expects a host string.",
                "rs.syncFrom ожидает строку с адресом хоста.",
            ),
            (
                "Replica set config response does not contain a config document.",
                "Ответ конфигурации реплика-сета не содержит документ config.",
            ),
            (
                "Replica set config must contain a members array of documents.",
                "Конфигурация реплика-сета должна содержать массив members из документов.",
            ),
            (
                "Replica set member must include a host string.",
                "Элемент реплика-сета должен содержать строковое поле host.",
            ),
            (
                "Replica set member with host '{}' already exists.",
                "Элемент реплика-сета с host '{}' уже существует.",
            ),
            (
                "Replica set member with host '{}' not found.",
                "Элемент реплика-сета с host '{}' не найден.",
            ),
            (
                "Replica set config version must be a number.",
                "Версия конфигурации реплика-сета должна быть числом.",
            ),
            (
                "Oplog stats are unavailable.",
                "Статистика oplog недоступна.",
            ),
            (
                "Oplog is empty; cannot compute replication info.",
                "Oplog пуст; невозможно вычислить информацию о репликации.",
            ),
            (
                "Oplog entry does not contain a timestamp.",
                "Запись oplog не содержит временной метки.",
            ),
            (
                "Replica set status does not contain members.",
                "Статус реплика-сета не содержит members.",
            ),
            (
                "Primary member optime is not available.",
                "Optime первичного узла недоступен.",
            ),
            (
                "slaveOk has no effect in this client.",
                "slaveOk не влияет на этот клиент.",
            ),
            ("unknown", "неизвестно"),
            (
                "Single-quoted string contains an unfinished escape sequence.",
                "Строка в одинарных кавычках содержит незавершённую escape-последовательность.",
            ),
            (
                "The \\x sequence must contain two hex digits.",
                "Последовательность \\x должна содержать две hex-цифры.",
            ),
            (
                "The \\u sequence must contain four hex digits.",
                "Последовательность \\u должна содержать четыре hex-цифры.",
            ),
            (
                "Enter the exact database name to confirm.",
                "Для подтверждения введите точное имя базы данных.",
            ),
            (
                "Enter the name of the first collection for the new database.",
                "Укажите имя первой коллекции для создаваемой базы.",
            ),
            (
                "A database with this name already exists.",
                "База данных с таким именем уже существует.",
            ),
            (
                "Document not found. It may have been deleted or the change was not applied.",
                "Документ не найден. Возможно, он был удалён или изменение не применено.",
            ),
            (
                "Index document must contain a string field named name.",
                "Документ индекса должен содержать строковое поле name.",
            ),
            (
                "Index name cannot be changed via collMod.",
                "Имя индекса не может быть изменено через collMod.",
            ),
            (
                "Failed to refresh database list:",
                "Не удалось обновить список баз данных:",
            ),
            (
                "Failed to delete index",
                "Ошибка удаления индекса",
            ),
            (
                "Database \"{}\" will be deleted along with all collections and documents. This action cannot be undone.",
                "База данных \"{}\" будет полностью удалена вместе со всеми коллекциями и документами. Это действие нельзя отменить.",
            ),
            (
                "Confirm deletion of all data by entering the database name \"{}\".",
                "Подтвердите удаление всех данных, введя название базы \"{}\".",
            ),
            ("Delete Database", "Удаление базы данных"),
            ("Processing...", "Выполнение операции..."),
            (
                "MongoDB creates a database only when the first collection is created. Provide the database name and the first collection to create immediately.",
                "MongoDB создаёт базу данных только при создании первой коллекции. Укажите имя базы и первой коллекции, которая будет создана сразу.",
            ),
            ("Creating database...", "Создание базы данных..."),
            ("Edit Document", "Изменение документа"),
            (
                "Edit the JSON representation of the document. The document will be fully replaced on save.",
                "Отредактируйте JSON-представление документа. При сохранении документ будет полностью заменён.",
            ),
            ("Saving document...", "Сохранение документа..."),
            ("Edit TTL Index", "Изменение TTL индекса"),
            (
                "Only the \"expireAfterSeconds\" field value can be changed. Other parameters will be ignored.",
                "Можно менять только значение поля \"expireAfterSeconds\". Остальные параметры будут проигнорированы.",
            ),
            ("Saving index...", "Сохранение индекса..."),
            ("Expected a document, got", "Ожидался документ, получено"),
            ("Expected an array, got", "Ожидался массив, получено"),
            ("Expected binary data, got", "Ожидались бинарные данные, получено"),
            ("Expected JavaScript code, got", "Ожидался JavaScript-код, получено"),
            (
                "Expected a regular expression, got",
                "Ожидалось регулярное выражение, получено",
            ),
            (
                "Expected JavaScript code with scope, got",
                "Ожидался JavaScript-код со scope, получено",
            ),
            ("Expected a Timestamp, got", "Ожидался Timestamp, получено"),
            ("Expected a DBRef, got", "Ожидался DBRef, получено"),
            ("Expected a MinKey, got", "Ожидался MinKey, получено"),
            ("Expected a MaxKey, got", "Ожидался MaxKey, получено"),
            ("Expected undefined, got", "Ожидалось значение undefined, получено"),
            ("Duration:", "Время:"),
            ("Element at index", "Элемент с индексом"),
            ("in insertMany must be a JSON object.", "в insertMany должен быть JSON-объектом."),
            ("Value must be an integer in the Int32 range.", "Значение должно быть целым числом в диапазоне Int32."),
            ("Value must be an integer in the Int64 range.", "Значение должно быть целым числом в диапазоне Int64."),
            ("Value must be a Double.", "Значение должно быть числом (Double)."),
            ("Value must be a valid Decimal128.", "Значение должно быть корректным Decimal128."),
            ("BinData expects a base64 string as the second argument.", "BinData ожидает base64-строку вторым аргументом."),
            ("BinData expects two arguments: a subtype and a base64 string.", "BinData ожидает два аргумента: подтип и base64-строку."),
            ("DBRef expects an ObjectId as the second argument.", "DBRef ожидает ObjectId в качестве второго аргумента."),
            ("DBRef expects two or three arguments: collection, _id, and an optional database name.", "DBRef ожидает два или три аргумента: коллекция, _id и опционально имя базы данных."),
            ("Hex string must contain an even number of characters.", "Hex-строка должна содержать чётное количество символов."),
            ("HexData expects two arguments: a subtype and a hex string.", "HexData ожидает два аргумента: подтип и hex-строку."),
            ("HexData expects a string as the second argument.", "HexData ожидает строку во втором аргументе."),
            ("NumberDecimal expects a valid decimal value.", "NumberDecimal ожидает корректное десятичное значение."),
            ("NumberInt expects an integer.", "NumberInt ожидает целое число."),
            ("NumberLong expects an integer.", "NumberLong ожидает целое число."),
            ("Object expects a JSON object, but received a value of type {}.", "Object ожидает JSON-объект, получено значение типа {other:?}."),
            ("ObjectId accepts either zero or one string argument.", "ObjectId поддерживает либо ноль, либо один строковый аргумент."),
            ("ObjectId requires a 24-character hex string or no arguments.", "ObjectId требует 24-символьную hex-строку либо вызывается без аргументов."),
            ("ObjectId.fromDate expects a single argument.", "ObjectId.fromDate ожидает один аргумент."),
            ("RegExp expects a string pattern.", "RegExp ожидает строковый шаблон."),
            ("RegExp expects a pattern and optional options.", "RegExp ожидает шаблон и необязательные опции."),
            ("Timestamp expects two arguments: time and increment.", "Timestamp ожидает два аргумента: время и инкремент."),
            ("UUID expects a string in the format xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.", "UUID ожидает строку формата xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx."),
            ("arrayFilters must be an array of objects.", "arrayFilters должен быть массивом объектов."),
            ("arrayFilters must contain at least one filter object.", "arrayFilters должен содержать хотя бы один объект фильтра."),
            ("collation must be a JSON object.", "collation должен быть JSON-объектом."),
            ("db.runCommand expects a document describing the command.", "db.runCommand ожидает документ с описанием команды."),
            ("db.runCommand supports only one argument (the command document).", "db.runCommand поддерживает только один аргумент (документ команды)."),
            ("db.adminCommand expects a document describing the command.", "db.adminCommand ожидает документ с описанием команды."),
            ("db.adminCommand supports only one argument (the command document).", "db.adminCommand поддерживает только один аргумент (документ команды)."),
            ("findOneAndModify expects a JSON object.", "findOneAndModify ожидает JSON-объект."),
            ("findOneAndModify requires a JSON object with parameters.", "findOneAndModify требует JSON-объект с параметрами."),
            ("findOneAndModify requires an 'update' parameter when remove=false.", "findOneAndModify требует параметр 'update', когда remove=false."),
            ("hint must be a string or a JSON object with index specification.", "hint должен быть строкой или JSON-объектом со спецификацией индекса."),
            ("returnDocument must be the string 'before' or 'after'.", "returnDocument должен быть строкой 'before' или 'after'."),
            ("writeConcern must be a JSON object.", "writeConcern должен быть JSON-объектом."),
            ("writeConcern.j must be a boolean value.", "writeConcern.j должен быть логическим значением."),
            ("writeConcern.w must be a non-negative integer.", "writeConcern.w должен быть неотрицательным целым числом."),
            ("writeConcern.w must be a string or a number.", "writeConcern.w должен быть строкой или числом."),
            ("writeConcern.w must not exceed the maximum allowed value.", "writeConcern.w не должен превышать максимально допустимое значение."),
            ("writeConcern.wtimeout must be a non-negative integer.", "writeConcern.wtimeout должен быть неотрицательным целым числом."),
            ("db.stats expects a number or an options object.", "Аргумент db.stats ожидается числом или объектом с параметрами."),
            ("{}::{} must be a positive integer.", "Аргумент {context}::{field} должен быть положительным целым числом."),
            ("{}::{} must be a number, received {}.", "Аргумент {context}::{field} должен быть числом, получено {other:?}."),
            ("{}::{} must fit into u32.", "Аргумент {context}::{field} должен помещаться в u32."),
            ("Argument must be a JSON object.", "Аргумент должен быть JSON-объектом"),
            ("Argument must be a string or a number.", "Аргумент должен быть строкой или числом."),
            ("Index argument must be a string with the index name or an object with keys.", "Аргумент индекса должен быть строкой с именем индекса или объектом с ключами."),
            ("Update argument must be an object with operators or an array of stages.", "Аргумент обновления должен быть объектом с операторами или массивом стадий."),
            ("The second argument to Code must be an object.", "Второй аргумент Code должен быть объектом."),
            ("Enter the exact index name to confirm.", "Для подтверждения введите точное имя индекса."),
            ("Enter the exact collection name to confirm.", "Для подтверждения введите точное имя коллекции."),
            ("Document must be a JSON object.", "Документ должен быть JSON-объектом."),
            ("NumberInt value is out of the Int32 range.", "Значение NumberInt выходит за диапазон Int32."),
            ("NumberLong value exceeds the i64 range.", "Значение NumberLong выходит за пределы диапазона i64."),
            ("Timestamp time value must fit into u32.", "Значение времени Timestamp должно помещаться в u32."),
            ("Value must be boolean, numeric, or a string equal to true/false.", "Значение должно быть логическим, числовым или строкой со значениями true/false."),
            ("Value must be a number or a string.", "Значение должно быть числом или строкой."),
            ("Collection name in getCollection must be a quoted string.", "Имя коллекции в getCollection должно быть строкой в кавычках."),
            ("The first argument to db.adminCommand must be a document.", "Первый аргумент db.adminCommand должен быть документом."),
            ("Code point 0x{} is not a valid character.", "Кодовая точка 0x{value:04X} не является допустимым символом."),
            ("Constructor '{}' is not supported.", "Конструктор '{identifier}' не поддерживается."),
            ("Method db.{} is not supported. Available methods: stats, runCommand, adminCommand, watch.", "Метод db.{other} не поддерживается. Доступны: stats, runCommand, adminCommand, watch."),
            ("Collection filters configured", "Настроены фильтры коллекций"),
            ("Failed to determine the tab to refresh indexes.", "Не удалось определить вкладку для обновления индексов."),
            ("Failed to convert Decimal128 to a number.", "Не удалось преобразовать Decimal128 в число."),
            ("Failed to convert string to date.", "Не удалось преобразовать строку в дату."),
            ("Unable to decode the BinData base64 string.", "Невозможно декодировать base64-строку BinData."),
            ("Unable to construct a date with the specified components.", "Невозможно построить дату с указанными компонентами."),
            ("Cannot convert value of type {other:?} to a date.", "Невозможно преобразовать значение типа {other:?} в дату."),
            ("Invalid character '{}' in the method name.", "Недопустимый символ '{}' в имени метода."),
            ("Invalid hex character '{}' in escape sequence.", "Некорректный hex-символ '{ch}' в escape-последовательности."),
            ("No active connection", "Нет активного соединения"),
            ("No active connection.", "Нет активного соединения."),
            ("New collection name must differ from the current one.", "Новое имя коллекции должно отличаться от текущего."),
            ("New collection name cannot be empty.", "Новое имя коллекции не может быть пустым."),
            ("Collection name cannot be empty.", "Имя коллекции не может быть пустым."),
            (
                "A collection with this name already exists.",
                "Коллекция с таким именем уже существует.",
            ),
            ("RegExp options must be a string.", "Опции RegExp должны быть строкой."),
            ("countDocuments options must be a JSON object.", "Опции countDocuments должны быть JSON-объектом."),
            ("deleteOne/deleteMany options must be a JSON object.", "Опции deleteOne/deleteMany должны быть JSON-объектом."),
            ("estimatedDocumentCount options must be a JSON object.", "Опции estimatedDocumentCount должны быть JSON-объектом."),
            ("findOneAndDelete options must be a JSON object.", "Опции findOneAndDelete должны быть JSON-объектом."),
            ("findOneAndReplace options must be a JSON object.", "Опции findOneAndReplace должны быть JSON-объектом."),
            ("findOneAndUpdate options must be a JSON object.", "Опции findOneAndUpdate должны быть JSON-объектом."),
            ("insertMany options must be a JSON object.", "Опции insertMany должны быть JSON-объектом."),
            ("insertOne options must be a JSON object.", "Опции insertOne должны быть JSON-объектом."),
            ("replace options must be a JSON object.", "Опции replace должны быть JSON-объектом."),
            ("update options must be a JSON object.", "Опции update должны быть JSON-объектом."),
            ("Parameter 'arrayFilters' is not supported when remove=true.", "Параметр 'arrayFilters' не поддерживается при remove=true."),
            ("Parameter 'bypassDocumentValidation' is not supported when remove=true.", "Параметр 'bypassDocumentValidation' не поддерживается при remove=true."),
            ("Parameter 'hint' must be a string or a JSON object.", "Параметр 'hint' должен быть строкой или JSON-объектом."),
            ("Parameter 'new' must be a boolean.", "Параметр 'new' должен быть булевым значением."),
            ("Parameter 'ordered' in insertMany options must be a boolean.", "Параметр 'ordered' в options insertMany должен быть логическим значением."),
            ("Parameter 'remove' must be a boolean.", "Параметр 'remove' должен быть булевым значением."),
            ("Parameter 'returnOriginal' must be a boolean.", "Параметр 'returnOriginal' должен быть булевым значением."),
            ("Parameter 'update' must not be set together with remove=true.", "Параметр 'update' не должен задаваться вместе с remove=true."),
            ("Parameter 'upsert' is not supported when remove=true.", "Параметр 'upsert' не поддерживается при remove=true."),
            ("Parameter '{}' must be a JSON object.", "Параметр '{field}' должен быть JSON-объектом."),
            ("Parameter '{}' must be a boolean value (true/false).", "Параметр '{field}' должен быть булевым значением (true/false)."),
            ("Parameter '{}' must be a non-negative integer.", "Параметр '{field}' должен быть неотрицательным целым числом."),
            ("Parameter '{}' is not supported in findOneAndModify.", "Параметр '{other}' не поддерживается в findOneAndModify."),
            ("Parameter '{}' is not supported in countDocuments options. Allowed: limit, skip, hint, maxTimeMS.", "Параметр '{other}' не поддерживается в options countDocuments. Доступны: limit, skip, hint, maxTimeMS."),
            ("Parameter '{}' is not supported in deleteOne/deleteMany options. Allowed: writeConcern, collation, hint.", "Параметр '{other}' не поддерживается в options deleteOne/deleteMany. Доступны: writeConcern, collation, hint."),
            ("Parameter '{}' is not supported in estimatedDocumentCount options. Only maxTimeMS is allowed.", "Параметр '{other}' не поддерживается в options estimatedDocumentCount. Доступен только maxTimeMS."),
            ("Parameter '{}' is not supported in findOneAndDelete options. Allowed: writeConcern, maxTimeMS, projection, sort, collation, hint, let, comment.", "Параметр '{other}' не поддерживается в options findOneAndDelete. Доступны: writeConcern, maxTimeMS, projection, sort, collation, hint, let, comment."),
            ("Parameter '{}' is not supported in findOneAndReplace options. Allowed: writeConcern, upsert, bypassDocumentValidation, maxTimeMS, projection, returnDocument, sort, collation, hint, let, comment.", "Параметр '{other}' не поддерживается в options findOneAndReplace. Доступны: writeConcern, upsert, bypassDocumentValidation, maxTimeMS, projection, returnDocument, sort, collation, hint, let, comment."),
            ("Parameter '{}' is not supported in findOneAndUpdate options. Allowed: writeConcern, upsert, arrayFilters, bypassDocumentValidation, maxTimeMS, projection, returnDocument, sort, collation, hint, let, comment.", "Параметр '{other}' не поддерживается в options findOneAndUpdate. Доступны: writeConcern, upsert, arrayFilters, bypassDocumentValidation, maxTimeMS, projection, returnDocument, sort, collation, hint, let, comment."),
            ("Parameter '{}' is not supported in insertMany options. Allowed: writeConcern, ordered.", "Параметр '{other}' не поддерживается в options insertMany. Доступны: writeConcern, ordered."),
            ("Parameter '{}' is not supported in insertOne options. Allowed: writeConcern.", "Параметр '{other}' не поддерживается в options insertOne. Доступно: writeConcern."),
            ("Parameter '{}' is not supported in replaceOne options. Allowed: writeConcern, upsert, collation, hint, bypassDocumentValidation, let, comment, sort.", "Параметр '{other}' не поддерживается в options replaceOne. Доступны: writeConcern, upsert, collation, hint, bypassDocumentValidation, let, comment, sort."),
            ("Parameter '{}' is not supported in updateOne/updateMany options. Allowed: writeConcern, upsert, arrayFilters, collation, hint, bypassDocumentValidation, let, comment, sort.", "Параметр '{other}' не поддерживается в options updateOne/updateMany. Доступны: writeConcern, upsert, arrayFilters, collation, hint, bypassDocumentValidation, let, comment, sort."),
            ("Parameter '{}' is not supported inside writeConcern. Allowed: w, j, wtimeout.", "Параметр '{other}' не поддерживается внутри writeConcern. Доступны: w, j, wtimeout."),
            ("Parameters 'fields' and 'projection' cannot be set at the same time.", "Параметры 'fields' и 'projection' нельзя задавать одновременно."),
            ("Parameters 'new' and 'returnOriginal' conflict.", "Параметры 'new' и 'returnOriginal' конфликтуют."),
            ("Document return options are not supported when remove=true.", "Параметры возврата документа не поддерживаются при remove=true."),
            ("The first argument to Timestamp must be a number or a date; received {}.", "Первый аргумент Timestamp должен быть числом или датой, получено {other:?}."),
            ("The first argument to db.runCommand must be a document.", "Первый аргумент db.runCommand должен быть документом."),
            ("Only one method call is supported after specifying the database.", "Поддерживается только один вызов метода после указания базы данных."),
            ("Only one method call is supported after specifying the collection.", "Поддерживается только один вызов метода после указания коллекции."),
            ("BinData subtype must be a number or a hex string.", "Подтип BinData должен быть числом или hex-строкой."),
            ("BinData subtype must be a number from 0 to 255.", "Подтип BinData должен быть числом от 0 до 255."),
            ("BinData subtype must be a number.", "Подтип BinData должен быть числом."),
            ("An empty update array is not supported. Add at least one stage.", "Пустой массив обновления не поддерживается. Добавьте хотя бы один этап."),
            ("Regular expression is not terminated with '/'.", "Регулярное выражение не закрыто символом '/'."),
            ("Call parenthesis for {} is not closed.", "Скобка вызова {identifier} не закрыта."),
            ("No saved connections", "Сохранённых соединений нет"),
            ("Auth", "AUTH"),
            ("SSH", "SSH"),
            ("Double-quoted string is not closed.", "Строка в двойных кавычках не закрыта."),
            ("Single-quoted string is not closed.", "Строка в одинарных кавычках не закрыта."),
            ("String must be true or false.", "Строка должна быть true или false."),
            ("String value in Timestamp must be a number or an ISO date.", "Строковое значение в Timestamp должно быть числом или ISO-датой."),
            ("Failed to convert string value to number.", "Строковое значение не удалось преобразовать в число."),
            ("Provide a database name.", "Укажите имя базы данных."),
            ("No filters configured", "Фильтры не заданы"),
            ("Function is missing a closing brace.", "Функция не содержит закрывающую фигурную скобку."),
            ("arrayFilters element at index {} must be a JSON object.", "Элемент arrayFilters с индексом {index} должен быть JSON-объектом."),
            ("Pipeline element at index {} must be a JSON object.", "Элемент pipeline под индексом {index} должен быть JSON-объектом."),
        ])
    })
}

fn english_fallback_map() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        russian_map().iter().map(|(english, russian)| (*russian, *english)).collect()
    })
}

pub fn tr(text: &'static str) -> &'static str {
    let english = english_fallback_map().get(text).copied().unwrap_or(text);
    match current_language() {
        Language::English => english,
        Language::Russian => russian_map().get(english).copied().unwrap_or(english),
    }
}

pub fn tr_format(template: &'static str, replacements: &[&str]) -> String {
    let mut result = tr(template).to_owned();
    for value in replacements {
        result = result.replacen("{}", value, 1);
    }
    result
}
