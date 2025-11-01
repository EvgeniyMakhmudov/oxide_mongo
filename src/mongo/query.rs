use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64_STANDARD};
use chrono::{Duration as ChronoDuration, TimeZone, Utc};
use mongodb::bson::spec::BinarySubtype;
use mongodb::bson::{
    self, Binary, Bson, DateTime, Decimal128, Document, JavaScriptCodeWithScope, Regex,
    Timestamp as BsonTimestamp, doc, oid::ObjectId,
};
use mongodb::options::{Acknowledgment, Collation, Hint, ReturnDocument, WriteConcern};
use mongodb::sync::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::i18n::{tr, tr_format};
use crate::mongo::shell_preprocessor::quote_unquoted_keys;

#[derive(Debug, Clone, Default)]
pub struct CountDocumentsParsedOptions {
    limit: Option<u64>,
    skip: Option<u64>,
    hint: Option<Hint>,
    max_time: Option<Duration>,
}

impl CountDocumentsParsedOptions {
    fn has_values(&self) -> bool {
        self.limit.is_some()
            || self.skip.is_some()
            || self.hint.is_some()
            || self.max_time.is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub struct EstimatedDocumentCountParsedOptions {
    max_time: Option<Duration>,
}

impl EstimatedDocumentCountParsedOptions {
    fn has_values(&self) -> bool {
        self.max_time.is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub struct InsertOneParsedOptions {
    write_concern: Option<WriteConcern>,
}

impl InsertOneParsedOptions {
    fn has_values(&self) -> bool {
        self.write_concern.is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub struct InsertManyParsedOptions {
    write_concern: Option<WriteConcern>,
    ordered: Option<bool>,
}

impl InsertManyParsedOptions {
    fn has_values(&self) -> bool {
        self.write_concern.is_some() || self.ordered.is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub struct DeleteParsedOptions {
    write_concern: Option<WriteConcern>,
    collation: Option<Collation>,
    hint: Option<Hint>,
}

impl DeleteParsedOptions {
    fn has_values(&self) -> bool {
        self.write_concern.is_some() || self.collation.is_some() || self.hint.is_some()
    }
}

#[derive(Debug, Clone)]
pub enum UpdateModificationsSpec {
    Document(Document),
    Pipeline(Vec<Document>),
}

#[derive(Debug, Clone, Default)]
pub struct UpdateParsedOptions {
    write_concern: Option<WriteConcern>,
    upsert: Option<bool>,
    array_filters: Option<Vec<Document>>,
    collation: Option<Collation>,
    hint: Option<Hint>,
    bypass_document_validation: Option<bool>,
    let_vars: Option<Document>,
    comment: Option<Bson>,
    sort: Option<Document>,
}

impl UpdateParsedOptions {
    fn has_values(&self) -> bool {
        self.write_concern.is_some()
            || self.upsert.is_some()
            || self.array_filters.is_some()
            || self.collation.is_some()
            || self.hint.is_some()
            || self.bypass_document_validation.is_some()
            || self.let_vars.is_some()
            || self.comment.is_some()
            || self.sort.is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReplaceParsedOptions {
    write_concern: Option<WriteConcern>,
    upsert: Option<bool>,
    collation: Option<Collation>,
    hint: Option<Hint>,
    bypass_document_validation: Option<bool>,
    let_vars: Option<Document>,
    comment: Option<Bson>,
    sort: Option<Document>,
}

impl ReplaceParsedOptions {
    fn has_values(&self) -> bool {
        self.write_concern.is_some()
            || self.upsert.is_some()
            || self.collation.is_some()
            || self.hint.is_some()
            || self.bypass_document_validation.is_some()
            || self.let_vars.is_some()
            || self.comment.is_some()
            || self.sort.is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub struct FindOneAndUpdateParsedOptions {
    write_concern: Option<WriteConcern>,
    upsert: Option<bool>,
    array_filters: Option<Vec<Document>>,
    bypass_document_validation: Option<bool>,
    max_time: Option<Duration>,
    projection: Option<Document>,
    return_document: Option<ReturnDocument>,
    sort: Option<Document>,
    collation: Option<Collation>,
    hint: Option<Hint>,
    let_vars: Option<Document>,
    comment: Option<Bson>,
}

impl FindOneAndUpdateParsedOptions {
    fn has_values(&self) -> bool {
        self.write_concern.is_some()
            || self.upsert.is_some()
            || self.array_filters.is_some()
            || self.bypass_document_validation.is_some()
            || self.max_time.is_some()
            || self.projection.is_some()
            || self.return_document.is_some()
            || self.sort.is_some()
            || self.collation.is_some()
            || self.hint.is_some()
            || self.let_vars.is_some()
            || self.comment.is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub struct FindOneAndReplaceParsedOptions {
    write_concern: Option<WriteConcern>,
    upsert: Option<bool>,
    bypass_document_validation: Option<bool>,
    max_time: Option<Duration>,
    projection: Option<Document>,
    return_document: Option<ReturnDocument>,
    sort: Option<Document>,
    collation: Option<Collation>,
    hint: Option<Hint>,
    let_vars: Option<Document>,
    comment: Option<Bson>,
}

impl FindOneAndReplaceParsedOptions {
    fn has_values(&self) -> bool {
        self.write_concern.is_some()
            || self.upsert.is_some()
            || self.bypass_document_validation.is_some()
            || self.max_time.is_some()
            || self.projection.is_some()
            || self.return_document.is_some()
            || self.sort.is_some()
            || self.collation.is_some()
            || self.hint.is_some()
            || self.let_vars.is_some()
            || self.comment.is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub struct FindOneAndDeleteParsedOptions {
    write_concern: Option<WriteConcern>,
    max_time: Option<Duration>,
    projection: Option<Document>,
    sort: Option<Document>,
    collation: Option<Collation>,
    hint: Option<Hint>,
    let_vars: Option<Document>,
    comment: Option<Bson>,
}

impl FindOneAndDeleteParsedOptions {
    fn has_values(&self) -> bool {
        self.write_concern.is_some()
            || self.max_time.is_some()
            || self.projection.is_some()
            || self.sort.is_some()
            || self.collation.is_some()
            || self.hint.is_some()
            || self.let_vars.is_some()
            || self.comment.is_some()
    }
}

#[derive(Debug, Clone)]
pub enum QueryOperation {
    Find {
        filter: Document,
    },
    FindOne {
        filter: Document,
    },
    Count {
        filter: Document,
    },
    CountDocuments {
        filter: Document,
        options: Option<CountDocumentsParsedOptions>,
    },
    EstimatedDocumentCount {
        options: Option<EstimatedDocumentCountParsedOptions>,
    },
    Distinct {
        field: String,
        filter: Document,
    },
    Aggregate {
        pipeline: Vec<Document>,
    },
    InsertOne {
        document: Document,
        options: Option<InsertOneParsedOptions>,
    },
    InsertMany {
        documents: Vec<Document>,
        options: Option<InsertManyParsedOptions>,
    },
    DeleteOne {
        filter: Document,
        options: Option<DeleteParsedOptions>,
    },
    DeleteMany {
        filter: Document,
        options: Option<DeleteParsedOptions>,
    },
    UpdateOne {
        filter: Document,
        update: UpdateModificationsSpec,
        options: Option<UpdateParsedOptions>,
    },
    UpdateMany {
        filter: Document,
        update: UpdateModificationsSpec,
        options: Option<UpdateParsedOptions>,
    },
    ReplaceOne {
        filter: Document,
        replacement: Document,
        options: Option<ReplaceParsedOptions>,
    },
    FindOneAndUpdate {
        filter: Document,
        update: UpdateModificationsSpec,
        options: Option<FindOneAndUpdateParsedOptions>,
    },
    FindOneAndReplace {
        filter: Document,
        replacement: Document,
        options: Option<FindOneAndReplaceParsedOptions>,
    },
    FindOneAndDelete {
        filter: Document,
        options: Option<FindOneAndDeleteParsedOptions>,
    },
    ListIndexes,
    DatabaseCommand {
        db: String,
        command: Document,
    },
}

#[derive(Debug, Clone)]
pub enum QueryResult {
    Documents(Vec<Bson>),
    Indexes(Vec<Bson>),
    SingleDocument { document: Document },
    Distinct { field: String, values: Vec<Bson> },
    Count { value: Bson },
}

struct QueryParser<'a> {
    db_name: &'a str,
    collection: &'a str,
}

impl<'a> QueryParser<'a> {
    fn parse_query(&self, text: &str) -> Result<QueryOperation, String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(String::from(tr(
                "Query must start with db.<collection>, db.getCollection('<collection>'), or a supported database method.",
            )));
        }

        let cleaned = trimmed.trim_end_matches(';').trim();

        if let Some(result) = self.try_parse_database_method(cleaned)? {
            return Ok(result);
        }

        let after_collection = Self::strip_collection_prefix(cleaned)?;

        let (method_name, args, remainder) = Self::extract_primary_method(after_collection)?;
        if !remainder.trim().is_empty() {
            let extra = remainder.trim_start();
            if method_name == "find" && extra.starts_with(".countDocuments(") {
                return Err(String::from(tr(
                    "countDocuments() must be called directly on a collection. Chains like db.collection.find(...).countDocuments(...) are not supported.",
                )));
            }
            return Err(String::from(tr(
                "Only a single method call is supported after specifying the collection.",
            )));
        }

        let args_trimmed = args.trim();
        match method_name.as_str() {
            "createIndex" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };

                if parts.is_empty() || parts.len() > 2 {
                    return Err(String::from(tr(
                        "createIndex expects a key document and an optional options object.",
                    )));
                }

                let keys_bson = Self::parse_shell_bson_value(&parts[0])?;
                let keys_doc = match keys_bson {
                    Bson::Document(doc) => doc,
                    _ => {
                        return Err(String::from(tr(
                            "The first argument to createIndex must be a document with keys.",
                        )));
                    }
                };

                let mut index_spec = Document::new();
                index_spec.insert("key", Bson::Document(keys_doc));

                if let Some(options_source) = parts.get(1) {
                    let options_bson = Self::parse_shell_bson_value(options_source)?;
                    let options_doc = match options_bson {
                        Bson::Document(doc) => doc,
                        _ => {
                            return Err(String::from(tr(
                                "createIndex options must be a JSON object.",
                            )));
                        }
                    };
                    for (key, value) in options_doc {
                        index_spec.insert(key, value);
                    }
                }

                let mut command = Document::new();
                command.insert("createIndexes", Bson::String(self.collection.to_string()));
                command.insert("indexes", Bson::Array(vec![Bson::Document(index_spec)]));

                Ok(QueryOperation::DatabaseCommand { db: self.db_name.to_string(), command })
            }
            "createIndexes" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };

                if parts.is_empty() || parts.len() > 2 {
                    return Err(String::from(tr(
                        "createIndexes expects an array of index definitions and optional options.",
                    )));
                }

                let indexes_bson = Self::parse_shell_bson_value(&parts[0])?;
                let mut index_entries = Vec::new();
                match indexes_bson {
                    Bson::Array(items) => {
                        if items.is_empty() {
                            return Err(String::from(tr(
                                "The index array for createIndexes cannot be empty.",
                            )));
                        }
                        for item in items {
                            match item {
                                Bson::Document(doc) => index_entries.push(Bson::Document(doc)),
                                _ => {
                                    return Err(String::from(tr(
                                        "Each index in createIndexes must be an object.",
                                    )));
                                }
                            }
                        }
                    }
                    Bson::Document(doc) => {
                        index_entries.push(Bson::Document(doc));
                    }
                    _ => {
                        return Err(String::from(tr(
                            "The first argument to createIndexes must be an array or an object.",
                        )));
                    }
                }

                let mut command = Document::new();
                command.insert("createIndexes", Bson::String(self.collection.to_string()));
                command.insert("indexes", Bson::Array(index_entries));

                if let Some(options_source) = parts.get(1) {
                    let options_bson = Self::parse_shell_bson_value(options_source)?;
                    let options_doc = match options_bson {
                        Bson::Document(doc) => doc,
                        _ => {
                            return Err(String::from(tr(
                                "createIndexes options must be a JSON object.",
                            )));
                        }
                    };
                    for (key, value) in options_doc {
                        command.insert(key, value);
                    }
                }

                Ok(QueryOperation::DatabaseCommand { db: self.db_name.to_string(), command })
            }
            "dropIndex" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };
                if parts.is_empty() || parts.len() > 2 {
                    return Err(String::from(tr(
                        "dropIndex expects an index name or key document and optional options.",
                    )));
                }

                let index_value = Self::parse_index_argument(&parts[0])?;

                let mut command = doc! {
                    "dropIndexes": self.collection.to_string(),
                    "index": index_value,
                };

                if let Some(options_source) = parts.get(1) {
                    let options_bson = Self::parse_shell_bson_value(options_source)?;
                    let options_doc = match options_bson {
                        Bson::Document(doc) => doc,
                        _ => {
                            return Err(String::from(tr(
                                "dropIndex options must be a JSON object.",
                            )));
                        }
                    };
                    for (key, value) in options_doc {
                        command.insert(key, value);
                    }
                }

                Ok(QueryOperation::DatabaseCommand { db: self.db_name.to_string(), command })
            }
            "dropIndexes" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };

                if parts.len() > 2 {
                    return Err(String::from(tr(
                        "dropIndexes supports at most two arguments: index and options.",
                    )));
                }

                let index_value = if let Some(first) = parts.get(0) {
                    if first.trim().is_empty() {
                        Bson::String("*".into())
                    } else {
                        Self::parse_index_argument(first)?
                    }
                } else {
                    Bson::String("*".into())
                };

                let mut command = doc! {
                    "dropIndexes": self.collection.to_string(),
                    "index": index_value,
                };

                if let Some(options_source) = parts.get(1) {
                    let options_bson = Self::parse_shell_bson_value(options_source)?;
                    let options_doc = match options_bson {
                        Bson::Document(doc) => doc,
                        _ => {
                            return Err(String::from(tr(
                                "dropIndexes options must be a JSON object.",
                            )));
                        }
                    };
                    for (key, value) in options_doc {
                        command.insert(key, value);
                    }
                }

                Ok(QueryOperation::DatabaseCommand { db: self.db_name.to_string(), command })
            }
            "getIndexes" => {
                if !args_trimmed.is_empty() {
                    return Err(String::from(tr("getIndexes does not take any arguments.")));
                }

                Ok(QueryOperation::ListIndexes)
            }
            "hideIndex" | "unhideIndex" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };

                if parts.len() != 1 {
                    return Err(String::from(tr(
                        "hideIndex/unhideIndex expect a single argument with the index name or keys.",
                    )));
                }

                let index_value = Self::parse_index_argument(&parts[0])?;

                let command_name =
                    if method_name == "hideIndex" { "hideIndex" } else { "unhideIndex" };

                let mut command = Document::new();
                command.insert(command_name, Bson::String(self.collection.to_string()));
                command.insert("index", index_value);

                Ok(QueryOperation::DatabaseCommand { db: self.db_name.to_string(), command })
            }
            "countDocuments" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };
                if parts.len() > 2 {
                    return Err(String::from(tr(
                        "countDocuments supports at most two arguments: query and options.",
                    )));
                }

                let filter = if let Some(first) = parts.get(0) {
                    if first.is_empty() { Document::new() } else { Self::parse_json_object(first)? }
                } else {
                    Document::new()
                };

                let options = if let Some(second) = parts.get(1) {
                    Self::parse_count_documents_options(second)?
                } else {
                    None
                };

                Ok(QueryOperation::CountDocuments { filter, options })
            }
            "estimatedDocumentCount" => {
                let options = if args_trimmed.is_empty() {
                    None
                } else {
                    let parts = Self::split_arguments(args_trimmed);
                    if parts.len() > 1 {
                        return Err(String::from(tr(
                            "estimatedDocumentCount accepts only the options argument.",
                        )));
                    }

                    match parts.get(0) {
                        Some(source) if source.trim().is_empty() => None,
                        Some(source) => Self::parse_estimated_count_options(source)?,
                        None => None,
                    }
                };

                Ok(QueryOperation::EstimatedDocumentCount { options })
            }
            "findOne" => {
                let filter = if args_trimmed.is_empty() {
                    Document::new()
                } else {
                    Self::parse_json_object(args_trimmed)?
                };
                Ok(QueryOperation::FindOne { filter })
            }
            "count" => {
                let filter = if args_trimmed.is_empty() {
                    Document::new()
                } else {
                    Self::parse_json_object(args_trimmed)?
                };
                Ok(QueryOperation::Count { filter })
            }
            "distinct" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };
                if parts.is_empty() {
                    return Err(String::from(tr("distinct requires at least the field name.")));
                }

                let field_value: Value = Self::parse_shell_json_value(&parts[0])?;
                let field = match field_value {
                    Value::String(s) => s,
                    _ => {
                        return Err(String::from(tr(
                            "The first argument to distinct must be a string.",
                        )));
                    }
                };

                let filter = if parts.len() > 1 {
                    let filter_value: Value = Self::parse_shell_json_value(&parts[1])?;
                    if !filter_value.is_object() {
                        return Err(String::from(tr("The distinct filter must be a JSON object.")));
                    }
                    bson::to_document(&filter_value)
                        .map_err(|error| format!("BSON conversion error: {error}"))?
                } else {
                    Document::new()
                };

                Ok(QueryOperation::Distinct { field, filter })
            }
            "aggregate" => {
                if args_trimmed.is_empty() {
                    return Err(String::from(tr(
                        "aggregate requires an array of stages as its argument.",
                    )));
                }

                let value: Value = Self::parse_shell_json_value(args_trimmed)?;
                let array = value
                    .as_array()
                    .ok_or_else(|| String::from(tr("The aggregate argument must be an array.")))?;
                let mut pipeline = Vec::new();
                for item in array {
                    let doc = item
                        .as_object()
                        .ok_or_else(|| String::from(tr("Pipeline elements must be objects.")))?;
                    pipeline.push(
                        bson::to_document(doc)
                            .map_err(|error| format!("BSON conversion error: {error}"))?,
                    );
                }
                Ok(QueryOperation::Aggregate { pipeline })
            }
            "insertOne" => {
                if args_trimmed.is_empty() {
                    return Err(String::from(tr(
                        "insertOne requires a document as the first argument.",
                    )));
                }

                let parts = Self::split_arguments(args_trimmed);
                if parts.is_empty() || parts.len() > 2 {
                    return Err(String::from(tr(
                        "insertOne accepts a single document and an optional options object.",
                    )));
                }

                let document = Self::parse_json_object(&parts[0])?;
                let options = if let Some(second) = parts.get(1) {
                    Self::parse_insert_one_options(second)?
                } else {
                    None
                };

                Ok(QueryOperation::InsertOne { document, options })
            }
            "insertMany" => {
                if args_trimmed.is_empty() {
                    return Err(String::from(tr(
                        "insertMany requires an array of documents as the first argument.",
                    )));
                }

                let parts = Self::split_arguments(args_trimmed);
                if parts.is_empty() || parts.len() > 2 {
                    return Err(String::from(tr(
                        "insertMany accepts an array of documents and an optional options object.",
                    )));
                }

                let docs_value: Value = Self::parse_shell_json_value(&parts[0])?;
                let docs_array = docs_value.as_array().ok_or_else(|| {
                    String::from(tr(
                        "The first argument to insertMany must be an array of documents.",
                    ))
                })?;
                if docs_array.is_empty() {
                    return Err(String::from(tr(
                        "insertMany requires at least one document in the array.",
                    )));
                }

                let mut documents = Vec::with_capacity(docs_array.len());
                for (index, entry) in docs_array.iter().enumerate() {
                    let object = entry.as_object().ok_or_else(|| {
                        format!(
                            "{} {} {}",
                            tr("Element at index"),
                            index,
                            tr("in insertMany must be a JSON object."),
                        )
                    })?;
                    let doc = bson::to_document(object)
                        .map_err(|error| format!("BSON conversion error: {error}"))?;
                    documents.push(doc);
                }

                let options = if let Some(second) = parts.get(1) {
                    Self::parse_insert_many_options(second)?
                } else {
                    None
                };

                Ok(QueryOperation::InsertMany { documents, options })
            }
            "updateOne" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };
                if parts.len() < 2 || parts.len() > 3 {
                    return Err(String::from(tr(
                        "updateOne expects a filter, an update, and an optional options object.",
                    )));
                }

