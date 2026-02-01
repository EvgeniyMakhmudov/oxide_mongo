use std::collections::HashMap;
use std::sync::OnceLock;

use super::DocSection;

pub(super) fn portuguese_docs() -> &'static HashMap<&'static str, DocSection> {
    static MAP: OnceLock<HashMap<&'static str, DocSection>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            (
                "general",
                DocSection {
                    title: "Informações gerais",
                    markdown: r#"# Informações gerais

## Nota importante

Oxide Mongo não é uma substituição completa do mongo shell padrão e não inclui um interpretador JavaScript.
O aplicativo não pretende reproduzir todo o shell. Em vez disso, ele emula os comandos mais comuns, tornando o trabalho com MongoDB conveniente em uma GUI.

Se você precisa de um ambiente JavaScript completo ou scripts complexos, o mongo shell ainda é a opção padrão.
Oxide Mongo foca em tarefas do dia a dia, navegação de dados e trabalho rápido com o banco de dados.

## Sobre o projeto

Oxide Mongo é um cliente GUI leve e multiplataforma para MongoDB.
O projeto é inspirado no excelente Robomongo (mais tarde Robo3T), que hoje praticamente não é mantido.

O objetivo é manter a filosofia do Robomongo:
- minimalismo em vez de uma interface sobrecarregada
- início rápido e baixo uso de recursos
- sem limitações intrusivas

Oxide Mongo é uma ferramenta aberta e gratuita para desenvolvedores e administradores que precisam de acesso rápido e claro ao MongoDB sem complexidade extra.
"#,
                },
            ),
            (
                "quick-start",
                DocSection {
                    title: "Início rápido",
                    markdown: r#"# Início rápido

## Primeiro início

Abaixo está um cenário passo a passo para o primeiro uso, quando você já tem um banco de dados e precisa buscar e editar um documento.

- Abra o menu "Conexões" e clique em "Criar".
- Preencha os parâmetros de conexão: endereço, porta, banco de dados. Ative autenticação e/ou túnel SSH se necessário.
- Clique em "Testar" e certifique-se de que a verificação foi bem-sucedida.
- Clique em "Salvar", depois selecione a conexão criada e clique em "Conectar".
- No painel esquerdo, expanda o banco de dados e a coleção e abra uma aba.
- No editor de consultas, insira uma busca, por exemplo `db.getCollection('my_collection').find({})`, e clique em "Enviar".
- Na tabela de resultados, selecione um documento e abra o menu de contexto "Editar documento...".
- Modifique o documento e clique em "Salvar".
- Se necessário, execute `find(...)` novamente para verificar se os dados foram atualizados.
"#,
                },
            ),
            (
                "supported-commands",
                DocSection {
                    title: "Comandos suportados",
                    markdown: r#"# Lista de comandos suportados:

## Para coleções:

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

Para find(...), os seguintes métodos são suportados:

    sort(...), hint(...), limit(...), skip(...), maxTimeMS(...), explain(), count(...), countDocuments(...), comment(...)

## Para bancos de dados

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
                    title: "Fluxo de alterações",
                    markdown: r#"# Fluxo de alterações

## Como funciona

O comando `watch(...)` inicia um fluxo de alterações. A consulta não retorna todos os documentos de uma vez. Em vez disso, ela aguarda novos eventos e os adiciona à tabela conforme chegam.

## Quando termina

O fluxo é encerrado automaticamente quando o número de elementos recebidos atinge o valor `limit`. Depois disso, a consulta é considerada concluída e o tempo de execução é mostrado.
"#,
                },
            ),
            (
                "hotkeys",
                DocSection {
                    title: "Teclas de atalho",
                    markdown: r#"# Teclas de atalho

- F2 — alternar resultados para a visão de Tabela
- F4 — alternar resultados para a visão de Texto
- Ctrl+Enter — executar a consulta atual
- Ctrl+W — fechar a aba ativa
"#,
                },
            ),
        ])
    })
}
