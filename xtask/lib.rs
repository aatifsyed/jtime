#![cfg(test)]

use anyhow::Context as _;
use expect_test::expect_file;
use itertools::Itertools as _;
use schemars::schema::{
    InstanceType, Metadata, ObjectValidation, Schema, SchemaObject, SingleOrVec,
};
use serde::Deserialize;

#[test]
fn schema_json() {
    let json = json_schema(true);
    expect_file!["../schema.json"].assert_eq(&json);
}

fn json_schema(strict: bool) -> String {
    let mut validation = ObjectValidation::default();
    for Row {
        key,
        ty,
        spec: _,
        description,
    } in rows()
    {
        let clobbered = validation.properties.insert(
            key,
            Schema::Object(SchemaObject {
                metadata: Some(Box::new(Metadata {
                    description: Some(description),
                    ..Default::default()
                })),
                instance_type: Some(SingleOrVec::Single(Box::new(match ty {
                    Type::String => InstanceType::String,
                    Type::Number => InstanceType::Number,
                }))),
                ..Default::default()
            }),
        );
        assert!(clobbered.is_none())
    }
    if strict {
        validation.required = validation.properties.keys().cloned().collect();
    }
    let root = Schema::Object(SchemaObject {
        metadata: Some(Box::new(Metadata {
            description: Some(String::from("outputs from GNU time")),
            ..Default::default()
        })),
        instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
        object: Some(Box::new(validation)),
        ..Default::default()
    });

    serde_json::to_string_pretty(&root).unwrap()
}

#[test]
fn format_string() {
    let inner = rows()
        .into_iter()
        .map(
            |Row {
                 key,
                 ty,
                 spec,
                 description: _,
             }| {
                match ty {
                    Type::String => format!(r#""{key}":"%{spec}""#),
                    Type::Number => format!(r#""{key}":%{spec}"#),
                }
            },
        )
        .join(",");
    let outer = format!("{{{inner}}}");
    expect_file!["../format"].assert_eq(&outer);
}

fn rows() -> Vec<Row> {
    csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(include_str!("spec.tsv").as_bytes())
        .deserialize::<Row>()
        .enumerate()
        .map(|(ix, row)| row.context(format!("couldn't deserialize row at index {ix}")))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

#[derive(Deserialize, Debug)]
struct Row {
    key: String,
    #[serde(rename = "type")]
    ty: Type,
    spec: char,
    description: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Type {
    String,
    Number,
}