                let filter = Self::parse_json_object(&parts[0])?;
                let update = Self::parse_update_spec(&parts[1])?;
                let options = if let Some(third) = parts.get(2) {
                    Self::parse_update_options(third)?
                } else {
                    None
                };

                Ok(QueryOperation::UpdateOne { filter, update, options })
            }
            "updateMany" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };
                if parts.len() < 2 || parts.len() > 3 {
                    return Err(String::from(tr(
                        "updateMany expects a filter, an update, and an optional options object.",
                    )));
                }

                let filter = Self::parse_json_object(&parts[0])?;
                let update = Self::parse_update_spec(&parts[1])?;
                let options = if let Some(third) = parts.get(2) {
                    Self::parse_update_options(third)?
                } else {
                    None
                };

                Ok(QueryOperation::UpdateMany { filter, update, options })
            }
            "replaceOne" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };
                if parts.len() < 2 || parts.len() > 3 {
                    return Err(String::from(tr(
                        "replaceOne expects a filter, a replacement document, and an optional options object.",
                    )));
                }

                let filter = Self::parse_json_object(&parts[0])?;
                let replacement = Self::parse_json_object(&parts[1])?;
                let options = if let Some(third) = parts.get(2) {
                    Self::parse_replace_options(third)?
                } else {
                    None
                };

                Ok(QueryOperation::ReplaceOne { filter, replacement, options })
            }
            "findOneAndUpdate" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };
                if parts.len() < 2 || parts.len() > 3 {
                    return Err(String::from(tr(
                        "findOneAndUpdate expects a filter, an update, and an optional options object.",
                    )));
                }

                let filter = Self::parse_json_object(&parts[0])?;
                let update = Self::parse_update_spec(&parts[1])?;
                let options = if let Some(third) = parts.get(2) {
                    Self::parse_find_one_and_update_options(third)?
                } else {
                    None
                };

                Ok(QueryOperation::FindOneAndUpdate { filter, update, options })
            }
            "findOneAndReplace" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };
                if parts.len() < 2 || parts.len() > 3 {
                    return Err(String::from(tr(
                        "findOneAndReplace expects a filter, a replacement document, and an optional options object.",
                    )));
                }

                let filter = Self::parse_json_object(&parts[0])?;
                let replacement = Self::parse_json_object(&parts[1])?;
                let options = if let Some(third) = parts.get(2) {
                    Self::parse_find_one_and_replace_options(third)?
                } else {
                    None
                };

                Ok(QueryOperation::FindOneAndReplace { filter, replacement, options })
            }
            "findOneAndDelete" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };
                if parts.is_empty() || parts.len() > 2 {
                    return Err(String::from(tr(
                        "findOneAndDelete expects a filter and an optional options object.",
                    )));
                }

                let filter = Self::parse_json_object(&parts[0])?;
                let options = if let Some(second) = parts.get(1) {
                    Self::parse_find_one_and_delete_options(second)?
                } else {
                    None
                };

                Ok(QueryOperation::FindOneAndDelete { filter, options })
            }
            "findOneAndModify" => self.parse_find_one_and_modify(args_trimmed),
            "deleteOne" => {
                if args_trimmed.is_empty() {
                    return Err(String::from(tr(
                        "deleteOne requires a filter as the first argument.",
                    )));
                }

                let parts = Self::split_arguments(args_trimmed);
                if parts.is_empty() || parts.len() > 2 {
                    return Err(String::from(tr(
                        "deleteOne accepts a filter and an optional options object.",
                    )));
                }

                let filter = Self::parse_json_object(&parts[0])?;
                let options = if let Some(second) = parts.get(1) {
                    Self::parse_delete_options(second)?
                } else {
                    None
                };

                Ok(QueryOperation::DeleteOne { filter, options })
            }
            "deleteMany" => {
                if args_trimmed.is_empty() {
                    return Err(String::from(tr(
                        "deleteMany requires a filter as the first argument.",
                    )));
                }

                let parts = Self::split_arguments(args_trimmed);
                if parts.is_empty() || parts.len() > 2 {
                    return Err(String::from(tr(
                        "deleteMany accepts a filter and an optional options object.",
                    )));
                }

                let filter = Self::parse_json_object(&parts[0])?;
                let options = if let Some(second) = parts.get(1) {
                    Self::parse_delete_options(second)?
                } else {
                    None
                };

                Ok(QueryOperation::DeleteMany { filter, options })
            }
            "find" => {
                if args_trimmed.is_empty() {
                    return Ok(QueryOperation::Find { filter: Document::new() });
                }
                let filter = Self::parse_json_object(args_trimmed)?;
                Ok(QueryOperation::Find { filter })
            }
            other => Err(tr_format(
                "Method {} is not supported. Available methods: find, findOne, count, countDocuments, estimatedDocumentCount, distinct, aggregate, insertOne, insertMany, updateOne, updateMany, replaceOne, findOneAndUpdate, findOneAndReplace, findOneAndDelete, deleteOne, deleteMany, createIndex, createIndexes, dropIndex, dropIndexes, getIndexes, hideIndex, unhideIndex.",
                &[other],
            )),
        }
    }

    fn strip_collection_prefix(text: &str) -> Result<&str, String> {
        if let Some(rest) = text.strip_prefix("db.getCollection(") {
            let rest = rest.trim_start();
            let (_, after_literal) = Self::parse_collection_literal(rest)?;
            let after_literal = after_literal.trim_start();
            let after_paren = after_literal.strip_prefix(')').ok_or_else(|| {
                String::from(tr("Expected ')' after collection name in getCollection."))
            })?;
            let after_paren = after_paren.trim_start();
            if !after_paren.starts_with('.') {
                return Err(String::from(tr(
                    "Expected a method call after specifying the collection.",
                )));
            }
            Ok(after_paren)
        } else if let Some(rest) = text.strip_prefix("db.") {
            if rest.is_empty() {
                return Err(String::from(tr("Expected collection name after db.")));
            }

            let bytes = rest.as_bytes();
            let mut index = 0usize;
            while index < bytes.len() {
                let byte = bytes[index];
                if (byte as char).is_ascii_alphanumeric() || byte == b'_' {
                    index += 1;
                    continue;
                }

                if byte == b'.' {
                    if index == 0 {
                        return Err(String::from(tr("Expected collection name after db.")));
                    }
                    return Ok(&rest[index..]);
                }

                return Err(format!(
                    "{} '{}'",
                    tr("Invalid character in the collection name:"),
                    byte as char
                ));
            }

            Err(String::from(tr("Expected a method call after specifying the collection.")))
        } else {
            Err(String::from(tr(
                "Query must start with db.<collection>, db.getCollection('<collection>'), or a supported method.",
            )))
        }
    }

    fn parse_collection_literal(text: &str) -> Result<(&str, &str), String> {
        if text.trim().is_empty() {
            return Err(String::from(tr("Collection name in getCollection is not provided.")));
        }

        let trimmed = text.trim_start();
        if trimmed.is_empty() {
            return Err(String::from(tr("Collection name in getCollection is not provided.")));
        }

        let bytes = trimmed.as_bytes();
        let quote = bytes[0];
        if quote != b'\'' && quote != b'"' {
            return Err(String::from(tr(
                "Collection name in getCollection must be a quoted string.",
            )));
        }

        let mut index = 1usize;
        while index < bytes.len() {
            match bytes[index] {
                b'\\' => index += 2,
                ch if ch == quote => {
                    let name = &trimmed[1..index];
                    let rest = &trimmed[index + 1..];
                    return Ok((name, rest));
                }
                _ => index += 1,
            }
        }

        Err(String::from(tr("Collection string in getCollection is not closed.")))
    }

    fn extract_primary_method(text: &str) -> Result<(String, String, &str), String> {
        if !text.starts_with('.') {
            return Err(String::from(tr(
                "Expected a method call after specifying the collection.",
            )));
        }

        let rest = &text[1..];
        if rest.is_empty() {
            return Err(String::from(tr("Expected method name after the dot.")));
        }

        let bytes = rest.as_bytes();
        let mut index = 0usize;
        while index < bytes.len() {
            let byte = bytes[index];
            if (byte as char).is_ascii_alphanumeric() || byte == b'_' {
                index += 1;
                continue;
            }

            if byte == b'(' {
                if index == 0 {
                    return Err(String::from(tr("Expected method name after the dot.")));
                }

                let method_name = &rest[..index];
                let mut depth = 0i32;
                let mut cursor = index + 1;
                while cursor < bytes.len() {
                    match bytes[cursor] {
                        b'(' => depth += 1,
                        b')' => {
                            if depth == 0 {
                                let args = &rest[index + 1..cursor];
                                let remainder = &rest[cursor + 1..];
                                return Ok((method_name.to_string(), args.to_string(), remainder));
                            }
                            depth -= 1;
                        }
                        _ => {}
                    }
                    cursor += 1;
                }

                return Err(String::from(tr("Method call parenthesis is not closed.")));
            }

            if byte == b'.' {
                return Err(String::from(tr(
                    "Only one method call is supported after specifying the collection.",
                )));
            }

            let character = (byte as char).to_string();
            return Err(tr_format("Invalid character '{}' in the method name.", &[&character]));
        }

        Err(String::from(tr("Expected '(' after the method name.")))
    }

    fn parse_count_documents_options(
        source: &str,
    ) -> Result<Option<CountDocumentsParsedOptions>, String> {
        let value: Value = Self::parse_shell_json_value(source)?;
        let object = value
            .as_object()
            .ok_or_else(|| String::from(tr("countDocuments options must be a JSON object.")))?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = CountDocumentsParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "limit" => {
                    let limit = Self::parse_non_negative_u64(value, "limit")?;
                    options.limit = Some(limit);
                }
                "skip" => {
                    let skip = Self::parse_non_negative_u64(value, "skip")?;
                    options.skip = Some(skip);
                }
                "maxTimeMS" => {
                    let millis = Self::parse_non_negative_u64(value, "maxTimeMS")?;
                    options.max_time = Some(Duration::from_millis(millis));
                }
                "hint" => {
                    let hint = match value {
                        Value::String(name) => Hint::Name(name.clone()),
                        Value::Object(map) => {
                            let doc = bson::to_document(map)
                                .map_err(|error| format!("BSON conversion error: {error}"))?;
                            Hint::Keys(doc)
                        }
                        _ => {
                            return Err(String::from(tr(
                                "Parameter 'hint' must be a string or a JSON object.",
                            )));
                        }
                    };
                    options.hint = Some(hint);
                }
                other => {
                    return Err(tr_format(
                        "Parameter '{}' is not supported in countDocuments options. Allowed: limit, skip, hint, maxTimeMS.",
                        &[other],
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn parse_estimated_count_options(
        source: &str,
    ) -> Result<Option<EstimatedDocumentCountParsedOptions>, String> {
        let value: Value = Self::parse_shell_json_value(source)?;
        let object = value.as_object().ok_or_else(|| {
            String::from(tr("estimatedDocumentCount options must be a JSON object."))
        })?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = EstimatedDocumentCountParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "maxTimeMS" => {
                    let millis = Self::parse_non_negative_u64(value, "maxTimeMS")?;
                    options.max_time = Some(Duration::from_millis(millis));
                }
                other => {
                    return Err(tr_format(
                        "Parameter '{}' is not supported in estimatedDocumentCount options. Only maxTimeMS is allowed.",
                        &[other],
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn try_parse_database_method(&self, cleaned: &str) -> Result<Option<QueryOperation>, String> {
        if let Some(rest) = cleaned.strip_prefix("db.") {
            let rest = rest.trim();
            if rest.starts_with("getCollection(") {
                return Ok(None);
            }

            if let Some(paren_pos) = rest.find('(') {
                let dot_pos = rest.find('.');
                if dot_pos.is_none() || paren_pos < dot_pos.unwrap() {
                    let synthetic = format!(".{rest}");
                    let (method_name, args, remainder) = Self::extract_primary_method(&synthetic)?;
                    if !remainder.trim().is_empty() {
                        return Err(String::from(tr(
                            "Only one method call is supported after specifying the database.",
                        )));
                    }
                    return self.parse_database_method(&method_name, &args).map(Some);
                }
            }
        }

        Ok(None)
    }

    fn parse_database_method(&self, method: &str, args: &str) -> Result<QueryOperation, String> {
        let args_trimmed = args.trim();

        match method {
            "stats" => {
                let mut command = doc! { "dbStats": 1 };

                if !args_trimmed.is_empty() {
                    if args_trimmed.starts_with('{') {
                        let extras = Self::parse_json_object(args_trimmed)?;
                        for (key, value) in extras {
                            command.insert(key, value);
                        }
                    } else {
                        let value: Value = Self::parse_shell_json_value(args_trimmed)?;

                        if let Some(number) = value.as_f64() {
                            command.insert("scale", Bson::Double(number));
                        } else if let Some(number) = value.as_i64() {
                            command.insert("scale", Bson::Int64(number));
                        } else if let Some(number) = value.as_u64() {
                            if number <= i64::MAX as u64 {
                                command.insert("scale", Bson::Int64(number as i64));
                            } else {
                                command.insert("scale", Bson::String(number.to_string()));
                            }
                        } else {
                            return Err(String::from(tr(
                                "db.stats expects a number or an options object.",
                            )));
                        }
                    }
                }

                Ok(QueryOperation::DatabaseCommand { db: self.db_name.to_string(), command })
            }
            "runCommand" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };

                if parts.is_empty() {
                    return Err(String::from(tr(
                        "db.runCommand expects a document describing the command.",
                    )));
                }
                if parts.len() > 1 {
                    return Err(String::from(tr(
                        "db.runCommand supports only one argument (the command document).",
                    )));
                }

                let command_bson = Self::parse_shell_bson_value(&parts[0])?;
                let command = match command_bson {
                    Bson::Document(doc) => doc,
                    _ => {
                        return Err(String::from(tr(
                            "The first argument to db.runCommand must be a document.",
                        )));
                    }
                };

                Ok(QueryOperation::DatabaseCommand { db: self.db_name.to_string(), command })
            }
            other => Err(tr_format(
                "Method db.{} is not supported. Available methods: stats, runCommand.",
                &[other],
            )),
        }
    }

    fn parse_insert_one_options(source: &str) -> Result<Option<InsertOneParsedOptions>, String> {
        let value: Value = Self::parse_shell_json_value(source)?;
        let object = value
            .as_object()
            .ok_or_else(|| String::from(tr("insertOne options must be a JSON object.")))?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = InsertOneParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "writeConcern" => {
                    options.write_concern = Self::parse_write_concern_value(value)?;
                }
                other => {
                    return Err(tr_format(
                        "Parameter '{}' is not supported in insertOne options. Allowed: writeConcern.",
                        &[other],
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn parse_insert_many_options(source: &str) -> Result<Option<InsertManyParsedOptions>, String> {
        let value: Value = Self::parse_shell_json_value(source)?;
        let object = value
            .as_object()
            .ok_or_else(|| String::from(tr("insertMany options must be a JSON object.")))?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = InsertManyParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "writeConcern" => {
                    options.write_concern = Self::parse_write_concern_value(value)?;
                }
                "ordered" => {
                    let ordered = value.as_bool().ok_or_else(|| {
                        String::from(tr(
                            "Parameter 'ordered' in insertMany options must be a boolean.",
                        ))
                    })?;
                    options.ordered = Some(ordered);
                }
                other => {
                    return Err(tr_format(
                        "Parameter '{}' is not supported in insertMany options. Allowed: writeConcern, ordered.",
                        &[other],
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn parse_delete_options(source: &str) -> Result<Option<DeleteParsedOptions>, String> {
        let value: Value = Self::parse_shell_json_value(source)?;
        let object = value.as_object().ok_or_else(|| {
            String::from(tr("deleteOne/deleteMany options must be a JSON object."))
        })?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = DeleteParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "writeConcern" => {
                    options.write_concern = Self::parse_write_concern_value(value)?;
                }
                "collation" => {
                    options.collation = Some(Self::parse_collation_value(value)?);
                }
                "hint" => {
                    options.hint = Some(Self::parse_hint_value(value)?);
                }
                other => {
                    return Err(tr_format(
                        "Parameter '{}' is not supported in deleteOne/deleteMany options. Allowed: writeConcern, collation, hint.",
                        &[other],
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn parse_update_spec(source: &str) -> Result<UpdateModificationsSpec, String> {
        let value: Value = Self::parse_shell_json_value(source)?;

        if let Some(object) = value.as_object() {
            let document = bson::to_document(object)
                .map_err(|error| format!("BSON conversion error: {error}"))?;
            Ok(UpdateModificationsSpec::Document(document))
        } else if let Some(array) = value.as_array() {
            let mut pipeline = Vec::with_capacity(array.len());
            for (index, entry) in array.iter().enumerate() {
                let object = entry.as_object().ok_or_else(|| {
                    tr_format(
                        "Pipeline element at index {} must be a JSON object.",
                        &[&index.to_string()],
                    )
                })?;
                let document = bson::to_document(object)
                    .map_err(|error| format!("BSON conversion error: {error}"))?;
                pipeline.push(document);
            }
            if pipeline.is_empty() {
                return Err(String::from(tr(
                    "An empty update array is not supported. Add at least one stage.",
                )));
            }
            Ok(UpdateModificationsSpec::Pipeline(pipeline))
        } else {
            Err(String::from(tr(
                "Update argument must be an object with operators or an array of stages.",
            )))
        }
    }

    fn parse_update_options(source: &str) -> Result<Option<UpdateParsedOptions>, String> {
        let value: Value = Self::parse_shell_json_value(source)?;
        let object = value
            .as_object()
            .ok_or_else(|| String::from(tr("update options must be a JSON object.")))?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = UpdateParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "writeConcern" => options.write_concern = Self::parse_write_concern_value(value)?,
                "upsert" => {
                    options.upsert = Some(Self::parse_bool_field(value, "upsert")?);
                }
                "arrayFilters" => {
                    options.array_filters = Some(Self::parse_array_filters(value)?);
                }
                "collation" => options.collation = Some(Self::parse_collation_value(value)?),
                "hint" => options.hint = Some(Self::parse_hint_value(value)?),
                "bypassDocumentValidation" => {
                    options.bypass_document_validation =
                        Some(Self::parse_bool_field(value, "bypassDocumentValidation")?);
                }
                "let" => {
                    options.let_vars = Some(Self::parse_document_field(value, "let")?);
                }
                "comment" => {
                    options.comment = Some(Self::parse_bson_value(value)?);
                }
                "sort" => {
                    options.sort = Some(Self::parse_document_field(value, "sort")?);
                }
                other => {
                    return Err(tr_format(
                        "Parameter '{}' is not supported in updateOne/updateMany options. Allowed: writeConcern, upsert, arrayFilters, collation, hint, bypassDocumentValidation, let, comment, sort.",
                        &[other],
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn parse_replace_options(source: &str) -> Result<Option<ReplaceParsedOptions>, String> {
        let value: Value = Self::parse_shell_json_value(source)?;
        let object = value
            .as_object()
            .ok_or_else(|| String::from(tr("replace options must be a JSON object.")))?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = ReplaceParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "writeConcern" => options.write_concern = Self::parse_write_concern_value(value)?,
                "upsert" => options.upsert = Some(Self::parse_bool_field(value, "upsert")?),
                "collation" => options.collation = Some(Self::parse_collation_value(value)?),
                "hint" => options.hint = Some(Self::parse_hint_value(value)?),
                "bypassDocumentValidation" => {
                    options.bypass_document_validation =
                        Some(Self::parse_bool_field(value, "bypassDocumentValidation")?);
                }
                "let" => options.let_vars = Some(Self::parse_document_field(value, "let")?),
                "comment" => options.comment = Some(Self::parse_bson_value(value)?),
                "sort" => options.sort = Some(Self::parse_document_field(value, "sort")?),
                other => {
                    return Err(tr_format(
                        "Parameter '{}' is not supported in replaceOne options. Allowed: writeConcern, upsert, collation, hint, bypassDocumentValidation, let, comment, sort.",
                        &[other],
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn parse_bool_field(value: &Value, field: &str) -> Result<bool, String> {
        value.as_bool().ok_or_else(|| {
            tr_format("Parameter '{}' must be a boolean value (true/false).", &[field])
        })
    }

    fn parse_document_field(value: &Value, field: &str) -> Result<Document, String> {
        let object = value
            .as_object()
            .ok_or_else(|| tr_format("Parameter '{}' must be a JSON object.", &[field]))?;
        bson::to_document(object).map_err(|error| format!("BSON conversion error: {error}"))
    }

    fn parse_array_filters(value: &Value) -> Result<Vec<Document>, String> {
        let array = value
            .as_array()
            .ok_or_else(|| String::from(tr("arrayFilters must be an array of objects.")))?;
        if array.is_empty() {
            return Err(String::from(tr("arrayFilters must contain at least one filter object.")));
        }

        let mut filters = Vec::with_capacity(array.len());
        for (index, entry) in array.iter().enumerate() {
            let object = entry.as_object().ok_or_else(|| {
                tr_format(
                    "arrayFilters element at index {} must be a JSON object.",
                    &[&index.to_string()],
                )
            })?;
            let filter = bson::to_document(object)
                .map_err(|error| format!("BSON conversion error: {error}"))?;
            filters.push(filter);
        }

        Ok(filters)
    }

    fn parse_bson_value(value: &Value) -> Result<Bson, String> {
        bson::to_bson(value).map_err(|error| format!("BSON conversion error: {error}"))
    }

    fn parse_find_one_and_update_options(
        source: &str,
    ) -> Result<Option<FindOneAndUpdateParsedOptions>, String> {
        let value: Value = Self::parse_shell_json_value(source)?;
        let object = value
            .as_object()
            .ok_or_else(|| String::from(tr("findOneAndUpdate options must be a JSON object.")))?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = FindOneAndUpdateParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "writeConcern" => options.write_concern = Self::parse_write_concern_value(value)?,
                "upsert" => options.upsert = Some(Self::parse_bool_field(value, "upsert")?),
                "arrayFilters" => options.array_filters = Some(Self::parse_array_filters(value)?),
                "bypassDocumentValidation" => {
                    options.bypass_document_validation =
                        Some(Self::parse_bool_field(value, "bypassDocumentValidation")?);
                }
                "maxTimeMS" => {
                    let millis = Self::parse_non_negative_u64(value, "maxTimeMS")?;
                    options.max_time = Some(Duration::from_millis(millis));
                }
                "projection" => {
                    options.projection = Some(Self::parse_document_field(value, "projection")?)
                }
                "returnDocument" => {
                    options.return_document = Some(Self::parse_return_document(value)?);
                }
                "sort" => options.sort = Some(Self::parse_document_field(value, "sort")?),
                "collation" => options.collation = Some(Self::parse_collation_value(value)?),
                "hint" => options.hint = Some(Self::parse_hint_value(value)?),
                "let" => options.let_vars = Some(Self::parse_document_field(value, "let")?),
                "comment" => options.comment = Some(Self::parse_bson_value(value)?),
                other => {
                    return Err(tr_format(
                        "Parameter '{}' is not supported in findOneAndUpdate options. Allowed: writeConcern, upsert, arrayFilters, bypassDocumentValidation, maxTimeMS, projection, returnDocument, sort, collation, hint, let, comment.",
                        &[other],
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn parse_find_one_and_replace_options(
        source: &str,
    ) -> Result<Option<FindOneAndReplaceParsedOptions>, String> {
        let value: Value = Self::parse_shell_json_value(source)?;
        let object = value
            .as_object()
            .ok_or_else(|| String::from(tr("findOneAndReplace options must be a JSON object.")))?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = FindOneAndReplaceParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "writeConcern" => options.write_concern = Self::parse_write_concern_value(value)?,
                "upsert" => options.upsert = Some(Self::parse_bool_field(value, "upsert")?),
                "bypassDocumentValidation" => {
                    options.bypass_document_validation =
                        Some(Self::parse_bool_field(value, "bypassDocumentValidation")?);
                }
                "maxTimeMS" => {
                    let millis = Self::parse_non_negative_u64(value, "maxTimeMS")?;
                    options.max_time = Some(Duration::from_millis(millis));
                }
                "projection" => {
                    options.projection = Some(Self::parse_document_field(value, "projection")?)
                }
                "returnDocument" => {
                    options.return_document = Some(Self::parse_return_document(value)?);
                }
                "sort" => options.sort = Some(Self::parse_document_field(value, "sort")?),
                "collation" => options.collation = Some(Self::parse_collation_value(value)?),
                "hint" => options.hint = Some(Self::parse_hint_value(value)?),
                "let" => options.let_vars = Some(Self::parse_document_field(value, "let")?),
                "comment" => options.comment = Some(Self::parse_bson_value(value)?),
                other => {
                    return Err(tr_format(
                        "Parameter '{}' is not supported in findOneAndReplace options. Allowed: writeConcern, upsert, bypassDocumentValidation, maxTimeMS, projection, returnDocument, sort, collation, hint, let, comment.",
                        &[other],
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn parse_find_one_and_delete_options(
        source: &str,
    ) -> Result<Option<FindOneAndDeleteParsedOptions>, String> {
        let value: Value = Self::parse_shell_json_value(source)?;
        let object = value
            .as_object()
            .ok_or_else(|| String::from(tr("findOneAndDelete options must be a JSON object.")))?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = FindOneAndDeleteParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "writeConcern" => options.write_concern = Self::parse_write_concern_value(value)?,
                "maxTimeMS" => {
                    let millis = Self::parse_non_negative_u64(value, "maxTimeMS")?;
                    options.max_time = Some(Duration::from_millis(millis));
                }
                "projection" => {
                    options.projection = Some(Self::parse_document_field(value, "projection")?)
                }
                "sort" => options.sort = Some(Self::parse_document_field(value, "sort")?),
                "collation" => options.collation = Some(Self::parse_collation_value(value)?),
                "hint" => options.hint = Some(Self::parse_hint_value(value)?),
                "let" => options.let_vars = Some(Self::parse_document_field(value, "let")?),
                "comment" => options.comment = Some(Self::parse_bson_value(value)?),
                other => {
                    return Err(tr_format(
                        "Parameter '{}' is not supported in findOneAndDelete options. Allowed: writeConcern, maxTimeMS, projection, sort, collation, hint, let, comment.",
                        &[other],
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn parse_find_one_and_modify(&self, source: &str) -> Result<QueryOperation, String> {
        if source.trim().is_empty() {
            return Err(String::from(tr(
                "findOneAndModify requires a JSON object with parameters.",
            )));
        }

        let value: Value = Self::parse_shell_json_value(source)?;
        let object = value
            .as_object()
            .ok_or_else(|| String::from(tr("findOneAndModify expects a JSON object.")))?;

        let mut filter = Document::new();
        let mut update_spec: Option<UpdateModificationsSpec> = None;
        let mut remove = false;
        let mut upsert = None;
        let mut bypass_document_validation = None;
        let mut array_filters = None;
        let mut max_time = None;
        let mut projection = None;
        let mut return_after: Option<bool> = None;
        let mut sort_doc = None;
        let mut write_concern = None;
        let mut collation = None;
        let mut hint = None;
        let mut let_vars = None;
        let mut comment = None;

        for (key, value) in object {
            match key.as_str() {
                "query" => {
                    let json = serde_json::to_string(value)
                        .map_err(|error| format!("JSON serialize error: {error}"))?;
                    filter = Self::parse_json_object(&json)?;
                }
                "sort" => {
                    let json = serde_json::to_string(value)
                        .map_err(|error| format!("JSON serialize error: {error}"))?;
                    sort_doc = Some(Self::parse_json_object(&json)?);
                }
                "update" => {
                    let json = serde_json::to_string(value)
                        .map_err(|error| format!("JSON serialize error: {error}"))?;
                    update_spec = Some(Self::parse_update_spec(&json)?);
                }
                "remove" => {
                    remove = value
                        .as_bool()
                        .ok_or_else(|| String::from(tr("Parameter 'remove' must be a boolean.")))?;
                }
                "new" | "returnNewDocument" => {
                    let flag = value
                        .as_bool()
                        .ok_or_else(|| String::from(tr("Parameter 'new' must be a boolean.")))?;
                    if let Some(current) = return_after {
                        if current != flag {
                            return Err(String::from(tr(
                                "Parameters 'new' and 'returnOriginal' conflict.",
                            )));
                        }
                    } else {
                        return_after = Some(flag);
                    }
                }
                "returnOriginal" => {
                    let flag = value.as_bool().ok_or_else(|| {
                        String::from(tr("Parameter 'returnOriginal' must be a boolean."))
                    })?;
                    let desired_after = !flag;
                    if let Some(current) = return_after {
                        if current != desired_after {
                            return Err(String::from(tr(
                                "Parameters 'new' and 'returnOriginal' conflict.",
                            )));
                        }
                    } else {
                        return_after = Some(desired_after);
                    }
                }
                "fields" | "projection" => {
                    let json = serde_json::to_string(value)
                        .map_err(|error| format!("JSON serialize error: {error}"))?;
                    let document = Self::parse_json_object(&json)?;
                    if projection.is_some() {
                        return Err(String::from(tr(
                            "Parameters 'fields' and 'projection' cannot be set at the same time.",
                        )));
                    }
                    projection = Some(document);
                }
                "upsert" => {
                    upsert = Some(Self::parse_bool_field(value, "upsert")?);
                }
                "bypassDocumentValidation" => {
                    bypass_document_validation =
                        Some(Self::parse_bool_field(value, "bypassDocumentValidation")?);
                }
                "arrayFilters" => {
                    array_filters = Some(Self::parse_array_filters(value)?);
                }
                "maxTimeMS" => {
                    let millis = Self::parse_non_negative_u64(value, "maxTimeMS")?;
                    max_time = Some(Duration::from_millis(millis));
                }
                "writeConcern" => {
                    write_concern = Self::parse_write_concern_value(value)?;
                }
                "collation" => {
                    collation = Some(Self::parse_collation_value(value)?);
                }
                "hint" => {
                    hint = Some(Self::parse_hint_value(value)?);
                }
                "let" => {
                    let json = serde_json::to_string(value)
                        .map_err(|error| format!("JSON serialize error: {error}"))?;
                    let_vars = Some(Self::parse_json_object(&json)?);
                }
                "comment" => {
                    comment = Some(Self::parse_bson_value(value)?);
                }
                other => {
                    return Err(tr_format(
                        "Parameter '{}' is not supported in findOneAndModify.",
                        &[other],
                    ));
                }
            }
        }

        if remove {
            if update_spec.is_some() {
                return Err(String::from(tr(
                    "Parameter 'update' must not be set together with remove=true.",
                )));
            }
            if upsert.is_some() {
                return Err(String::from(tr(
                    "Parameter 'upsert' is not supported when remove=true.",
                )));
            }
            if bypass_document_validation.is_some() {
                return Err(String::from(tr(
                    "Parameter 'bypassDocumentValidation' is not supported when remove=true.",
                )));
            }
            if array_filters.is_some() {
                return Err(String::from(tr(
                    "Parameter 'arrayFilters' is not supported when remove=true.",
                )));
            }
            if return_after.is_some() {
                return Err(String::from(tr(
                    "Document return options are not supported when remove=true.",
                )));
            }

            let mut options = FindOneAndDeleteParsedOptions::default();
            options.write_concern = write_concern;
            options.max_time = max_time;
            options.projection = projection;
            options.sort = sort_doc;
            options.collation = collation;
            options.hint = hint;
            options.let_vars = let_vars;
            options.comment = comment;

            let options = if options.has_values() { Some(options) } else { None };
            return Ok(QueryOperation::FindOneAndDelete { filter, options });
        }

        let update_spec = update_spec.ok_or_else(|| {
            String::from(tr("findOneAndModify requires an 'update' parameter when remove=false."))
        })?;

        let mut options = FindOneAndUpdateParsedOptions::default();
        options.write_concern = write_concern;
        options.upsert = upsert;
        options.array_filters = array_filters;
        options.bypass_document_validation = bypass_document_validation;
        options.max_time = max_time;
        options.projection = projection;
        options.return_document = return_after
            .map(|after| if after { ReturnDocument::After } else { ReturnDocument::Before });
        options.sort = sort_doc;
        options.collation = collation;
        options.hint = hint;
        options.let_vars = let_vars;
        options.comment = comment;

        let options = if options.has_values() { Some(options) } else { None };
        Ok(QueryOperation::FindOneAndUpdate { filter, update: update_spec, options })
    }

    fn parse_return_document(value: &Value) -> Result<ReturnDocument, String> {
        let text = value
            .as_str()
            .ok_or_else(|| {
                String::from(tr("returnDocument must be the string 'before' or 'after'."))
            })?
            .trim()
            .to_lowercase();

        match text.as_str() {
            "before" => Ok(ReturnDocument::Before),
            "after" => Ok(ReturnDocument::After),
            _ => Err(String::from(tr("returnDocument must be the string 'before' or 'after'."))),
        }
    }

    fn parse_write_concern_value(value: &Value) -> Result<Option<WriteConcern>, String> {
        let object = value
            .as_object()
            .ok_or_else(|| String::from(tr("writeConcern must be a JSON object.")))?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut write_concern = WriteConcern::default();
        let mut has_values = false;

        for (key, value) in object {
            match key.as_str() {
                "w" => {
                    let ack = match value {
                        Value::String(s) => Acknowledgment::from(s.as_str()),
                        Value::Number(number) => {
                            let raw = number.as_u64().ok_or_else(|| {
                                String::from(tr("writeConcern.w must be a non-negative integer."))
                            })?;
                            let nodes = u32::try_from(raw).map_err(|_| {
                                String::from(tr(
                                    "writeConcern.w must not exceed the maximum allowed value.",
                                ))
                            })?;
                            Acknowledgment::Nodes(nodes)
                        }
                        _ => {
                            return Err(String::from(tr(
                                "writeConcern.w must be a string or a number.",
                            )));
                        }
                    };
                    write_concern.w = Some(ack);
                    has_values = true;
                }
                "j" => {
                    let journal = value.as_bool().ok_or_else(|| {
                        String::from(tr("writeConcern.j must be a boolean value."))
                    })?;
                    write_concern.journal = Some(journal);
                    has_values = true;
                }
                "wtimeout" | "wtimeoutMS" => {
                    let millis = value.as_u64().ok_or_else(|| {
                        String::from(tr("writeConcern.wtimeout must be a non-negative integer."))
                    })?;
                    write_concern.w_timeout = Some(Duration::from_millis(millis));
                    has_values = true;
                }
                other => {
                    return Err(tr_format(
                        "Parameter '{}' is not supported inside writeConcern. Allowed: w, j, wtimeout.",
                        &[other],
                    ));
                }
            }
        }

        if has_values { Ok(Some(write_concern)) } else { Ok(None) }
    }

    fn parse_collation_value(value: &Value) -> Result<Collation, String> {
        let object = value
            .as_object()
            .ok_or_else(|| String::from(tr("collation must be a JSON object.")))?;
        let document =
            bson::to_document(object).map_err(|error| format!("BSON conversion error: {error}"))?;
        bson::from_document::<Collation>(document)
            .map_err(|error| format!("Collation parse error: {error}"))
    }

    fn parse_hint_value(value: &Value) -> Result<Hint, String> {
        match value {
            Value::String(name) => Ok(Hint::Name(name.clone())),
            Value::Object(map) => {
                let document = bson::to_document(map)
                    .map_err(|error| format!("BSON conversion error: {error}"))?;
                Ok(Hint::Keys(document))
            }
            _ => Err(String::from(tr(
                "hint must be a string or a JSON object with index specification.",
            ))),
        }
    }

    fn parse_non_negative_u64(value: &Value, field: &str) -> Result<u64, String> {
        match value {
            Value::Number(number) => number.as_u64().ok_or_else(|| {
                tr_format("Parameter '{}' must be a non-negative integer.", &[field])
            }),
            _ => Err(tr_format("Parameter '{}' must be a non-negative integer.", &[field])),
        }
    }

    fn split_arguments(args: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escape = false;

        for ch in args.chars() {
            if in_string {
                current.push(ch);
                if escape {
                    escape = false;
                } else if ch == '\\' {
                    escape = true;
                } else if ch == '"' {
                    in_string = false;
                }
                continue;
            }

            match ch {
                '"' => {
                    in_string = true;
                    current.push(ch);
                }
                '{' | '[' => {
                    depth += 1;
                    current.push(ch);
                }
                '}' | ']' => {
                    depth -= 1;
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    result.push(current.trim().to_string());
                    current.clear();
                }
                _ => current.push(ch),
            }
        }

        if !current.trim().is_empty() {
            result.push(current.trim().to_string());
        }

        result
    }

    fn parse_shell_json_value(source: &str) -> Result<Value, String> {
        let quoted = quote_unquoted_keys(source);
        let normalized = Self::preprocess_shell_json(&quoted)?;
        serde_json::from_str(&normalized).map_err(|error| format!("JSON parse error: {error}"))
    }

    fn preprocess_shell_json(source: &str) -> Result<String, String> {
        let chars: Vec<char> = source.chars().collect();
        let len = chars.len();
        let mut result = String::with_capacity(source.len());
        let mut index = 0usize;

        while index < len {
            let ch = chars[index];

            if ch == '\"' {
                let end = Self::skip_double_quoted(&chars, index)?;
                result.extend(&chars[index..end]);
                index = end;
                continue;
            }

            if ch == '\'' {
                let (json_literal, next_index) = Self::collect_single_quoted_string(&chars, index)?;
                result.push_str(&json_literal);
                index = next_index;
                continue;
            }

            if ch == '-' {
                if let Some((replacement, consumed)) =
                    Self::try_parse_negative_constant(&chars, index)?
                {
                    result.push_str(&replacement);
                    index = consumed;
                    continue;
                }
            }

            if ch == '/' {
                if let Some((replacement, consumed)) = Self::try_parse_regex_literal(&chars, index)?
                {
                    result.push_str(&replacement);
                    index = consumed;
                    continue;
                }
            }

            if Self::is_identifier_start(ch) {
                let start_index = index;
                let (identifier, mut next_index) = Self::read_identifier(&chars, index);
                index = next_index;

                if identifier == "new" {
                    next_index = Self::skip_whitespace(&chars, next_index);
                    let (next_identifier, after_identifier) =
                        Self::read_identifier(&chars, next_index);
                    if !next_identifier.is_empty() && Self::is_special_construct(&next_identifier) {
                        if let Some((replacement, consumed)) = Self::convert_special_construct(
                            &chars,
                            after_identifier,
                            &next_identifier,
                        )? {
                            result.push_str(&replacement);
                            index = consumed;
                            continue;
                        }
                    }

                    result.push_str("new");
                    if !next_identifier.is_empty() {
                        result.push(' ');
                        result.push_str(&next_identifier);
                        index = after_identifier;
                    }
                    continue;
                }

                if identifier == "function" {
                    let (code, consumed) = Self::extract_function_literal(&chars, start_index)?;
                    let replacement = Self::bson_to_extended_json(Bson::JavaScriptCode(code))?;
                    result.push_str(&replacement);
                    index = consumed;
                    continue;
                }

                if let Some(replacement) = Self::convert_constant(&identifier)? {
                    result.push_str(&replacement);
                    continue;
                }

                if Self::is_special_construct(&identifier) {
                    if let Some((replacement, consumed_until)) =
                        Self::convert_special_construct(&chars, index, &identifier)?
                    {
                        result.push_str(&replacement);
                        index = consumed_until;
                        continue;
                    }
                }

                result.push_str(&identifier);
                continue;
            }

            result.push(ch);
            index += 1;
        }

        Ok(result)
    }

    fn skip_whitespace(chars: &[char], mut index: usize) -> usize {
        let len = chars.len();
        while index < len && chars[index].is_whitespace() {
            index += 1;
        }
        index
    }

    fn read_identifier(chars: &[char], start: usize) -> (String, usize) {
        let len = chars.len();
        if start >= len || !Self::is_identifier_start(chars[start]) {
            return (String::new(), start);
        }
        let mut index = start + 1;
        while index < len && Self::is_identifier_part(chars[index]) {
            index += 1;
        }
        (chars[start..index].iter().collect(), index)
    }

    fn convert_constant(identifier: &str) -> Result<Option<String>, String> {
        match identifier {
            "Infinity" => Ok(Some(Self::bson_to_extended_json(Bson::Double(f64::INFINITY))?)),
            "NaN" => Ok(Some(Self::bson_to_extended_json(Bson::Double(f64::NAN))?)),
            "undefined" => Ok(Some(Self::bson_to_extended_json(Bson::Undefined)?)),
            _ => Ok(None),
        }
    }

    fn matches_keyword(chars: &[char], start: usize, keyword: &str) -> bool {
        let len = chars.len();
        let keyword_len = keyword.len();
        if start + keyword_len > len {
            return false;
        }

        chars[start..start + keyword_len].iter().zip(keyword.chars()).all(|(&ch, kw)| ch == kw)
    }

    fn prev_non_whitespace(chars: &[char], index: usize) -> Option<char> {
        let mut idx = index;
        while idx > 0 {
            idx -= 1;
            let ch = chars[idx];
            if !ch.is_whitespace() {
                return Some(ch);
            }
        }
        None
    }

    fn try_parse_negative_constant(
        chars: &[char],
        index: usize,
    ) -> Result<Option<(String, usize)>, String> {
        if Self::matches_keyword(chars, index + 1, "Infinity") {
            let consumed = index + 1 + "Infinity".len();
            let replacement = Self::bson_to_extended_json(Bson::Double(f64::NEG_INFINITY))?;
            return Ok(Some((replacement, consumed)));
        }

        Ok(None)
    }

    fn try_parse_regex_literal(
        chars: &[char],
        index: usize,
    ) -> Result<Option<(String, usize)>, String> {
        if chars[index] != '/' {
            return Ok(None);
        }

        if let Some(prev) = Self::prev_non_whitespace(chars, index) {
            if !matches!(prev, ':' | ',' | '{' | '[' | '(') {
                return Ok(None);
            }
        }

        let len = chars.len();
        let mut pattern = String::new();
        let mut escape = false;
        let mut cursor = index + 1;

        while cursor < len {
            let ch = chars[cursor];
            if escape {
                pattern.push(ch);
                escape = false;
            } else if ch == '\\' {
                pattern.push(ch);
                escape = true;
            } else if ch == '/' {
                break;
            } else {
                pattern.push(ch);
            }
            cursor += 1;
        }

        if cursor >= len || chars[cursor] != '/' {
            return Err(String::from(tr("Regular expression is not terminated with '/'.")));
        }

        cursor += 1;
        let mut options = String::new();
        while cursor < len && chars[cursor].is_ascii_alphabetic() {
            options.push(chars[cursor]);
            cursor += 1;
        }

        let regex = Regex { pattern, options };
        let replacement = Self::bson_to_extended_json(Bson::RegularExpression(regex))?;
        Ok(Some((replacement, cursor)))
    }

    fn extract_function_literal(chars: &[char], start: usize) -> Result<(String, usize), String> {
        let len = chars.len();
        let mut index = start;
        let mut buffer = String::new();
        let mut in_string = false;
        let mut string_delim = '\'';
        let mut escape = false;
        let mut paren_depth = 0i32;
        let mut brace_depth = 0i32;
        let mut encountered_brace = false;

        while index < len {
            let ch = chars[index];
            buffer.push(ch);
            index += 1;

            if in_string {
                if escape {
                    escape = false;
                    continue;
                }
                if ch == '\\' {
                    escape = true;
                } else if ch == string_delim {
                    in_string = false;
                }
                continue;
            }

            match ch {
                '\'' | '"' => {
                    in_string = true;
                    string_delim = ch;
                }
                '(' => paren_depth += 1,
                ')' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                }
                '{' => {
                    brace_depth += 1;
                    encountered_brace = true;
                }
                '}' => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                        if encountered_brace && brace_depth == 0 && paren_depth == 0 {
                            break;
                        }
                    }
                }
                _ => {}
            }
        }

        if brace_depth != 0 {
            return Err(String::from(tr("Function is missing a closing brace.")));
        }

        Ok((buffer.trim().to_string(), index))
    }

    fn collect_single_quoted_string(
        chars: &[char],
        start: usize,
    ) -> Result<(String, usize), String> {
        let (raw, next_index) = Self::read_single_quoted(chars, start)?;
        Ok((Value::String(raw).to_string(), next_index))
    }

    fn read_single_quoted(chars: &[char], start: usize) -> Result<(String, usize), String> {
        let mut buffer = String::new();
        let mut index = start + 1;
        let len = chars.len();

        while index < len {
            match chars[index] {
                '\\' => {
                    index += 1;
                    if index >= len {
                        return Err(String::from(tr(
                            "Single-quoted string contains an unfinished escape sequence.",
                        )));
                    }

                    let (ch, consumed) = match chars[index] {
                        '\\' => ('\\', 1),
                        '\'' => ('\'', 1),
                        '"' => ('"', 1),
                        'n' => ('\n', 1),
                        'r' => ('\r', 1),
                        't' => ('\t', 1),
                        'b' => ('\u{0008}', 1),
                        'f' => ('\u{000C}', 1),
                        'v' => ('\u{000B}', 1),
                        '0' => ('\u{0000}', 1),
                        'x' => {
                            if index + 2 >= len {
                                return Err(String::from(tr(
                                    "The \\x sequence must contain two hex digits.",
                                )));
                            }
                            let high = Self::hex_value(chars[index + 1])?;
                            let low = Self::hex_value(chars[index + 2])?;
                            let value = ((high << 4) | low) as u32;
                            (Self::codepoint_to_char(value)?, 3)
                        }
                        'u' => {
                            if index + 4 >= len {
                                return Err(String::from(tr(
                                    "The \\u sequence must contain four hex digits.",
                                )));
                            }
                            let mut value = 0u32;
                            for offset in 1..=4 {
                                value = (value << 4) | Self::hex_value(chars[index + offset])?;
                            }
                            (Self::codepoint_to_char(value)?, 5)
                        }
                        other => (other, 1),
                    };

                    buffer.push(ch);
                    index += consumed;
                }
                '\'' => return Ok((buffer, index + 1)),
                other => {
                    buffer.push(other);
                    index += 1;
                }
            }
        }

        Err(String::from(tr("Single-quoted string is not closed.")))
    }

    fn skip_single_quoted(chars: &[char], start: usize) -> Result<usize, String> {
        let (_, next) = Self::read_single_quoted(chars, start)?;
        Ok(next)
    }

    fn skip_double_quoted(chars: &[char], start: usize) -> Result<usize, String> {
        let mut index = start + 1;
        let len = chars.len();

        while index < len {
            match chars[index] {
                '\\' => {
                    index += 2;
                }
                '\"' => return Ok(index + 1),
                _ => index += 1,
            }
        }

        Err(String::from(tr("Double-quoted string is not closed.")))
    }

    fn hex_value(ch: char) -> Result<u32, String> {
        ch.to_digit(16).ok_or_else(|| {
            tr_format("Invalid hex character '{}' in escape sequence.", &[&ch.to_string()])
        })
    }

    fn codepoint_to_char(value: u32) -> Result<char, String> {
        char::from_u32(value).ok_or_else(|| {
            tr_format("Code point 0x{} is not a valid character.", &[&format!("{value:04X}")])
        })
    }

    fn is_identifier_start(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }

    fn is_identifier_part(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_' || ch == '.'
    }

    fn is_special_construct(identifier: &str) -> bool {
        matches!(
            identifier,
            "ObjectId"
                | "ObjectId.fromDate"
                | "ISODate"
                | "Date"
                | "NumberDecimal"
                | "NumberLong"
                | "NumberInt"
                | "NumberDouble"
                | "Number"
                | "String"
                | "Boolean"
                | "BinData"
                | "HexData"
                | "UUID"
                | "Timestamp"
                | "RegExp"
                | "Code"
                | "Array"
                | "Object"
                | "DBRef"
                | "MinKey"
                | "MaxKey"
                | "Undefined"
        )
    }

    fn convert_special_construct(
        chars: &[char],
        after_identifier: usize,
        identifier: &str,
    ) -> Result<Option<(String, usize)>, String> {
        let mut index = after_identifier;
        let len = chars.len();

        while index < len && chars[index].is_whitespace() {
            index += 1;
        }

        if index >= len || chars[index] != '(' {
            return Ok(None);
        }

        index += 1;
        let args_start = index;
        let mut depth = 0usize;

        while index < len {
            match chars[index] {
                '(' => {
                    depth += 1;
                    index += 1;
                }
                ')' => {
                    if depth == 0 {
                        let args: String = chars[args_start..index].iter().collect();
                        let replacement = Self::build_extended_json(identifier, &args)?;
                        return Ok(Some((replacement, index + 1)));
                    }
                    depth -= 1;
                    index += 1;
                }
                '\'' => {
                    index = Self::skip_single_quoted(chars, index)?;
                }
                '\"' => {
                    index = Self::skip_double_quoted(chars, index)?;
                }
                _ => index += 1,
            }
        }

        Err(tr_format("Call parenthesis for {} is not closed.", &[identifier]))
    }

    fn build_extended_json(identifier: &str, args: &str) -> Result<String, String> {
        let parts = Self::split_arguments(args);
        let bson = Self::build_special_bson(identifier, &parts)?;
        Self::bson_to_extended_json(bson)
    }

    fn build_special_bson(identifier: &str, parts: &[String]) -> Result<Bson, String> {
        match identifier {
            "ObjectId" => {
                let object_id = match parts.len() {
                    0 => ObjectId::new(),
                    1 => {
                        let value = Self::parse_shell_json_value(&parts[0])?;
                        let hex = Self::value_as_string(&value)?;
                        ObjectId::from_str(&hex).map_err(|_| {
                            String::from(tr(
                                "ObjectId requires a 24-character hex string or no arguments.",
                            ))
                        })?
                    }
                    _ => {
                        return Err(String::from(tr(
                            "ObjectId accepts either zero or one string argument.",
                        )));
                    }
                };
                Ok(Bson::ObjectId(object_id))
            }
            "ObjectId.fromDate" => {
                if parts.len() != 1 {
                    return Err(String::from(tr("ObjectId.fromDate expects a single argument.")));
                }
                let date = Self::parse_date_constructor(&[parts[0].clone()])?;
                let seconds = (date.timestamp_millis() / 1000) as u32;
                Ok(Bson::ObjectId(ObjectId::from_parts(seconds, [0; 5], [0; 3])))
            }
            "ISODate" | "Date" => {
                let datetime = Self::parse_date_constructor(parts)?;
                Ok(Bson::DateTime(datetime))
            }
            "NumberDecimal" => {
                let literal = parts.get(0).cloned().unwrap_or_else(|| String::from(tr("0")));
                let value = Self::parse_shell_json_value(&literal)?;
                let text = Self::value_as_string(&value)?;
                let decimal = Decimal128::from_str(&text).map_err(|_| {
                    String::from(tr("NumberDecimal expects a valid decimal value."))
                })?;
                Ok(Bson::Decimal128(decimal))
            }
            "NumberLong" => {
                let literal = parts.get(0).cloned().unwrap_or_else(|| String::from(tr("0")));
                let value = Self::parse_shell_json_value(&literal)?;
                let text = Self::value_as_string(&value)?;
                let parsed = i128::from_str(&text)
                    .map_err(|_| String::from(tr("NumberLong expects an integer.")))?;
                let value = i64::try_from(parsed)
                    .map_err(|_| String::from(tr("NumberLong value exceeds the i64 range.")))?;
                Ok(Bson::Int64(value))
            }
            "NumberInt" => {
                let literal = parts.get(0).cloned().unwrap_or_else(|| String::from(tr("0")));
                let value = Self::parse_shell_json_value(&literal)?;
                let text = Self::value_as_string(&value)?;
                let parsed = i64::from_str(&text)
                    .map_err(|_| String::from(tr("NumberInt expects an integer.")))?;
                let value = i32::try_from(parsed)
                    .map_err(|_| String::from(tr("NumberInt value is out of the Int32 range.")))?;
                Ok(Bson::Int32(value))
            }
            "NumberDouble" | "Number" => {
                let literal = parts.get(0).cloned().unwrap_or_else(|| String::from(tr("0")));
                let value = Self::parse_shell_json_value(&literal)?;
                let number = Self::value_as_f64(&value)?;
                Ok(Bson::Double(number))
            }
            "Boolean" => {
                let literal = parts.get(0).cloned().unwrap_or_else(|| String::from(tr("false")));
                let value = Self::parse_shell_json_value(&literal)?;
                let flag = Self::value_as_bool(&value)?;
                Ok(Bson::Boolean(flag))
            }
            "String" => {
                let text = if let Some(arg) = parts.get(0) {
                    let value = Self::parse_shell_json_value(arg)?;
                    Self::value_as_string(&value)?
                } else {
                    String::new()
                };
                Ok(Bson::String(text))
            }
            "UUID" => {
                let uuid = if let Some(arg) = parts.get(0) {
                    let value = Self::parse_shell_json_value(arg)?;
                    let text = Self::value_as_string(&value)?;
                    Uuid::parse_str(&text)
                            .map_err(|_| String::from(tr("UUID expects a string in the format xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.")))?
                } else {
                    Uuid::new_v4()
                };
                Ok(Bson::Binary(Binary {
                    subtype: BinarySubtype::Uuid,
                    bytes: uuid.as_bytes().to_vec(),
                }))
            }
            "BinData" => {
                if parts.len() != 2 {
                    return Err(String::from(tr(
                        "BinData expects two arguments: a subtype and a base64 string.",
                    )));
                }
                let subtype_value = Self::parse_shell_json_value(&parts[0])?;
                let subtype = Self::value_as_u8(&subtype_value)?;
                let data_value = Self::parse_shell_json_value(&parts[1])?;
                let encoded = data_value.as_str().ok_or_else(|| {
                    String::from(tr("BinData expects a base64 string as the second argument."))
                })?;
                let bytes = BASE64_STANDARD
                    .decode(encoded)
                    .map_err(|_| String::from(tr("Unable to decode the BinData base64 string.")))?;
                Ok(Bson::Binary(Binary { subtype: BinarySubtype::from(subtype), bytes }))
            }
            "HexData" => {
                if parts.len() != 2 {
                    return Err(String::from(tr(
                        "HexData expects two arguments: a subtype and a hex string.",
                    )));
                }
                let subtype_value = Self::parse_shell_json_value(&parts[0])?;
                let subtype = Self::value_as_u8(&subtype_value)?;
                let hex_value = Self::parse_shell_json_value(&parts[1])?;
                let hex_string = hex_value.as_str().ok_or_else(|| {
                    String::from(tr("HexData expects a string as the second argument."))
                })?;
                let bytes = Self::decode_hex(hex_string)?;
                Ok(Bson::Binary(Binary { subtype: BinarySubtype::from(subtype), bytes }))
            }
            "Array" => {
                let mut items = Vec::new();
                for part in parts {
                    let value = Self::parse_shell_bson_value(part)?;
                    items.push(value);
                }
                Ok(Bson::Array(items))
            }
            "Object" => {
                if parts.is_empty() {
                    return Ok(Bson::Document(Document::new()));
                }
                let value = Self::parse_shell_bson_value(&parts[0])?;
                match value {
                    Bson::Document(doc) => Ok(Bson::Document(doc)),
                    other => Err(tr_format(
                        "Object expects a JSON object, but received a value of type {}.",
                        &[&format!("{other:?}")],
                    )),
                }
            }
            "Timestamp" => {
                if parts.len() != 2 {
                    return Err(String::from(tr(
                        "Timestamp expects two arguments: time and increment.",
                    )));
                }
                let time = Self::parse_timestamp_seconds(&parts[0])?;
                let increment = Self::parse_u32_argument(&parts[1], "Timestamp", "i")?;
                Ok(Bson::Timestamp(BsonTimestamp { time, increment }))
            }
            "RegExp" => {
                if parts.is_empty() || parts.len() > 2 {
                    return Err(String::from(tr("RegExp expects a pattern and optional options.")));
                }
                let pattern_value = Self::parse_shell_json_value(&parts[0])?;
                let pattern = pattern_value
                    .as_str()
                    .ok_or_else(|| String::from(tr("RegExp expects a string pattern.")))?
                    .to_string();
                let options = if let Some(arg) = parts.get(1) {
                    let options_value = Self::parse_shell_json_value(arg)?;
                    options_value
                        .as_str()
                        .ok_or_else(|| String::from(tr("RegExp options must be a string.")))?
                        .to_string()
                } else {
                    String::new()
                };
                Ok(Bson::RegularExpression(Regex { pattern, options }))
            }
            "Code" => {
                let code_text = parts.get(0).cloned().unwrap_or_else(String::new);
                let code_value = Self::parse_shell_json_value(&code_text)?;
                let code = Self::value_as_string(&code_value)?;
                if let Some(scope_part) = parts.get(1) {
                    let scope_bson = Self::parse_shell_bson_value(scope_part)?;
                    let scope = match scope_bson {
                        Bson::Document(doc) => doc,
                        _ => {
                            return Err(String::from(tr(
                                "The second argument to Code must be an object.",
                            )));
                        }
                    };
                    Ok(Bson::JavaScriptCodeWithScope(JavaScriptCodeWithScope { code, scope }))
                } else {
                    Ok(Bson::JavaScriptCode(code))
                }
            }
            "DBRef" => {
                if parts.len() < 2 || parts.len() > 3 {
                    return Err(String::from(tr(
                        "DBRef expects two or three arguments: collection, _id, and an optional database name.",
                    )));
                }
                let collection_value = Self::parse_shell_json_value(&parts[0])?;
                let collection = Self::value_as_string(&collection_value)?;
                let id_bson = Self::parse_shell_bson_value(&parts[1])?;
                let id = match id_bson {
                    Bson::ObjectId(oid) => oid,
                    _ => {
                        return Err(String::from(tr(
                            "DBRef expects an ObjectId as the second argument.",
                        )));
                    }
                };
                let db_name = if let Some(db_part) = parts.get(2) {
                    let value = Self::parse_shell_json_value(db_part)?;
                    Some(Self::value_as_string(&value)?)
                } else {
                    None
                };
                let mut doc = Document::new();
                doc.insert("$ref", Bson::String(collection));
                doc.insert("$id", Bson::ObjectId(id));
                if let Some(db) = db_name {
                    doc.insert("$db", Bson::String(db));
                }
                Ok(Bson::Document(doc))
            }
            "MinKey" => Ok(Bson::MinKey),
            "MaxKey" => Ok(Bson::MaxKey),
            "Undefined" => Ok(Bson::Undefined),
            _ => Err(tr_format("Constructor '{}' is not supported.", &[identifier])),
        }
    }

    fn bson_to_extended_json(value: Bson) -> Result<String, String> {
        serde_json::to_string(&value).map_err(|error| format!("JSON serialization error: {error}"))
    }

    fn parse_shell_bson_value(source: &str) -> Result<Bson, String> {
        let normalized = Self::preprocess_shell_json(source)?;
        serde_json::from_str(&normalized).map_err(|error| format!("JSON parse error: {error}"))
    }

    fn value_as_bool(value: &Value) -> Result<bool, String> {
        if let Some(flag) = value.as_bool() {
            Ok(flag)
        } else if let Some(number) = value.as_i64() {
            Ok(number != 0)
        } else if let Some(number) = value.as_u64() {
            Ok(number != 0)
        } else if let Some(text) = value.as_str() {
            match text.trim().to_lowercase().as_str() {
                "true" | "1" => Ok(true),
                "false" | "0" => Ok(false),
                _ => Err(String::from(tr("String must be true or false."))),
            }
        } else {
            Err(String::from(tr(
                "Value must be boolean, numeric, or a string equal to true/false.",
            )))
        }
    }

    fn value_as_f64(value: &Value) -> Result<f64, String> {
        if let Some(number) = value.as_f64() {
            Ok(number)
        } else if let Some(number) = value.as_i64() {
            Ok(number as f64)
        } else if let Some(number) = value.as_u64() {
            Ok(number as f64)
        } else if let Some(text) = value.as_str() {
            match text.trim().to_lowercase().as_str() {
                "infinity" => Ok(f64::INFINITY),
                "-infinity" => Ok(f64::NEG_INFINITY),
                "nan" => Ok(f64::NAN),
                other => other
                    .parse::<f64>()
                    .map_err(|_| String::from(tr("Failed to convert string value to number."))),
            }
        } else {
            Err(String::from(tr("Value must be a number or a string.")))
        }
    }

    fn parse_date_constructor(parts: &[String]) -> Result<DateTime, String> {
        if parts.is_empty() {
            return Ok(DateTime::now());
        }

        if parts.len() == 1 {
            let bson = Self::parse_shell_bson_value(&parts[0])?;
            return match bson {
                Bson::DateTime(dt) => Ok(dt),
                Bson::String(text) => DateTime::parse_rfc3339_str(&text)
                    .or_else(|_| {
                        if let Ok(ms) = text.parse::<i128>() {
                            Ok(DateTime::from_millis(ms as i64))
                        } else {
                            Err(())
                        }
                    })
                    .map_err(|_| String::from(tr("Failed to convert string to date."))),
                Bson::Int32(value) => Ok(DateTime::from_millis(value as i64)),
                Bson::Int64(value) => Ok(DateTime::from_millis(value)),
                Bson::Double(value) => Ok(DateTime::from_millis(value as i64)),
                Bson::Decimal128(value) => {
                    let millis = value.to_string().parse::<f64>().map_err(|_| {
                        String::from(tr("Failed to convert Decimal128 to a number."))
                    })?;
                    Ok(DateTime::from_millis(millis as i64))
                }
                Bson::Null => Ok(DateTime::now()),
                other => Err(tr_format(
                    "Cannot convert value of type {} to a date.",
                    &[&format!("{other:?}")],
                )),
            };
        }

        Self::construct_date_from_components(parts)
    }

    fn construct_date_from_components(parts: &[String]) -> Result<DateTime, String> {
        let mut components = [0i64; 7];
        for (index, part) in parts.iter().enumerate().take(7) {
            let value = Self::parse_shell_json_value(part)?;
            let number = Self::value_as_f64(&value)?;
            components[index] = number.trunc() as i64;
        }

        let year = components[0] as i32;
        let month_zero = components.get(1).copied().unwrap_or(0);
        let month = (month_zero + 1).clamp(1, 12) as u32;
        let day = components.get(2).copied().unwrap_or(1).clamp(1, 31) as u32;
        let hour = components.get(3).copied().unwrap_or(0).clamp(0, 23) as u32;
        let minute = components.get(4).copied().unwrap_or(0).clamp(0, 59) as u32;
        let second = components.get(5).copied().unwrap_or(0).clamp(0, 59) as u32;
        let millis = components.get(6).copied().unwrap_or(0);

        let base =
            Utc.with_ymd_and_hms(year, month, day, hour, minute, second).single().ok_or_else(
                || String::from(tr("Unable to construct a date with the specified components.")),
            )?;

        let chrono_dt = base + ChronoDuration::milliseconds(millis);
        Ok(DateTime::from_millis(chrono_dt.timestamp_millis()))
    }

    fn parse_timestamp_seconds(value: &str) -> Result<u32, String> {
        let trimmed = value.trim();
        if let Some(prefix) = trimmed.strip_suffix(".getTime()/1000") {
            let date = Self::parse_date_constructor(&[prefix.trim().to_string()])?;
            return Ok((date.timestamp_millis() / 1000) as u32);
        }

        if let Some(prefix) = trimmed.strip_suffix(".getTime()") {
            let date = Self::parse_date_constructor(&[prefix.trim().to_string()])?;
            return Ok(date.timestamp_millis() as u32);
        }

        let bson = Self::parse_shell_bson_value(trimmed)?;
        match bson {
            Bson::DateTime(dt) => Ok((dt.timestamp_millis() / 1000) as u32),
            Bson::Int32(value) => Ok(value as u32),
            Bson::Int64(value) => u32::try_from(value)
                .map_err(|_| String::from(tr("Timestamp time value must fit into u32."))),
            Bson::Double(value) => Ok(value as u32),
            Bson::String(text) => {
                if let Ok(dt) = DateTime::parse_rfc3339_str(&text) {
                    Ok((dt.timestamp_millis() / 1000) as u32)
                } else {
                    let number = text.parse::<f64>().map_err(|_| {
                        String::from(tr(
                            "String value in Timestamp must be a number or an ISO date.",
                        ))
                    })?;
                    Ok(number as u32)
                }
            }
            other => Err(tr_format(
                "The first argument to Timestamp must be a number or a date; received {}.",
                &[&format!("{other:?}")],
            )),
        }
    }

    fn parse_u32_argument(value: &str, context: &str, field: &str) -> Result<u32, String> {
        let bson = Self::parse_shell_bson_value(value)?;
        match bson {
            Bson::Int32(v) => Ok(v as u32),
            Bson::Int64(v) => u32::try_from(v)
                .map_err(|_| tr_format("{}::{} must fit into u32.", &[context, field])),
            Bson::Double(v) => Ok(v as u32),
            Bson::String(text) => text
                .parse::<u32>()
                .map_err(|_| tr_format("{}::{} must be a positive integer.", &[context, field])),
            other => Err(tr_format(
                "{}::{} must be a number, received {}.",
                &[context, field, &format!("{other:?}")],
            )),
        }
    }

    fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
        let cleaned: String = value.chars().filter(|ch| !ch.is_whitespace()).collect();
        if cleaned.len() % 2 != 0 {
            return Err(String::from(tr("Hex string must contain an even number of characters.")));
        }
        let mut bytes = Vec::with_capacity(cleaned.len() / 2);
        let chars: Vec<char> = cleaned.chars().collect();
        for chunk in chars.chunks(2) {
            let high = Self::hex_value(chunk[0])?;
            let low = Self::hex_value(chunk[1])?;
            bytes.push(((high << 4) | low) as u8);
        }
        Ok(bytes)
    }

    fn value_as_string(value: &Value) -> Result<String, String> {
        if let Some(s) = value.as_str() {
            Ok(s.to_string())
        } else if value.is_number() {
            Ok(value.to_string())
        } else {
            Err(String::from(tr("Argument must be a string or a number.")))
        }
    }

    fn value_as_u8(value: &Value) -> Result<u8, String> {
        if let Some(number) = value.as_u64() {
            u8::try_from(number)
                .map_err(|_| String::from(tr("BinData subtype must be a number from 0 to 255.")))
        } else if let Some(number) = value.as_i64() {
            if (0..=255).contains(&number) {
                Ok(number as u8)
            } else {
                Err(String::from(tr("BinData subtype must be a number from 0 to 255.")))
            }
        } else if let Some(text) = value.as_str() {
            u8::from_str_radix(text, 16)
                .map_err(|_| String::from(tr("BinData subtype must be a number or a hex string.")))
        } else {
            Err(String::from(tr("BinData subtype must be a number.")))
        }
    }

    fn parse_json_object(source: &str) -> Result<Document, String> {
        let value = Self::parse_shell_json_value(source)?;
        let object =
            value.as_object().ok_or_else(|| String::from(tr("Argument must be a JSON object.")))?;
        bson::to_document(object).map_err(|error| format!("BSON conversion error: {error}"))
    }

    fn parse_index_argument(source: &str) -> Result<Bson, String> {
        let value = Self::parse_shell_bson_value(source)?;
        match value {
            Bson::String(name) => Ok(Bson::String(name)),
            Bson::Document(doc) => Ok(Bson::Document(doc)),
            _ => Err(String::from(tr(
                "Index argument must be a string with the index name or an object with keys.",
            ))),
        }
    }
}

pub fn parse_collection_query(
    db_name: &str,
    collection: &str,
    text: &str,
) -> Result<QueryOperation, String> {
    QueryParser { db_name, collection }.parse_query(text)
}

fn u64_to_bson(value: u64) -> Bson {
    if value <= i64::MAX as u64 {
        Bson::Int64(value as i64)
    } else {
        Bson::String(value.to_string())
    }
}

pub fn run_collection_query(
    client: Arc<Client>,
    db_name: String,
    collection_name: String,
    operation: QueryOperation,
    skip: u64,
    limit: u64,
    timeout: Option<Duration>,
) -> Result<QueryResult, String> {
    let database = client.database(&db_name);
    let collection = database.collection::<Document>(&collection_name);

    match operation {
        QueryOperation::Find { filter } => {
            if limit == 0 {
                return Ok(QueryResult::Documents(Vec::new()));
            }

            let mut builder = collection.find(filter);
            if skip > 0 {
                builder = builder.skip(skip);
            }

            let limit_capped = limit.min(i64::MAX as u64) as i64;
            if limit_capped > 0 {
                builder = builder.limit(limit_capped);
            }

            if let Some(timeout) = timeout {
                builder = builder.max_time(timeout);
            }

            let cursor = builder.run().map_err(|err| err.to_string())?;
            let take_limit = if limit_capped > 0 { limit_capped as usize } else { usize::MAX };
            let mut documents = Vec::new();

            for result in cursor.into_iter().take(take_limit) {
                let document = result.map_err(|err| err.to_string())?;
                documents.push(Bson::Document(document));
            }

            Ok(QueryResult::Documents(documents))
        }
        QueryOperation::FindOne { filter } => {
            let mut builder = collection.find(filter);
            if skip > 0 {
                builder = builder.skip(skip);
            }
            builder = builder.limit(1);

            if let Some(timeout) = timeout {
                builder = builder.max_time(timeout);
            }

            let cursor = builder.run().map_err(|err| err.to_string())?;
            if let Some(result) = cursor.into_iter().next() {
                let document = result.map_err(|err| err.to_string())?;
                Ok(QueryResult::SingleDocument { document })
            } else {
                Ok(QueryResult::Documents(Vec::new()))
            }
        }
        QueryOperation::Count { filter } => {
            let mut action = collection.count_documents(filter);
            if let Some(timeout) = timeout {
                action = action.max_time(timeout);
            }

            let count = action.run().map_err(|err| err.to_string())?;

            let count_value = if count <= i64::MAX as u64 {
                Bson::Int64(count as i64)
            } else {
                Bson::String(count.to_string())
            };

            Ok(QueryResult::Count { value: count_value })
        }
        QueryOperation::CountDocuments { filter, options } => {
            let mut builder = collection.count_documents(filter);

            if let Some(opts) = options {
                if let Some(limit) = opts.limit {
                    builder = builder.limit(limit);
                }
                if let Some(skip) = opts.skip {
                    builder = builder.skip(skip);
                }
                if let Some(max_time) = opts.max_time {
                    builder = builder.max_time(max_time);
                }
                if let Some(hint) = opts.hint {
                    builder = builder.hint(hint);
                }
            }

            if let Some(timeout) = timeout {
                builder = builder.max_time(timeout);
            }

            let count = builder.run().map_err(|err| err.to_string())?;

            let count_value = if count <= i64::MAX as u64 {
                Bson::Int64(count as i64)
            } else {
                Bson::String(count.to_string())
            };

            Ok(QueryResult::Count { value: count_value })
        }
        QueryOperation::EstimatedDocumentCount { options } => {
            let mut builder = collection.estimated_document_count();

            if let Some(opts) = options {
                if let Some(max_time) = opts.max_time {
                    builder = builder.max_time(max_time);
                }
            }

            if let Some(timeout) = timeout {
                builder = builder.max_time(timeout);
            }

            let count = builder.run().map_err(|err| err.to_string())?;

            let count_value = if count <= i64::MAX as u64 {
                Bson::Int64(count as i64)
            } else {
                Bson::String(count.to_string())
            };

            Ok(QueryResult::Count { value: count_value })
        }
        QueryOperation::Distinct { field, filter } => {
            let mut action = collection.distinct(field.clone(), filter);
            if let Some(timeout) = timeout {
                action = action.max_time(timeout);
            }

            let values = action.run().map_err(|err| err.to_string())?;

            Ok(QueryResult::Distinct { field, values })
        }
        QueryOperation::Aggregate { mut pipeline } => {
            if skip > 0 {
                let skip_i64 = i64::try_from(skip).unwrap_or(i64::MAX);
                pipeline.push(doc! { "$skip": skip_i64 });
            }

            if limit > 0 {
                let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
                pipeline.push(doc! { "$limit": limit_i64 });
            }

            let mut action = collection.aggregate(pipeline);
            if let Some(timeout) = timeout {
                action = action.max_time(timeout);
            }

            let cursor = action.run().map_err(|err| err.to_string())?;

            let mut documents = Vec::new();
            for result in cursor {
                let document = result.map_err(|err| err.to_string())?;
                documents.push(Bson::Document(document));
            }

            Ok(QueryResult::Documents(documents))
        }
        QueryOperation::InsertOne { document, options } => {
            let mut action = collection.insert_one(document);
            if let Some(opts) = options {
                if let Some(write_concern) = opts.write_concern {
                    action = action.write_concern(write_concern);
                }
            }

            let result = action.run().map_err(|err| err.to_string())?;

            let mut response = Document::new();
            response.insert("operation", Bson::String(String::from(tr("insertOne"))));
            response.insert("insertedId", result.inserted_id);

            Ok(QueryResult::SingleDocument { document: response })
        }
        QueryOperation::InsertMany { documents, options } => {
            let mut action = collection.insert_many(documents);
            if let Some(opts) = options {
                if let Some(ordered) = opts.ordered {
                    action = action.ordered(ordered);
                }
                if let Some(write_concern) = opts.write_concern {
                    action = action.write_concern(write_concern);
                }
            }

            let result = action.run().map_err(|err| err.to_string())?;
            let mut pairs: Vec<(usize, Bson)> = result.inserted_ids.into_iter().collect();
            pairs.sort_by_key(|(index, _)| *index);

            let mut ids_document = Document::new();
            for (index, id) in pairs {
                ids_document.insert(index.to_string(), id);
            }

            let mut response = Document::new();
            response.insert("operation", Bson::String(String::from(tr("insertMany"))));
            response.insert("insertedCount", Bson::Int64(ids_document.len() as i64));
            response.insert("insertedIds", Bson::Document(ids_document));

            Ok(QueryResult::SingleDocument { document: response })
        }
        QueryOperation::UpdateOne { filter, update, options } => {
            let mut action = match update {
                UpdateModificationsSpec::Document(document) => {
                    collection.update_one(filter, document)
                }
                UpdateModificationsSpec::Pipeline(pipeline) => {
                    collection.update_one(filter, pipeline)
                }
            };

            if let Some(opts) = options {
                if let Some(write_concern) = opts.write_concern {
                    action = action.write_concern(write_concern);
                }
                if let Some(upsert) = opts.upsert {
                    action = action.upsert(upsert);
                }
                if let Some(array_filters) = opts.array_filters {
                    action = action.array_filters(array_filters);
                }
                if let Some(collation) = opts.collation {
                    action = action.collation(collation);
                }
                if let Some(hint) = opts.hint {
                    action = action.hint(hint);
                }
                if let Some(bypass) = opts.bypass_document_validation {
                    action = action.bypass_document_validation(bypass);
                }
                if let Some(let_vars) = opts.let_vars {
                    action = action.let_vars(let_vars);
                }
                if let Some(comment) = opts.comment {
                    action = action.comment(comment);
                }
                if let Some(sort) = opts.sort {
                    action = action.sort(sort);
                }
            }

            let result = action.run().map_err(|err| err.to_string())?;

            let mut response = Document::new();
            response.insert("operation", Bson::String(String::from(tr("updateOne"))));
            response.insert("matchedCount", u64_to_bson(result.matched_count));
            response.insert("modifiedCount", u64_to_bson(result.modified_count));
            if let Some(id) = result.upserted_id {
                response.insert("upsertedId", id);
            }

            Ok(QueryResult::SingleDocument { document: response })
        }
        QueryOperation::UpdateMany { filter, update, options } => {
            let mut action = match update {
                UpdateModificationsSpec::Document(document) => {
                    collection.update_many(filter, document)
                }
                UpdateModificationsSpec::Pipeline(pipeline) => {
                    collection.update_many(filter, pipeline)
                }
            };

            if let Some(opts) = options {
                if let Some(write_concern) = opts.write_concern {
                    action = action.write_concern(write_concern);
                }
                if let Some(upsert) = opts.upsert {
                    action = action.upsert(upsert);
                }
                if let Some(array_filters) = opts.array_filters {
                    action = action.array_filters(array_filters);
                }
                if let Some(collation) = opts.collation {
                    action = action.collation(collation);
                }
                if let Some(hint) = opts.hint {
                    action = action.hint(hint);
                }
                if let Some(bypass) = opts.bypass_document_validation {
                    action = action.bypass_document_validation(bypass);
                }
                if let Some(let_vars) = opts.let_vars {
                    action = action.let_vars(let_vars);
                }
                if let Some(comment) = opts.comment {
                    action = action.comment(comment);
                }
                if let Some(sort) = opts.sort {
                    action = action.sort(sort);
                }
            }

            let result = action.run().map_err(|err| err.to_string())?;

            let mut response = Document::new();
            response.insert("operation", Bson::String(String::from(tr("updateMany"))));
            response.insert("matchedCount", u64_to_bson(result.matched_count));
            response.insert("modifiedCount", u64_to_bson(result.modified_count));
            if let Some(id) = result.upserted_id {
                response.insert("upsertedId", id);
            }

            Ok(QueryResult::SingleDocument { document: response })
        }
        QueryOperation::DeleteOne { filter, options } => {
            let mut action = collection.delete_one(filter);
            if let Some(opts) = options {
                if let Some(collation) = opts.collation {
                    action = action.collation(collation);
                }
                if let Some(hint) = opts.hint {
                    action = action.hint(hint);
                }
                if let Some(write_concern) = opts.write_concern {
                    action = action.write_concern(write_concern);
                }
            }

            let result = action.run().map_err(|err| err.to_string())?;
            let deleted_count = result.deleted_count;
            let deleted_bson = u64_to_bson(deleted_count);

            let mut response = Document::new();
            response.insert("operation", Bson::String(String::from(tr("deleteOne"))));
            response.insert("deletedCount", deleted_bson);

            Ok(QueryResult::SingleDocument { document: response })
        }
        QueryOperation::DeleteMany { filter, options } => {
            let mut action = collection.delete_many(filter);
            if let Some(opts) = options {
                if let Some(collation) = opts.collation {
                    action = action.collation(collation);
                }
                if let Some(hint) = opts.hint {
                    action = action.hint(hint);
                }
                if let Some(write_concern) = opts.write_concern {
                    action = action.write_concern(write_concern);
                }
            }

            let result = action.run().map_err(|err| err.to_string())?;
            let deleted_count = result.deleted_count;
            let deleted_bson = u64_to_bson(deleted_count);

            let mut response = Document::new();
            response.insert("operation", Bson::String(String::from(tr("deleteMany"))));
            response.insert("deletedCount", deleted_bson);

            Ok(QueryResult::SingleDocument { document: response })
        }
        QueryOperation::ReplaceOne { filter, replacement, options } => {
            let mut action = collection.replace_one(filter, replacement);
            if let Some(opts) = options {
                if let Some(write_concern) = opts.write_concern {
                    action = action.write_concern(write_concern);
                }
                if let Some(upsert) = opts.upsert {
                    action = action.upsert(upsert);
                }
                if let Some(collation) = opts.collation {
                    action = action.collation(collation);
                }
                if let Some(hint) = opts.hint {
                    action = action.hint(hint);
                }
                if let Some(bypass) = opts.bypass_document_validation {
                    action = action.bypass_document_validation(bypass);
                }
                if let Some(let_vars) = opts.let_vars {
                    action = action.let_vars(let_vars);
                }
                if let Some(comment) = opts.comment {
                    action = action.comment(comment);
                }
                if let Some(sort) = opts.sort {
                    action = action.sort(sort);
                }
            }

            let result = action.run().map_err(|err| err.to_string())?;

            let mut response = Document::new();
            response.insert("operation", Bson::String(String::from(tr("replaceOne"))));
            response.insert("matchedCount", u64_to_bson(result.matched_count));
            response.insert("modifiedCount", u64_to_bson(result.modified_count));
            if let Some(id) = result.upserted_id {
                response.insert("upsertedId", id);
            }

            Ok(QueryResult::SingleDocument { document: response })
        }
        QueryOperation::FindOneAndUpdate { filter, update, options } => {
            let mut action = match update {
                UpdateModificationsSpec::Document(document) => {
                    collection.find_one_and_update(filter, document)
                }
                UpdateModificationsSpec::Pipeline(pipeline) => {
                    collection.find_one_and_update(filter, pipeline)
                }
            };

            if let Some(mut opts) = options {
                if let Some(write_concern) = opts.write_concern {
                    action = action.write_concern(write_concern);
                }
                if let Some(upsert) = opts.upsert {
                    action = action.upsert(upsert);
                }
                if let Some(array_filters) = opts.array_filters.take() {
                    action = action.array_filters(array_filters);
                }
                if let Some(bypass) = opts.bypass_document_validation {
                    action = action.bypass_document_validation(bypass);
                }
                if let Some(max_time) = opts.max_time {
                    action = action.max_time(max_time);
                }
                if let Some(projection) = opts.projection.take() {
                    action = action.projection(projection);
                }
                if let Some(return_document) = opts.return_document {
                    action = action.return_document(return_document);
                }
                if let Some(sort) = opts.sort.take() {
                    action = action.sort(sort);
                }
                if let Some(collation) = opts.collation {
                    action = action.collation(collation);
                }
                if let Some(hint) = opts.hint {
                    action = action.hint(hint);
                }
                if let Some(let_vars) = opts.let_vars.take() {
                    action = action.let_vars(let_vars);
                }
                if let Some(comment) = opts.comment {
                    action = action.comment(comment);
                }
            }

            if let Some(timeout) = timeout {
                action = action.max_time(timeout);
            }

            let result = action.run().map_err(|err| err.to_string())?;
            match result {
                Some(document) => Ok(QueryResult::SingleDocument { document }),
                None => Ok(QueryResult::Documents(Vec::new())),
            }
        }
        QueryOperation::FindOneAndReplace { filter, replacement, options } => {
            let mut action = collection.find_one_and_replace(filter, replacement);

            if let Some(mut opts) = options {
                if let Some(write_concern) = opts.write_concern {
                    action = action.write_concern(write_concern);
                }
                if let Some(upsert) = opts.upsert {
                    action = action.upsert(upsert);
                }
                if let Some(bypass) = opts.bypass_document_validation {
                    action = action.bypass_document_validation(bypass);
                }
                if let Some(max_time) = opts.max_time {
                    action = action.max_time(max_time);
                }
                if let Some(projection) = opts.projection.take() {
                    action = action.projection(projection);
                }
                if let Some(return_document) = opts.return_document {
                    action = action.return_document(return_document);
                }
                if let Some(sort) = opts.sort.take() {
                    action = action.sort(sort);
                }
                if let Some(collation) = opts.collation {
                    action = action.collation(collation);
                }
                if let Some(hint) = opts.hint {
                    action = action.hint(hint);
                }
                if let Some(let_vars) = opts.let_vars.take() {
                    action = action.let_vars(let_vars);
                }
                if let Some(comment) = opts.comment {
                    action = action.comment(comment);
                }
            }

            if let Some(timeout) = timeout {
                action = action.max_time(timeout);
            }

            let result = action.run().map_err(|err| err.to_string())?;
            match result {
                Some(document) => Ok(QueryResult::SingleDocument { document }),
                None => Ok(QueryResult::Documents(Vec::new())),
            }
        }
        QueryOperation::FindOneAndDelete { filter, options } => {
            let mut action = collection.find_one_and_delete(filter);

            if let Some(mut opts) = options {
                if let Some(write_concern) = opts.write_concern {
                    action = action.write_concern(write_concern);
                }
                if let Some(max_time) = opts.max_time {
                    action = action.max_time(max_time);
                }
                if let Some(projection) = opts.projection.take() {
                    action = action.projection(projection);
                }
                if let Some(sort) = opts.sort.take() {
                    action = action.sort(sort);
                }
                if let Some(collation) = opts.collation {
                    action = action.collation(collation);
                }
                if let Some(hint) = opts.hint {
                    action = action.hint(hint);
                }
                if let Some(let_vars) = opts.let_vars.take() {
                    action = action.let_vars(let_vars);
                }
                if let Some(comment) = opts.comment {
                    action = action.comment(comment);
                }
            }

            if let Some(timeout) = timeout {
                action = action.max_time(timeout);
            }

            let result = action.run().map_err(|err| err.to_string())?;
            match result {
                Some(document) => Ok(QueryResult::SingleDocument { document }),
                None => Ok(QueryResult::Documents(Vec::new())),
            }
        }
        QueryOperation::ListIndexes => {
            let mut action = collection.list_indexes();
            if let Some(timeout) = timeout {
                action = action.max_time(timeout);
            }

            let cursor = action.run().map_err(|err| err.to_string())?;
            let mut documents = Vec::new();
            for result in cursor {
                let model = result.map_err(|err| err.to_string())?;
                let document = bson::to_document(&model)
                    .map_err(|error| format!("BSON conversion error: {error}"))?;
                documents.push(Bson::Document(document));
            }
            Ok(QueryResult::Indexes(documents))
        }
        QueryOperation::DatabaseCommand { db, command } => {
            let database = client.database(&db);
            let action = database.run_command(command);
            let document = action.run().map_err(|err| err.to_string())?;
            Ok(QueryResult::SingleDocument { document })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::bson::doc;
    use serde_json::json;

    fn parse(query: &str) -> QueryOperation {
        parse_collection_query("testdb", "users", query).expect("query should parse")
    }

    #[test]
    fn parses_simple_find_query() {
        let operation = parse("db.users.find({ \"name\": \"Alice\" })");
        match operation {
            QueryOperation::Find { filter } => assert_eq!(filter, doc! { "name": "Alice" }),
            other => panic!("unexpected operation: {:?}", other),
        }
    }

    #[test]
    fn parses_count_documents_with_options() {
        let operation = parse(
            "db.users.countDocuments({ \"status\": \"active\" }, { \"limit\": 5, \"hint\": { \"name\": \"status_1\" } })",
        );
        match operation {
            QueryOperation::CountDocuments { filter, options } => {
                assert_eq!(filter, doc! { "status": "active" });
                let parsed = options.expect("options expected");
                assert_eq!(parsed.limit, Some(5));
                match parsed.hint {
                    Some(Hint::Keys(doc)) => assert_eq!(doc, doc! { "name": "status_1" }),
                    other => panic!("unexpected hint: {:?}", other),
                }
                assert!(parsed.skip.is_none());
            }
            other => panic!("unexpected operation: {:?}", other),
        }
    }

    #[test]
    fn parses_update_pipeline() {
        let operation = parse(
            "db.users.updateOne({ \"name\": \"Bob\" }, [ { \"$set\": { \"age\": 42 } }, { \"$unset\": \"temp\" } ])",
        );
        match operation {
            QueryOperation::UpdateOne { filter, update, options } => {
                assert_eq!(filter, doc! { "name": "Bob" });
                let pipeline = match update {
                    UpdateModificationsSpec::Pipeline(docs) => docs,
                    other => panic!("expected pipeline, got {:?}", other),
                };
                assert_eq!(
                    pipeline,
                    vec![doc! { "$set": { "age": 42i64 } }, doc! { "$unset": "temp" }]
                );
                assert!(options.is_none());
            }
            other => panic!("unexpected operation: {:?}", other),
        }
    }

    #[test]
    fn parses_shell_bson_helpers() {
        let oid = QueryParser::parse_shell_bson_value("ObjectId(\"64d2f9f18d964a7848d35300\")")
            .expect("valid object id");
        assert_eq!(oid, Bson::ObjectId(ObjectId::from_str("64d2f9f18d964a7848d35300").unwrap()));

        let date = QueryParser::parse_shell_bson_value("ISODate(\"2024-03-01T12:30:00Z\")")
            .expect("valid ISO date");
        match date {
            Bson::DateTime(dt) => {
                assert_eq!(dt, DateTime::parse_rfc3339_str("2024-03-01T12:30:00Z").unwrap())
            }
            other => panic!("expected datetime, got {:?}", other),
        }

        let number_long =
            QueryParser::parse_shell_bson_value("NumberLong(42)").expect("valid NumberLong");
        match number_long {
            Bson::Int64(value) => assert_eq!(value, 42),
            Bson::Int32(value) => assert_eq!(value, 42),
            other => panic!("expected integer, got {:?}", other),
        }
    }

    #[test]
    fn parses_insert_one_with_options() {
        let operation = parse(
            "db.users.insertOne({ \"name\": \"Zoe\" }, { \"writeConcern\": { \"w\": 2, \"j\": true, \"wtimeout\": 500 } })",
        );

        match operation {
            QueryOperation::InsertOne { document, options } => {
                assert_eq!(document, doc! { "name": "Zoe" });

                let opts = options.expect("options expected");
                let write_concern = opts.write_concern.expect("write concern parsed");
                match write_concern.w {
                    Some(Acknowledgment::Nodes(nodes)) => assert_eq!(nodes, 2),
                    other => panic!("unexpected acknowledgment: {:?}", other),
                }
                assert_eq!(write_concern.journal, Some(true));
                assert_eq!(write_concern.w_timeout, Some(Duration::from_millis(500)));
            }
            other => panic!("unexpected operation: {:?}", other),
        }
    }

    #[test]
    fn parse_update_options_supports_multiple_fields() {
        let source = r#"{
            "writeConcern": { "w": "majority" },
            "upsert": true,
            "arrayFilters": [ { "score": { "$gt": 5 } } ],
            "collation": { "locale": "en" },
            "hint": { "score": -1 },
            "bypassDocumentValidation": false,
            "let": { "threshold": 10 },
            "comment": "touch",
            "sort": { "score": -1 }
        }"#;

        let options = QueryParser::parse_update_options(source)
            .expect("should parse")
            .expect("options expected");

        let write_concern = options.write_concern.expect("write concern");
        assert!(matches!(write_concern.w, Some(Acknowledgment::Majority)));
        assert_eq!(options.upsert, Some(true));
        assert_eq!(options.array_filters, Some(vec![doc! { "score": { "$gt": 5i64 } }]));
        let collation = options.collation.expect("collation expected");
        assert_eq!(collation.locale, "en");
        assert_eq!(options.hint, Some(Hint::Keys(doc! { "score": -1i64 })));
        assert_eq!(options.bypass_document_validation, Some(false));
        assert_eq!(options.let_vars, Some(doc! { "threshold": 10i64 }));
        assert_eq!(options.comment, Some(Bson::String("touch".to_string())));
        assert_eq!(options.sort, Some(doc! { "score": -1i64 }));
    }

    #[test]
    fn parse_update_options_rejects_unknown_fields() {
        assert!(QueryParser::parse_update_options("{ \"unexpected\": true }").is_err());
    }

    #[test]
    fn parse_find_one_and_update_options_reads_all_supported_fields() {
        let source = r#"{
            "writeConcern": { "w": 1 },
            "upsert": false,
            "arrayFilters": [ { "elem.status": { "$ne": "done" } } ],
            "bypassDocumentValidation": true,
            "maxTimeMS": 1500,
            "projection": { "name": 1 },
            "returnDocument": "after",
            "sort": { "age": -1 },
            "collation": { "locale": "fr" },
            "hint": "age_1",
            "let": { "var": 1 },
            "comment": { "note": "keep" }
        }"#;

        let options = QueryParser::parse_find_one_and_update_options(source)
            .expect("should parse")
            .expect("options expected");

        let write_concern = options.write_concern.expect("writeConcern expected");
        assert!(matches!(write_concern.w, Some(Acknowledgment::Nodes(1))));
        assert_eq!(options.upsert, Some(false));
        assert_eq!(options.array_filters, Some(vec![doc! { "elem.status": { "$ne": "done" } }]));
        assert_eq!(options.bypass_document_validation, Some(true));
        assert_eq!(options.max_time, Some(Duration::from_millis(1500)));
        assert_eq!(options.projection, Some(doc! { "name": 1i64 }));
        assert!(matches!(options.return_document, Some(ReturnDocument::After)));
        assert_eq!(options.sort, Some(doc! { "age": -1i64 }));
        let collation = options.collation.unwrap();
        assert_eq!(collation.locale, "fr");
        assert_eq!(options.hint, Some(Hint::Name("age_1".to_string())));
        assert_eq!(options.let_vars, Some(doc! { "var": 1i64 }));
        assert_eq!(options.comment, Some(Bson::Document(doc! { "note": "keep" })));
    }

    #[test]
    fn parse_write_concern_rejects_invalid_types() {
        let value = json!({ "w": true });
        assert!(QueryParser::parse_write_concern_value(&value).is_err());
    }

    #[test]
    fn parses_database_stats_with_numeric_scale() {
        let parser = QueryParser { db_name: "analytics", collection: "ignored" };
        let operation = parser.parse_query("db.stats(2048)").expect("stats should parse");

        match operation {
            QueryOperation::DatabaseCommand { db, command } => {
                assert_eq!(db, "analytics");
                assert_eq!(command.get_i32("dbStats"), Ok(1));
                match command.get("scale") {
                    Some(Bson::Int32(value)) => assert_eq!(*value, 2048),
                    Some(Bson::Int64(value)) => assert_eq!(*value, 2048),
                    Some(Bson::Double(value)) => assert_eq!(*value, 2048.0),
                    other => panic!("unexpected scale representation: {:?}", other),
                }
            }
            other => panic!("unexpected operation: {:?}", other),
        }
    }
}
