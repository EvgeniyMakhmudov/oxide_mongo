use std::collections::HashMap;
use std::sync::OnceLock;

pub(crate) fn portuguese_map() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            ("Expand Hierarchically", "Expandir hierarquicamente"),
            ("Collapse Hierarchically", "Recolher hierarquicamente"),
            ("Expand All Hierarchically", "Expandir tudo hierarquicamente"),
            ("Collapse All Hierarchically", "Recolher tudo hierarquicamente"),
            ("Copy JSON", "Copiar JSON"),
            ("Duplicate Tab", "Duplicar aba"),
            ("Tab Color", "Cor da aba"),
            ("Reset Tab Color", "Redefinir cor da aba"),
            ("View", "Visualizar"),
            ("Table", "Tabela"),
            ("Text", "Texto"),
            (
                "Text view is available only for document results",
                "A visualização de texto está disponível apenas para resultados de documentos",
            ),
            ("Copy Key", "Copiar chave"),
            ("Copy Value", "Copiar valor"),
            ("Copy Path", "Copiar caminho"),
            ("Edit Value Only...", "Editar apenas valor..."),
            ("Delete Index", "Excluir índice"),
            ("Hide Index", "Ocultar índice"),
            ("Unhide Index", "Mostrar índice"),
            ("comment expects a value.", "comment espera um valor."),
            (
                "explain must be followed by find(...).",
                "explain deve ser seguido por find(...).",
            ),
            ("finish does not take any arguments.", "finish não aceita argumentos."),
            (
                "No methods are supported after finish().",
                "Nenhum método é suportado após finish().",
            ),
            ("Edit Index...", "Editar índice..."),
            ("Edit Document...", "Editar documento..."),
            ("Cancel", "Cancelar"),
            ("Save", "Salvar"),
            ("Field value will be modified", "O valor do campo será modificado"),
            ("Value Type", "Tipo de valor"),
            ("Saving value...", "Salvando valor..."),
            ("Name", "Nome"),
            ("Address/Host/IP", "Endereço/Host/IP"),
            ("Port", "Porta"),
            (
                "Add filter for system databases",
                "Adicionar filtro para bancos de dados do sistema",
            ),
            ("Include", "Incluir"),
            ("Exclude", "Excluir"),
            ("Testing...", "Testando..."),
            ("Test", "Testar"),
            ("No connections", "Sem conexões"),
            ("Create Database", "Criar banco de dados"),
            ("Refresh", "Atualizar"),
            ("Server Status", "Status do servidor"),
            ("Close", "Fechar"),
            ("No databases", "Sem bancos de dados"),
            ("Statistics", "Estatísticas"),
            ("Drop Database", "Excluir banco de dados"),
            ("Loading collections...", "Carregando coleções..."),
            ("No collections", "Sem coleções"),
            ("Open Empty Tab", "Abrir aba vazia"),
            ("View Documents", "Ver documentos"),
            ("Help", "Ajuda"),
            ("Documentation", "Documentação"),
            ("General information", "Informações gerais"),
            ("Quick start", "Início rápido"),
            ("Supported commands", "Comandos suportados"),
            ("Change stream", "Fluxo de alterações"),
            ("Search", "Pesquisar"),
            ("Matches", "Resultados"),
            ("Change Stream", "Fluxo de alterações"),
            ("Delete Documents...", "Excluir documentos..."),
            ("Delete All Documents...", "Excluir todos os documentos..."),
            ("Rename Collection...", "Renomear coleção..."),
            ("Drop Collection...", "Excluir coleção..."),
            ("Create Collection", "Criar coleção"),
            ("Create Index", "Criar índice"),
            ("Indexes", "Índices"),
            ("About", "Sobre"),
            ("Licenses", "Licenças"),
            ("Primary licenses", "Licenças principais"),
            ("Color schemes", "Esquemas de cores"),
            ("Fonts", "Fontes"),
            ("License", "Licença"),
            ("Link", "Link"),
            ("Unknown", "Desconhecido"),
            ("Version", "Versão"),
            ("Homepage", "Página inicial"),
            ("Project started", "Projeto iniciado"),
            ("Author", "Autor"),
            (
                "MongoDB GUI client for browsing collections, running queries, and managing data.",
                "Cliente GUI do MongoDB para navegar em coleções, executar consultas e gerenciar dados.",
            ),
            ("No tabs opened", "Nenhuma aba aberta"),
            ("No active tab", "Nenhuma aba ativa"),
            ("Connections", "Conexões"),
            ("Create", "Criar"),
            ("Edit", "Editar"),
            ("Delete", "Excluir"),
            ("Connect", "Conectar"),
            ("Cancel", "Cancelar"),
            ("Deleted", "Excluído"),
            ("Save error: ", "Erro ao salvar: "),
            ("Delete \"{}\"?", "Excluir \"{}\"?"),
            ("Yes", "Sim"),
            ("No", "Não"),
            ("connection", "conexão"),
            ("Unknown client", "Cliente desconhecido"),
            ("Connecting...", "Conectando..."),
            ("Ready", "Pronto"),
            ("Send", "Enviar"),
            ("Executing...", "Executando..."),
            ("Canceling...", "Cancelando..."),
            ("Canceled", "Cancelado"),
            ("Completed", "Concluído"),
            ("No results", "Sem resultados"),
            ("{} ms", "{} ms"),
            ("{} documents", "{} documentos"),
            ("Error:", "Erro:"),
            (
                "Query not executed yet. Compose a query and press Send.",
                "A consulta ainda não foi executada. Criar uma consulta e pressionar Enviar.",
            ),
            (
                "Connection inactive. Reconnect and run the query again.",
                "Conexão inativa. Conectar novamente e executar a consulta novamente.",
            ),
            (
                "Query not yet executed. Compose a query and press Send.",
                "A consulta ainda não foi executada. Criar uma consulta e pressionar Enviar.",
            ),
            ("Loading serverStatus...", "Carregando serverStatus..."),
            ("New connection", "Nova conexão"),
            ("Edit connection", "Editar conexão"),
            ("Connection established", "Conexão estabelecida"),
            ("Selected connection not found", "Conexão selecionada não encontrada"),
            ("Saved", "Salvo"),
            ("General", "Geral"),
            ("Database filter", "Filtro de banco de dados"),
            ("Authorization", "Autorização"),
            ("SSH tunnel", "Túnel SSH"),
            ("Use", "Usar"),
            ("Login", "Usuário"),
            ("Password", "Senha"),
            ("Private key", "Chave privada"),
            ("Text key", "Chave em texto"),
            ("Passphrase", "Frase secreta"),
            ("Prompt for password", "Solicitar senha"),
            ("Store in file", "Salvar em arquivo"),
            ("Authentication mechanism", "Mecanismo de autenticação"),
            ("Authentication method", "Método de autenticação"),
            ("Database", "Banco de dados"),
            ("Connection type", "Tipo de conexão"),
            ("Direct connection", "Conexão direta"),
            ("ReplicaSet", "ReplicaSet"),
            ("Login cannot be empty", "Nome de usuário não pode estar vazio"),
            ("Database cannot be empty", "Banco de dados não pode estar vazio"),
            ("Password cannot be empty", "Senha não pode estar vazia"),
            ("Server address", "Endereço do servidor"),
            ("Server port", "Porta do servidor"),
            ("Username", "Nome de usuário"),
            (
                "SSH port must be a number between 0 and 65535",
                "A porta SSH deve ser um número entre 0 e 65535",
            ),
            (
                "SSH server address cannot be empty",
                "Endereço do servidor SSH não pode estar vazio",
            ),
            (
                "SSH username cannot be empty",
                "Nome de usuário SSH não pode estar vazio",
            ),
            ("SSH password cannot be empty", "Senha SSH não pode estar vazia"),
            (
                "SSH private key cannot be empty",
                "Chave privada SSH não pode estar vazia",
            ),
            (
                "SSH private key file not found",
                "Arquivo de chave privada SSH não encontrado",
            ),
            (
                "SSH known_hosts file not found",
                "Arquivo known_hosts SSH não encontrado",
            ),
            (
                "Failed to read SSH known_hosts",
                "Falha ao ler known_hosts SSH",
            ),
            (
                "Failed to read SSH host key",
                "Falha ao ler chave do host SSH",
            ),
            (
                "SSH host key mismatch",
                "Chave do host SSH não corresponde",
            ),
            (
                "SSH host is not present in known_hosts",
                "Host SSH não está presente em known_hosts",
            ),
            (
                "SSH known_hosts check failed",
                "Verificação de known_hosts SSH falhou",
            ),
            (
                "SSH private key text is not supported on this platform",
                "Chave privada SSH em texto não é suportada nesta plataforma",
            ),
            (
                "SSH password authentication failed. Check username and password.",
                "Autenticação por senha SSH falhou. Verificar nome de usuário e senha.",
            ),
            (
                "SSH private key passphrase is required.",
                "Frase secreta da chave privada SSH é obrigatória.",
            ),
            (
                "SSH private key passphrase is incorrect.",
                "Frase secreta da chave privada SSH está incorreta.",
            ),
            ("Database name", "Nome do banco de dados"),
            ("First collection name", "Nome da primeira coleção"),
            ("Collection Name", "Nome da coleção"),
            ("Settings", "Configurações"),
            ("Settings Error", "Erro de configurações"),
            (
                "Failed to load settings:",
                "Falha ao carregar configurações:",
            ),
            (
                "Failed to apply settings:",
                "Falha ao aplicar configurações:",
            ),
            (
                "Unable to load the settings file. Use defaults or exit.",
                "Não foi possível carregar o arquivo de configurações. Usar padrões ou sair.",
            ),
            ("Use Defaults", "Usar padrões"),
            ("Exit", "Sair"),
            ("Behavior", "Comportamento"),
            ("Appearance", "Aparência"),
            ("Color Theme", "Tema de cores"),
            ("Expand first result item", "Expandir o primeiro item do resultado"),
            (
                "Query timeout (seconds)",
                "Timeout da consulta (segundos)",
            ),
            ("Seconds", "Segundos"),
            ("Sort fields alphabetically", "Ordenar campos alfabeticamente"),
            (
                "Sort index names alphabetically",
                "Ordenar nomes de índices alfabeticamente",
            ),
            (
                "Close related tabs when closing a database",
                "Fechar abas relacionadas ao fechar um banco de dados",
            ),
            ("Enable logging", "Ativar log"),
            ("Log level", "Nível de log"),
            ("Log file path", "Caminho do arquivo de log"),
            ("Path", "Caminho"),
            ("Language", "Idioma"),
            ("English", "Inglês"),
            ("Russian", "Russo"),
            ("Primary Font", "Fonte primária"),
            ("Query Result Font", "Fonte do resultado da consulta"),
            ("Query Editor Font", "Fonte do editor de consultas"),
            ("Font Size", "Tamanho da fonte"),
            ("Theme", "Tema"),
            ("Widget Surfaces", "Superfícies de widgets"),
            ("Widget Background", "Fundo do widget"),
            ("Widget Border", "Borda do widget"),
            ("Subtle Buttons", "Botões sutis"),
            ("Primary Buttons", "Botões primários"),
            ("Table Rows", "Linhas da tabela"),
            ("Even Row", "Linha par"),
            ("Odd Row", "Linha ímpar"),
            ("Header Background", "Fundo do cabeçalho"),
            ("Separator", "Separador"),
            ("Menu Items", "Itens do menu"),
            ("Menu Background", "Fundo do menu"),
            ("Menu Hover Background", "Fundo do menu ao passar o mouse"),
            ("Menu Text", "Texto do menu"),
            ("Default Colors", "Cores padrão"),
            ("Active", "Ativo"),
            ("Hover", "Hover"),
            ("Pressed", "Pressionado"),
            ("Text", "Texto"),
            ("Border", "Borda"),
            ("Apply", "Aplicar"),
            ("System Default", "Padrão do sistema"),
            ("Monospace", "Monoespaçada"),
            ("Serif", "Serif"),
            ("System", "Sistema"),
            ("Light", "Claro"),
            ("Dark", "Escuro"),
            ("localhost", "localhost"),
            ("27017", "27017"),
            ("serverStatus", "serverStatus"),
            ("admin", "admin"),
            ("stats", "stats"),
            ("collStats", "collStats"),
            ("indexes", "indexes"),
            (
                "db.runCommand({ serverStatus: 1 })",
                "db.runCommand({ serverStatus: 1 })",
            ),
            ("Delete All Documents", "Excluir todos os documentos"),
            (
                "All documents from collection \"{}\" in database \"{}\" will be deleted. This action cannot be undone.",
                "Todos os documentos da coleção \"{}\" no banco de dados \"{}\" serão excluídos. Esta ação não pode ser desfeita.",
            ),
            (
                "Confirm deletion of all documents by entering the collection name \"{}\".",
                "Confirmar a exclusão de todos os documentos inserindo o nome da coleção \"{}\".",
            ),
            ("Confirm Deletion", "Confirmar exclusão"),
            ("Delete Collection", "Excluir coleção"),
            (
                "Collection \"{}\" in database \"{}\" will be deleted along with all documents. This action cannot be undone.",
                "A coleção \"{}\" no banco de dados \"{}\" será excluída junto com todos os documentos. Esta ação não pode ser desfeita.",
            ),
            (
                "Confirm deletion of the collection by entering its name \"{}\".",
                "Confirmar a exclusão da coleção inserindo o nome \"{}\".",
            ),
            ("Rename Collection", "Renomear coleção"),
            (
                "Enter a new name for collection \"{}\" in database \"{}\".",
                "Inserir um novo nome para a coleção \"{}\" no banco de dados \"{}\".",
            ),
            (
                "Enter a name for the new collection in database \"{}\".",
                "Inserir um nome para a nova coleção no banco de dados \"{}\".",
            ),
            ("New Collection Name", "Novo nome da coleção"),
            ("Rename", "Renomear"),
            ("Delete Index", "Excluir índice"),
            (
                "Index \"{}\" of collection \"{}\" in database \"{}\" will be deleted. This action cannot be undone.",
                "O índice \"{}\" da coleção \"{}\" no banco de dados \"{}\" será excluído. Esta ação não pode ser desfeita.",
            ),
            (
                "Confirm index deletion by entering its name \"{}\".",
                "Confirmar a exclusão do índice inserindo o nome \"{}\".",
            ),
            (
                "updateOne expects a filter, an update, and an optional options object.",
                "updateOne espera um filtro, um update e um objeto de opções opcional.",
            ),
            (
                "updateMany expects a filter, an update, and an optional options object.",
                "updateMany espera um filtro, um update e um objeto de opções opcional.",
            ),
            (
                "replaceOne expects a filter, a replacement document, and an optional options object.",
                "replaceOne espera um filtro, um documento de substituição e um objeto de opções opcional.",
            ),
            (
                "findOneAndUpdate expects a filter, an update, and an optional options object.",
                "findOneAndUpdate espera um filtro, um update e um objeto de opções opcional.",
            ),
            (
                "findOneAndReplace expects a filter, a replacement document, and an optional options object.",
                "findOneAndReplace espera um filtro, um documento de substituição e um objeto de opções opcional.",
            ),
            (
                "findOneAndDelete expects a filter and an optional options object.",
                "findOneAndDelete espera um filtro e um objeto de opções opcional.",
            ),
            (
                "deleteOne requires a filter as the first argument.",
                "deleteOne requer um filtro como primeiro argumento.",
            ),
            (
                "deleteOne accepts a filter and an optional options object.",
                "deleteOne aceita um filtro e um objeto de opções opcional.",
            ),
            (
                "deleteMany requires a filter as the first argument.",
                "deleteMany requer um filtro como primeiro argumento.",
            ),
            (
                "deleteMany accepts a filter and an optional options object.",
                "deleteMany aceita um filtro e um objeto de opções opcional.",
            ),
            (
                "Method {} is not supported. Available methods: find, watch, findOne, count, countDocuments, estimatedDocumentCount, distinct, aggregate, insertOne, insertMany, updateOne, updateMany, replaceOne, findOneAndUpdate, findOneAndReplace, findOneAndDelete, deleteOne, deleteMany, createIndex, createIndexes, dropIndex, dropIndexes, getIndexes, hideIndex, unhideIndex.",
                "O método {} não é suportado. Métodos disponíveis: find, watch, findOne, count, countDocuments, estimatedDocumentCount, distinct, aggregate, insertOne, insertMany, updateOne, updateMany, replaceOne, findOneAndUpdate, findOneAndReplace, findOneAndDelete, deleteOne, deleteMany, createIndex, createIndexes, dropIndex, dropIndexes, getIndexes, hideIndex, unhideIndex.",
            ),
            (
                "watch accepts at most one argument (the pipeline array).",
                "watch aceita no máximo um argumento (o array pipeline).",
            ),
            (
                "watch pipeline element at index {} must be an object.",
                "O elemento do pipeline watch no índice {} deve ser um objeto.",
            ),
            (
                "watch pipeline must be an array of stages or a single stage object.",
                "O pipeline watch deve ser um array de stages ou um único objeto stage.",
            ),
            (
                "watch supports at most two arguments: pipeline and options.",
                "watch suporta no máximo dois argumentos: pipeline e options.",
            ),
            (
                "watch options must be a JSON object.",
                "As opções de watch devem ser um objeto JSON.",
            ),
            (
                "Invalid character in the collection name:",
                "Caractere inválido no nome da coleção:",
            ),
            (
                "Query must start with db.<collection>, db.getCollection('<collection>'), rs.<method>, or a supported database method.",
                "A consulta deve começar com db.<collection>, db.getCollection('<collection>'), rs.<method> ou um método de banco de dados suportado.",
            ),
            (
                "Query must start with db.<collection>, db.getCollection('<collection>'), rs.<method>, or a supported method.",
                "A consulta deve começar com db.<collection>, db.getCollection('<collection>'), rs.<method> ou um método suportado.",
            ),
            (
                "Only one method call is supported after specifying the replica set helper.",
                "Apenas uma chamada de método é suportada após especificar o helper do replica set.",
            ),
            (
                "Method rs.{} is not supported. Available methods: status, conf, isMaster, hello, printReplicationInfo, printSecondaryReplicationInfo, initiate, reconfig, stepDown, freeze, add, addArb, remove, syncFrom, slaveOk.",
                "O método rs.{} não é suportado. Métodos disponíveis: status, conf, isMaster, hello, printReplicationInfo, printSecondaryReplicationInfo, initiate, reconfig, stepDown, freeze, add, addArb, remove, syncFrom, slaveOk.",
            ),
            (
                "Method rs.{} does not accept arguments.",
                "O método rs.{} não aceita argumentos.",
            ),
            (
                "rs.initiate expects no arguments or a config document.",
                "rs.initiate espera nenhum argumento ou um documento de configuração.",
            ),
            (
                "rs.reconfig expects a config document and an optional options document.",
                "rs.reconfig espera um documento de configuração e um documento de opções opcional.",
            ),
            (
                "rs.stepDown expects an optional number of seconds and an optional secondary catch-up period.",
                "rs.stepDown espera um número opcional de segundos e um período opcional de recuperação dos secundários.",
            ),
            (
                "rs.freeze expects a number of seconds.",
                "rs.freeze espera um número de segundos.",
            ),
            (
                "rs.add expects a host string or a member document.",
                "rs.add espera uma string de host ou um documento de membro.",
            ),
            (
                "rs.addArb expects a host string or a member document.",
                "rs.addArb espera uma string de host ou um documento de membro.",
            ),
            (
                "rs.remove expects a host string.",
                "rs.remove espera uma string de host.",
            ),
            (
                "rs.syncFrom expects a host string.",
                "rs.syncFrom espera uma string de host.",
            ),
            (
                "Replica set config response does not contain a config document.",
                "A resposta de configuração do replica set não contém um documento de configuração.",
            ),
            (
                "Replica set config must contain a members array of documents.",
                "A configuração do replica set deve conter um array members de documentos.",
            ),
            (
                "Replica set member must include a host string.",
                "O membro do replica set deve incluir uma string de host.",
            ),
            (
                "Replica set member with host '{}' already exists.",
                "Já existe um membro do replica set com o host '{}'.",
            ),
            (
                "Replica set member with host '{}' not found.",
                "Membro do replica set com host '{}' não encontrado.",
            ),
            (
                "Replica set config version must be a number.",
                "A versão da configuração do replica set deve ser um número.",
            ),
            ("Oplog stats are unavailable.", "As estatísticas do oplog não estão disponíveis."),
            (
                "Oplog is empty; cannot compute replication info.",
                "O oplog está vazio; não é possível calcular informações de replicação.",
            ),
            (
                "Oplog entry does not contain a timestamp.",
                "A entrada do oplog não contém um timestamp.",
            ),
            (
                "Replica set status does not contain members.",
                "O status do replica set não contém members.",
            ),
            (
                "Primary member optime is not available.",
                "O optime do membro primário não está disponível.",
            ),
            (
                "slaveOk has no effect in this client.",
                "slaveOk não tem efeito neste cliente.",
            ),
            ("unknown", "desconhecido"),
            (
                "Single-quoted string contains an unfinished escape sequence.",
                "A string com aspas simples contém uma sequência de escape incompleta.",
            ),
            (
                "The \\x sequence must contain two hex digits.",
                "A sequência \\x deve conter dois dígitos hex.",
            ),
            (
                "The \\u sequence must contain four hex digits.",
                "A sequência \\u deve conter quatro dígitos hex.",
            ),
            (
                "Enter the exact database name to confirm.",
                "Inserir o nome exato do banco de dados para confirmar.",
            ),
            (
                "Enter the name of the first collection for the new database.",
                "Inserir o nome da primeira coleção para o novo banco de dados.",
            ),
            (
                "A database with this name already exists.",
                "Já existe um banco de dados com esse nome.",
            ),
            (
                "Document not found. It may have been deleted or the change was not applied.",
                "Documento não encontrado. Ele pode ter sido excluído ou a alteração não foi aplicada.",
            ),
            (
                "Index document must contain a string field named name.",
                "O documento de índice deve conter um campo string chamado name.",
            ),
            (
                "Index name cannot be changed via collMod.",
                "O nome do índice não pode ser alterado via collMod.",
            ),
            (
                "Failed to refresh database list:",
                "Falha ao atualizar a lista de bancos de dados:",
            ),
            ("Failed to delete index", "Falha ao excluir índice"),
            (
                "Database \"{}\" will be deleted along with all collections and documents. This action cannot be undone.",
                "O banco de dados \"{}\" será excluído junto com todas as coleções e documentos. Esta ação não pode ser desfeita.",
            ),
            (
                "Confirm deletion of all data by entering the database name \"{}\".",
                "Confirmar a exclusão de todos os dados inserindo o nome do banco de dados \"{}\".",
            ),
            ("Delete Database", "Excluir banco de dados"),
            ("Processing...", "Processando..."),
            (
                "MongoDB creates a database only when the first collection is created. Provide the database name and the first collection to create immediately.",
                "O MongoDB cria um banco de dados apenas quando a primeira coleção é criada. Informar o nome do banco de dados e a primeira coleção a ser criada imediatamente.",
            ),
            ("Creating database...", "Criando banco de dados..."),
            ("Edit Document", "Editar documento"),
            (
                "Edit the JSON representation of the document. The document will be fully replaced on save.",
                "Editar a representação JSON do documento. O documento será totalmente substituído ao salvar.",
            ),
            ("Saving document...", "Salvando documento..."),
            ("Edit TTL Index", "Editar índice TTL"),
            (
                "Only the \"expireAfterSeconds\" field value can be changed. Other parameters will be ignored.",
                "Apenas o valor do campo \"expireAfterSeconds\" pode ser alterado. Outros parâmetros serão ignorados.",
            ),
            ("Saving index...", "Salvando índice..."),
            ("Expected a document, got", "Esperado um documento, obtido"),
            ("Expected an array, got", "Esperado um array, obtido"),
            ("Expected binary data, got", "Esperado dados binários, obtido"),
            ("Expected JavaScript code, got", "Esperado código JavaScript, obtido"),
            (
                "Expected a regular expression, got",
                "Esperada uma expressão regular, obtido",
            ),
            (
                "Expected JavaScript code with scope, got",
                "Esperado código JavaScript com scope, obtido",
            ),
            ("Expected a Timestamp, got", "Esperado um Timestamp, obtido"),
            ("Expected a DBRef, got", "Esperado um DBRef, obtido"),
            ("Expected a MinKey, got", "Esperado um MinKey, obtido"),
            ("Expected a MaxKey, got", "Esperado um MaxKey, obtido"),
            ("Expected undefined, got", "Esperado undefined, obtido"),
            ("Duration:", "Duração:"),
            ("Element at index", "Elemento no índice"),
            (
                "in insertMany must be a JSON object.",
                "em insertMany deve ser um objeto JSON.",
            ),
            (
                "Value must be an integer in the Int32 range.",
                "O valor deve ser um inteiro no intervalo Int32.",
            ),
            (
                "Value must be an integer in the Int64 range.",
                "O valor deve ser um inteiro no intervalo Int64.",
            ),
            ("Value must be a Double.", "O valor deve ser um Double."),
            (
                "Value must be a valid Decimal128.",
                "O valor deve ser um Decimal128 válido.",
            ),
            (
                "BinData expects a base64 string as the second argument.",
                "BinData espera uma string base64 como segundo argumento.",
            ),
            (
                "BinData expects two arguments: a subtype and a base64 string.",
                "BinData espera dois argumentos: um subtipo e uma string base64.",
            ),
            (
                "DBRef expects an ObjectId as the second argument.",
                "DBRef espera um ObjectId como segundo argumento.",
            ),
            (
                "DBRef expects two or three arguments: collection, _id, and an optional database name.",
                "DBRef espera dois ou três argumentos: collection, _id e um nome de banco de dados opcional.",
            ),
            (
                "Hex string must contain an even number of characters.",
                "A string hex deve conter um número par de caracteres.",
            ),
            (
                "HexData expects two arguments: a subtype and a hex string.",
                "HexData espera dois argumentos: um subtipo e uma string hex.",
            ),
            (
                "HexData expects a string as the second argument.",
                "HexData espera uma string como segundo argumento.",
            ),
            (
                "NumberDecimal expects a valid decimal value.",
                "NumberDecimal espera um valor decimal válido.",
            ),
            ("NumberInt expects an integer.", "NumberInt espera um inteiro."),
            ("NumberLong expects an integer.", "NumberLong espera um inteiro."),
            (
                "Object expects a JSON object, but received a value of type {}.",
                "Object espera um objeto JSON, mas recebeu um valor do tipo {}.",
            ),
            (
                "ObjectId accepts either zero or one string argument.",
                "ObjectId aceita zero ou um argumento string.",
            ),
            (
                "ObjectId requires a 24-character hex string or no arguments.",
                "ObjectId requer uma string hex de 24 caracteres ou nenhum argumento.",
            ),
            (
                "ObjectId.fromDate expects a single argument.",
                "ObjectId.fromDate espera um único argumento.",
            ),
            ("RegExp expects a string pattern.", "RegExp espera um padrão string."),
            (
                "RegExp expects a pattern and optional options.",
                "RegExp espera um padrão e opções opcionais.",
            ),
            (
                "Timestamp expects two arguments: time and increment.",
                "Timestamp espera dois argumentos: time e increment.",
            ),
            (
                "UUID expects a string in the format xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.",
                "UUID espera uma string no formato xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.",
            ),
            (
                "arrayFilters must be an array of objects.",
                "arrayFilters deve ser um array de objetos.",
            ),
            (
                "arrayFilters must contain at least one filter object.",
                "arrayFilters deve conter pelo menos um objeto de filtro.",
            ),
            ("collation must be a JSON object.", "collation deve ser um objeto JSON."),
            (
                "db.runCommand expects a document describing the command.",
                "db.runCommand espera um documento descrevendo o comando.",
            ),
            (
                "db.runCommand supports only one argument (the command document).",
                "db.runCommand suporta apenas um argumento (o documento de comando).",
            ),
            (
                "db.adminCommand expects a document describing the command.",
                "db.adminCommand espera um documento descrevendo o comando.",
            ),
            (
                "db.adminCommand supports only one argument (the command document).",
                "db.adminCommand suporta apenas um argumento (o documento de comando).",
            ),
            (
                "aggregate supports at most two arguments: pipeline and options.",
                "aggregate suporta no máximo dois argumentos: pipeline e options.",
            ),
            (
                "aggregate options must be a JSON object.",
                "As opções de aggregate devem ser um objeto JSON.",
            ),
            (
                "aggregate cursor options must be a JSON object.",
                "As opções do cursor aggregate devem ser um objeto JSON.",
            ),
            (
                "distinct supports at most three arguments: field, filter, and options.",
                "distinct suporta no máximo três argumentos: field, filter e options.",
            ),
            (
                "distinct options must be a JSON object.",
                "As opções de distinct devem ser um objeto JSON.",
            ),
            (
                "findOneAndModify expects a JSON object.",
                "findOneAndModify espera um objeto JSON.",
            ),
            (
                "findOneAndModify requires a JSON object with parameters.",
                "findOneAndModify requer um objeto JSON com parâmetros.",
            ),
            (
                "findOneAndModify requires an 'update' parameter when remove=false.",
                "findOneAndModify requer o parâmetro 'update' quando remove=false.",
            ),
            (
                "hint must be a string or a JSON object with index specification.",
                "hint deve ser uma string ou um objeto JSON com especificação de índice.",
            ),
            (
                "returnDocument must be the string 'before' or 'after'.",
                "returnDocument deve ser a string 'before' ou 'after'.",
            ),
            (
                "writeConcern must be a JSON object.",
                "writeConcern deve ser um objeto JSON.",
            ),
            (
                "writeConcern.j must be a boolean value.",
                "writeConcern.j deve ser um valor booleano.",
            ),
            (
                "writeConcern.w must be a non-negative integer.",
                "writeConcern.w deve ser um inteiro não negativo.",
            ),
            (
                "writeConcern.w must be a string or a number.",
                "writeConcern.w deve ser uma string ou um número.",
            ),
            (
                "writeConcern.w must not exceed the maximum allowed value.",
                "writeConcern.w não deve exceder o valor máximo permitido.",
            ),
            (
                "writeConcern.wtimeout must be a non-negative integer.",
                "writeConcern.wtimeout deve ser um inteiro não negativo.",
            ),
            (
                "db.stats expects a number or an options object.",
                "db.stats espera um número ou um objeto de opções.",
            ),
            ("{}::{} must be a positive integer.", "{}::{} deve ser um inteiro positivo."),
            (
                "{}::{} must be a number, received {}.",
                "{}::{} deve ser um número, recebido {}.",
            ),
            ("{}::{} must fit into u32.", "{}::{} deve caber em u32."),
            ("Argument must be a JSON object.", "O argumento deve ser um objeto JSON."),
            (
                "Argument must be a string or a number.",
                "O argumento deve ser uma string ou um número.",
            ),
            (
                "Index argument must be a string with the index name or an object with keys.",
                "O argumento do índice deve ser uma string com o nome do índice ou um objeto com chaves.",
            ),
            (
                "Update argument must be an object with operators or an array of stages.",
                "O argumento de update deve ser um objeto com operadores ou um array de stages.",
            ),
            (
                "The second argument to Code must be an object.",
                "O segundo argumento de Code deve ser um objeto.",
            ),
            (
                "Enter the exact index name to confirm.",
                "Inserir o nome exato do índice para confirmar.",
            ),
            (
                "Enter the exact collection name to confirm.",
                "Inserir o nome exato da coleção para confirmar.",
            ),
            ("Document must be a JSON object.", "O documento deve ser um objeto JSON."),
            (
                "NumberInt value is out of the Int32 range.",
                "O valor de NumberInt está fora do intervalo Int32.",
            ),
            (
                "NumberLong value exceeds the i64 range.",
                "O valor de NumberLong excede o intervalo i64.",
            ),
            (
                "Timestamp time value must fit into u32.",
                "O valor de time do Timestamp deve caber em u32.",
            ),
            (
                "Value must be boolean, numeric, or a string equal to true/false.",
                "O valor deve ser booleano, numérico ou uma string igual a true/false.",
            ),
            (
                "Value must be a number or a string.",
                "O valor deve ser um número ou uma string.",
            ),
            (
                "Collection name in getCollection must be a quoted string.",
                "O nome da coleção em getCollection deve ser uma string entre aspas.",
            ),
            (
                "The first argument to db.adminCommand must be a document.",
                "O primeiro argumento de db.adminCommand deve ser um documento.",
            ),
            (
                "Code point 0x{} is not a valid character.",
                "O ponto de código 0x{} não é um caractere válido.",
            ),
            (
                "Constructor '{}' is not supported.",
                "O construtor '{}' não é suportado.",
            ),
            (
                "Method db.{} is not supported. Available methods: stats, runCommand, adminCommand, watch.",
                "O método db.{} não é suportado. Métodos disponíveis: stats, runCommand, adminCommand, watch.",
            ),
            ("Collection filters configured", "Filtros de coleção configurados"),
            (
                "Failed to determine the tab to refresh indexes.",
                "Falha ao determinar a aba para atualizar índices.",
            ),
            (
                "Failed to convert Decimal128 to a number.",
                "Falha ao converter Decimal128 para número.",
            ),
            (
                "Failed to convert string to date.",
                "Falha ao converter string para data.",
            ),
            (
                "Unable to decode the BinData base64 string.",
                "Não foi possível decodificar a string base64 do BinData.",
            ),
            (
                "Unable to construct a date with the specified components.",
                "Não foi possível construir uma data com os componentes especificados.",
            ),
            (
                "Cannot convert value of type {other:?} to a date.",
                "Não é possível converter valor do tipo {other:?} para data.",
            ),
            (
                "Invalid character '{}' in the method name.",
                "Caractere inválido '{}' no nome do método.",
            ),
            (
                "Invalid hex character '{}' in escape sequence.",
                "Caractere hex inválido '{}' na sequência de escape.",
            ),
            ("No active connection", "Sem conexão ativa"),
            ("No active connection.", "Sem conexão ativa."),
            (
                "New collection name must differ from the current one.",
                "O novo nome da coleção deve ser diferente do atual.",
            ),
            (
                "New collection name cannot be empty.",
                "O novo nome da coleção não pode estar vazio.",
            ),
            (
                "Collection name cannot be empty.",
                "O nome da coleção não pode estar vazio.",
            ),
            (
                "A collection with this name already exists.",
                "Já existe uma coleção com esse nome.",
            ),
            ("RegExp options must be a string.", "As opções de RegExp devem ser uma string."),
            (
                "countDocuments options must be a JSON object.",
                "As opções de countDocuments devem ser um objeto JSON.",
            ),
            (
                "deleteOne/deleteMany options must be a JSON object.",
                "As opções de deleteOne/deleteMany devem ser um objeto JSON.",
            ),
            (
                "estimatedDocumentCount options must be a JSON object.",
                "As opções de estimatedDocumentCount devem ser um objeto JSON.",
            ),
            (
                "findOneAndDelete options must be a JSON object.",
                "As opções de findOneAndDelete devem ser um objeto JSON.",
            ),
            (
                "findOneAndReplace options must be a JSON object.",
                "As opções de findOneAndReplace devem ser um objeto JSON.",
            ),
            (
                "findOneAndUpdate options must be a JSON object.",
                "As opções de findOneAndUpdate devem ser um objeto JSON.",
            ),
            (
                "insertMany options must be a JSON object.",
                "As opções de insertMany devem ser um objeto JSON.",
            ),
            (
                "insertOne options must be a JSON object.",
                "As opções de insertOne devem ser um objeto JSON.",
            ),
            ("replace options must be a JSON object.", "As opções de replace devem ser um objeto JSON."),
            ("update options must be a JSON object.", "As opções de update devem ser um objeto JSON."),
            (
                "Parameter 'arrayFilters' is not supported when remove=true.",
                "O parâmetro 'arrayFilters' não é suportado quando remove=true.",
            ),
            (
                "Parameter 'bypassDocumentValidation' is not supported when remove=true.",
                "O parâmetro 'bypassDocumentValidation' não é suportado quando remove=true.",
            ),
            (
                "Parameter 'hint' must be a string or a JSON object.",
                "O parâmetro 'hint' deve ser uma string ou um objeto JSON.",
            ),
            (
                "Parameter 'new' must be a boolean.",
                "O parâmetro 'new' deve ser um booleano.",
            ),
            (
                "Parameter 'ordered' in insertMany options must be a boolean.",
                "O parâmetro 'ordered' nas opções de insertMany deve ser um booleano.",
            ),
            (
                "Parameter 'remove' must be a boolean.",
                "O parâmetro 'remove' deve ser um booleano.",
            ),
            (
                "Parameter 'returnOriginal' must be a boolean.",
                "O parâmetro 'returnOriginal' deve ser um booleano.",
            ),
            (
                "Parameter 'update' must not be set together with remove=true.",
                "O parâmetro 'update' não deve ser definido junto com remove=true.",
            ),
            (
                "Parameter 'upsert' is not supported when remove=true.",
                "O parâmetro 'upsert' não é suportado quando remove=true.",
            ),
            (
                "Parameter '{}' must be a JSON object.",
                "O parâmetro '{}' deve ser um objeto JSON.",
            ),
            (
                "Parameter '{}' must be a boolean value (true/false).",
                "O parâmetro '{}' deve ser um valor booleano (true/false).",
            ),
            ("Parameter '{}' must be a string.", "O parâmetro '{}' deve ser uma string."),
            (
                "Parameter '{}' must be a non-negative integer.",
                "O parâmetro '{}' deve ser um inteiro não negativo.",
            ),
            (
                "Parameter '{}' must be a timestamp.",
                "O parâmetro '{}' deve ser um timestamp.",
            ),
            (
                "Parameter '{}' has an unsupported value '{}'.",
                "O parâmetro '{}' tem um valor não suportado '{}'.",
            ),
            ("Parameter '{}' must fit into u32.", "O parâmetro '{}' deve caber em u32."),
            (
                "Parameter '{}' is not supported in findOneAndModify.",
                "O parâmetro '{}' não é suportado em findOneAndModify.",
            ),
            (
                "Parameter '{}' is not supported in countDocuments options. Allowed: limit, skip, hint, maxTimeMS.",
                "O parâmetro '{}' não é suportado nas opções de countDocuments. Permitidos: limit, skip, hint, maxTimeMS.",
            ),
            (
                "Parameter '{}' is not supported in watch options. Allowed: fullDocument, fullDocumentBeforeChange, maxAwaitTimeMS, batchSize, collation, showExpandedEvents, comment, startAtOperationTime.",
                "O parâmetro '{}' não é suportado nas opções de watch. Permitidos: fullDocument, fullDocumentBeforeChange, maxAwaitTimeMS, batchSize, collation, showExpandedEvents, comment, startAtOperationTime.",
            ),
            (
                "Parameter '{}' is not supported in watch options. Resume tokens are not supported.",
                "O parâmetro '{}' não é suportado nas opções de watch. Resume tokens não são suportados.",
            ),
            (
                "Parameter '{}' is not supported in aggregate options. Allowed: allowDiskUse, batchSize, bypassDocumentValidation, collation, comment, hint, let, maxTimeMS, cursor.",
                "O parâmetro '{}' não é suportado nas opções de aggregate. Permitidos: allowDiskUse, batchSize, bypassDocumentValidation, collation, comment, hint, let, maxTimeMS, cursor.",
            ),
            (
                "Parameter '{}' is not supported in aggregate cursor options. Allowed: batchSize.",
                "O parâmetro '{}' não é suportado nas opções de cursor de aggregate. Permitidos: batchSize.",
            ),
            (
                "Parameter '{}' is not supported in distinct options. Allowed: maxTimeMS, collation.",
                "O parâmetro '{}' não é suportado nas opções de distinct. Permitidos: maxTimeMS, collation.",
            ),
            (
                "Parameter '{}' is not supported in deleteOne/deleteMany options. Allowed: writeConcern, collation, hint.",
                "O parâmetro '{}' não é suportado nas opções de deleteOne/deleteMany. Permitidos: writeConcern, collation, hint.",
            ),
            (
                "Parameter '{}' is not supported in estimatedDocumentCount options. Only maxTimeMS is allowed.",
                "O parâmetro '{}' não é suportado nas opções de estimatedDocumentCount. Apenas maxTimeMS é permitido.",
            ),
            (
                "Parameter '{}' is not supported in findOneAndDelete options. Allowed: writeConcern, maxTimeMS, projection, sort, collation, hint, let, comment.",
                "O parâmetro '{}' não é suportado nas opções de findOneAndDelete. Permitidos: writeConcern, maxTimeMS, projection, sort, collation, hint, let, comment.",
            ),
            (
                "Parameter '{}' is not supported in findOneAndReplace options. Allowed: writeConcern, upsert, bypassDocumentValidation, maxTimeMS, projection, returnDocument, sort, collation, hint, let, comment.",
                "O parâmetro '{}' não é suportado nas opções de findOneAndReplace. Permitidos: writeConcern, upsert, bypassDocumentValidation, maxTimeMS, projection, returnDocument, sort, collation, hint, let, comment.",
            ),
            (
                "Parameter '{}' is not supported in findOneAndUpdate options. Allowed: writeConcern, upsert, arrayFilters, bypassDocumentValidation, maxTimeMS, projection, returnDocument, sort, collation, hint, let, comment.",
                "O parâmetro '{}' não é suportado nas opções de findOneAndUpdate. Permitidos: writeConcern, upsert, arrayFilters, bypassDocumentValidation, maxTimeMS, projection, returnDocument, sort, collation, hint, let, comment.",
            ),
            (
                "Parameter '{}' is not supported in insertMany options. Allowed: writeConcern, ordered.",
                "O parâmetro '{}' não é suportado nas opções de insertMany. Permitidos: writeConcern, ordered.",
            ),
            (
                "Parameter '{}' is not supported in insertOne options. Allowed: writeConcern.",
                "O parâmetro '{}' não é suportado nas opções de insertOne. Permitidos: writeConcern.",
            ),
            (
                "Parameter '{}' is not supported in replaceOne options. Allowed: writeConcern, upsert, collation, hint, bypassDocumentValidation, let, comment, sort.",
                "O parâmetro '{}' não é suportado nas opções de replaceOne. Permitidos: writeConcern, upsert, collation, hint, bypassDocumentValidation, let, comment, sort.",
            ),
            (
                "Parameter '{}' is not supported in updateOne/updateMany options. Allowed: writeConcern, upsert, arrayFilters, collation, hint, bypassDocumentValidation, let, comment, sort.",
                "O parâmetro '{}' não é suportado nas opções de updateOne/updateMany. Permitidos: writeConcern, upsert, arrayFilters, collation, hint, bypassDocumentValidation, let, comment, sort.",
            ),
            (
                "Parameter '{}' is not supported inside writeConcern. Allowed: w, j, wtimeout.",
                "O parâmetro '{}' não é suportado dentro de writeConcern. Permitidos: w, j, wtimeout.",
            ),
            (
                "Parameters 'fields' and 'projection' cannot be set at the same time.",
                "Os parâmetros 'fields' e 'projection' não podem ser definidos ao mesmo tempo.",
            ),
            (
                "Parameters 'new' and 'returnOriginal' conflict.",
                "Os parâmetros 'new' e 'returnOriginal' estão em conflito.",
            ),
            (
                "Document return options are not supported when remove=true.",
                "As opções de retorno de documento não são suportadas quando remove=true.",
            ),
            (
                "The first argument to Timestamp must be a number or a date; received {}.",
                "O primeiro argumento de Timestamp deve ser um número ou uma data; recebido {}.",
            ),
            (
                "The first argument to db.runCommand must be a document.",
                "O primeiro argumento de db.runCommand deve ser um documento.",
            ),
            (
                "Only one method call is supported after specifying the database.",
                "Apenas uma chamada de método é suportada após especificar o banco de dados.",
            ),
            (
                "Only one method call is supported after specifying the collection.",
                "Apenas uma chamada de método é suportada após especificar a coleção.",
            ),
            (
                "BinData subtype must be a number or a hex string.",
                "O subtipo BinData deve ser um número ou uma string hex.",
            ),
            (
                "BinData subtype must be a number from 0 to 255.",
                "O subtipo BinData deve ser um número de 0 a 255.",
            ),
            ("BinData subtype must be a number.", "O subtipo BinData deve ser um número."),
            (
                "An empty update array is not supported. Add at least one stage.",
                "Um array de update vazio não é suportado. Adicionar pelo menos uma stage.",
            ),
            (
                "Regular expression is not terminated with '/'.",
                "A expressão regular não é terminada com '/'.",
            ),
            (
                "Call parenthesis for {} is not closed.",
                "O parêntese de chamada para {} não está fechado.",
            ),
            ("No saved connections", "Sem conexões salvas"),
            ("Auth", "Auth"),
            ("SSH", "SSH"),
            (
                "Double-quoted string is not closed.",
                "A string com aspas duplas não está fechada.",
            ),
            (
                "Single-quoted string is not closed.",
                "A string com aspas simples não está fechada.",
            ),
            (
                "String must be true or false.",
                "A string deve ser true ou false.",
            ),
            (
                "String value in Timestamp must be a number or an ISO date.",
                "O valor string em Timestamp deve ser um número ou uma data ISO.",
            ),
            (
                "Failed to convert string value to number.",
                "Falha ao converter valor string para número.",
            ),
            ("Provide a database name.", "Informar um nome de banco de dados."),
            ("No filters configured", "Sem filtros configurados"),
            (
                "Function is missing a closing brace.",
                "A função está sem uma chave de fechamento.",
            ),
            (
                "arrayFilters element at index {} must be a JSON object.",
                "O elemento arrayFilters no índice {} deve ser um objeto JSON.",
            ),
            (
                "Pipeline element at index {} must be a JSON object.",
                "O elemento do pipeline no índice {} deve ser um objeto JSON.",
            ),
        ])
    })
}
