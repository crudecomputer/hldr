/*
TODO
- Verify that aliases are not reused across *different* entities
- Update analyzer so that this can't happen:

    - Alias same as another table name
        table t1 ()
        table table1 as t1 ()

    - Different tables have same alias
        table table1 as t ()
        table table2 as t ()
*/
pub mod error;

use std::collections::HashSet;
use crate::parser::nodes::*;
use error::*;


pub type AnalyzeResult = Result<ValidatedParseTree, AnalyzeError>;

pub struct ValidatedParseTree(ParseTree);

impl ValidatedParseTree {
    pub fn into_inner(self) -> ParseTree {
        self.0
    }
}

type RefSet = HashSet<String>;

pub fn analyze(parse_tree: ParseTree) -> AnalyzeResult {
    let mut refset = RefSet::default();

    for node in &parse_tree.nodes {
        match node {
            StructuralNode::Schema(schema) => {
                for table in &schema.nodes {
                    analyze_table(Some(&schema), table, &mut refset)?;
                }
            }
            StructuralNode::Table(table) => {
                analyze_table(None, table, &mut refset)?;
            }
        }
    }

    Ok(ValidatedParseTree(parse_tree))
}

fn analyze_table(schema: Option<&Schema>, table: &Table, refset: &mut RefSet) -> Result<(), AnalyzeError> {
    // TODO: This is mostly copy-pasta
    let table_scope = {
        let scope = table.identity.alias.as_ref().unwrap_or(&table.identity.name);
        match schema {
            Some(schema) => format!(
                "{}.{}",
                schema.identity.alias.as_ref().unwrap_or(&schema.identity.name),
                scope,
            ),
            None => scope.to_owned(),
        }
    };
    for record in &table.nodes {
        analyze_record(record, &refset, &table_scope)?;

        if let Some(name) = &record.name {
            let key = format!("{}.{}", table_scope, name);

            if !refset.insert(key) {
                return Err(AnalyzeError {
                    kind: AnalyzeErrorKind::DuplicateRecord {
                        scope: table_scope,
                        record: name.clone(),
                    },
                });
            }
        }
    }

    Ok(())
}

fn analyze_record(record: &Record, refset: &RefSet, parent_scope: &str) -> Result<(), AnalyzeError> {
    let mut attrnames = HashSet::new();

    for attr in &record.nodes {
        if !attrnames.insert(&attr.name) {
            return Err(AnalyzeError {
                kind: AnalyzeErrorKind::DuplicateColumn {
                    scope: parent_scope.to_owned(),
                    column: attr.name.clone(),
                },
            });
        }

        if let Value::Reference(val) = &attr.value {
            // Column-level references only need validation that the column being referenced
            // is explicitly declared in the record already, since they cannot come from the
            // database.
            if val.record.is_none() {
                if !attrnames.contains(&val.column) {
                    return Err(AnalyzeError {
                        kind: AnalyzeErrorKind::ColumnNotFound {
                            column: val.column.clone(),
                        },
                    });
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
                return Err(AnalyzeError {
                    kind: AnalyzeErrorKind::RecordNotFound {
                        record: expected_key,
                    },
                });
            }
        }
    }

    Ok(())
}
