use std::collections::HashSet;
use crate::v3::parser::nodes::*;

mod error;
use error::AnalyzeError;

pub type AnalyzeResult = Result<ValidatedParseTree, AnalyzeError>;

pub struct ValidatedParseTree(ParseTree);

/*
// The schema name, table name, or alias used to define the scope
type ScopeName = String;
type RecordNames = HashSet<String>;
type TableRecordsMap = HashMap<ScopeName, RecordNames>;

#[derive(Default)]
struct RefSet {
    unqualified: TableRecordsMap,
    qualified: HashMap<ScopeName, TableRecordsMap>,
}
*/
type RefSet = HashSet<String>;

pub fn analyze(parse_tree: ParseTree) -> AnalyzeResult {
    let mut refset = RefSet::default();

    for node in &parse_tree.nodes {
        match node {
            StructuralNode::Schema(schema) => {
                for table in &schema.nodes {
                    analyze_table(Some(&schema), table, &mut refset);
                }
            }
            StructuralNode::Table(table) => {
                analyze_table(None, table, &mut refset);
            }
        }
    }

    Ok(ValidatedParseTree(parse_tree))
}

fn analyze_table(schema: Option<&Schema>, table: &Table, refset: &mut RefSet) {
    /*
    let table_records = {
        let table_scope_name = table.alias.as_ref().unwrap_or(&table.name);
        match schema {
            Some(schema) => {
                let schema_scope_name = schema.alias.as_ref().unwrap_or(&schema.name);
                refset.qualified
                    .entry(schema_scope_name.clone())
                    .or_default()
                    .entry(table_scope_name.clone())
                    .or_default()
            }
            None => {
                refset.unqualified
                    .entry(table_scope_name.clone())
                    .or_default()
            }
        }
    };
    */
    let scope_name = match schema {
        Some(schema) => format!(
            "{}.{}",
            schema.alias.as_ref().unwrap_or(&schema.name),
            table.alias.as_ref().unwrap_or(&table.name),
        ),
        None => table.alias.as_ref().unwrap_or(&table.name).to_owned(),
    };

    for record in &table.nodes {
        if let Some(name) = &record.name {
            let key = format!("{}.{}", scope_name, name);

            if !refset.insert(key) {
                panic!("duplicate record in table {}: {}", table.name, name);
            }
        }

        analyze_record(record, &refset, &scope_name);
    }
}

fn analyze_record(record: &Record, refset: &RefSet, parent_scope: &str) {
    let mut attrnames = HashSet::new();

    for attr in &record.nodes {
        if !attrnames.insert(&attr.name) {
            panic!("duplicate column in table {}: {}", parent_scope, attr.name);
        }

        if let Value::Reference(val) = &attr.value {
            // Column-level references only need validation that the column being referenced
            // is explicitly declared in the record already, since they cannot come from the
            // database.
            if val.record.is_none() {
                if !attrnames.contains(&val.column) {
                    panic!("column not found: {}", val.column);
                }
                continue;
            }

            let expected_key = match (val.schema.as_ref(), val.table.as_ref(), val.record.as_ref()) {
                (Some(schema), Some(table), Some(record)) => format!("{}.{}.{}", schema, table, record),
                (None, Some(table), Some(record)) => format!("{}.{}", table, record),
                // Unqualified references to other records are only permitted within the same parent table scope.
                (None, None, Some(record)) => format!("{}.{}", parent_scope, record),
                _ => unreachable!("invalid reference: {:?}", val),
            };

            if !refset.contains(&expected_key) {
                panic!("record not found: {}", expected_key);
            }
        }
    }
}
