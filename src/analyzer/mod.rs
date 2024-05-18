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

- What about situations like..

    table t1 (
        rec1 ()
    )
    table t2 (
        (col t1.rec1.col)
        (col "t1".rec1."col")
    )
*/
pub mod error;

use crate::parser::nodes::*;
use error::*;
use std::collections::{HashMap, HashSet};

pub type AnalyzeResult = Result<ValidatedParseTree, AnalyzeError>;

pub struct ValidatedParseTree(ParseTree);

impl ValidatedParseTree {
    pub fn into_inner(self) -> ParseTree {
        self.0
    }
}

// Map of possibly duplicate record names to the scopes they are defined in
#[derive(Debug, Default)]
struct RefSet(HashMap<String, HashSet<String>>);

impl RefSet {
    /// Attempts to add the record with the given scope to the set.
    /// Returns true if there was not already a record with the same name & scope
    /// or returns false if there was.
    fn insert_record_scope(&mut self, scope: &str, record_name: &str) -> bool {
        self.0
            .entry(record_name.to_owned())
            .or_insert_with(HashSet::new)
            .insert(scope.to_owned())
    }

    fn get_record_scopes(&self, record_name: &str) -> Option<&HashSet<String>> {
        self.0.get(record_name)
    }

    fn has_record_scope(&self, record_name: &str, scope: &str) -> bool {
        self.get_record_scopes(record_name)
            .map(|scopes| scopes.contains(scope))
            .unwrap_or(false)
    }
}

pub fn analyze(parse_tree: ParseTree) -> AnalyzeResult {
    let mut refset = RefSet::default();

    for node in &parse_tree.nodes {
        match node {
            StructuralNode::Schema(schema) => {
                for table in &schema.nodes {
                    analyze_table(Some(schema), table, &mut refset)?;
                }
            }
            StructuralNode::Table(table) => {
                analyze_table(None, table, &mut refset)?;
            }
        }
    }

    Ok(ValidatedParseTree(parse_tree))
}

fn analyze_table(
    schema: Option<&SchemaNode>,
    table: &TableNode,
    refset: &mut RefSet,
) -> Result<(), AnalyzeError> {
    // TODO: This is mostly copy-pasta
    let table_scope = {
        let scope = table
            .identity
            .alias
            .as_ref()
            .unwrap_or(&table.identity.name);

        match schema {
            Some(schema) => format!(
                "{}.{}",
                schema
                    .identity
                    .alias
                    .as_ref()
                    .unwrap_or(&schema.identity.name),
                scope,
            ),
            None => scope.to_owned(),
        }
    };
    for record in &table.nodes {
        analyze_record(record, refset, &table_scope)?;

        if let Some(name) = &record.name {
            if !refset.insert_record_scope(&table_scope, name) {
                return Err(AnalyzeError {
                    kind: AnalyzeErrorKind::DuplicateRecord {
                        record: name.clone(),
                    },
                    position: record.position,
                });
            }
        }
    }

    Ok(())
}

fn analyze_record(
    record: &RecordNode,
    refset: &RefSet,
    parent_scope: &str,
) -> Result<(), AnalyzeError> {
    let mut attrnames = HashSet::new();

    for attr in &record.nodes {
        if !attrnames.insert(&attr.name) {
            return Err(AnalyzeError {
                kind: AnalyzeErrorKind::DuplicateColumn {
                    column: attr.name.clone(),
                },
                position: attr.position,
            });
        }

        if let Value::Reference(refval) = &attr.value {
            // Column-level references only need validation that the column being referenced
            // is explicitly declared in the record already
            if let Reference::ColumnLevel(c) = refval {
                if !attrnames.contains(&c.column) {
                    return Err(AnalyzeError {
                        kind: AnalyzeErrorKind::ColumnNotFound {
                            column: c.column.clone(),
                        },
                        position: attr.position,
                    });
                }
                continue;
            }

            match refval {
                Reference::SchemaLevel(s) => {
                    let scope = format!("{}.{}", s.schema, s.table);

                    if !refset.has_record_scope(&s.record, &scope) {
                        return Err(AnalyzeError {
                            kind: AnalyzeErrorKind::RecordNotFound {
                                record: format!("{}.{}", scope, s.record),
                            },
                            position: attr.position,
                        });
                    }
                }
                Reference::TableLevel(t) => {
                    if !refset.has_record_scope(&t.record, &t.table) {
                        return Err(AnalyzeError {
                            kind: AnalyzeErrorKind::RecordNotFound {
                                record: format!("{}.{}", t.table, t.record),
                            },
                            position: attr.position,
                        });
                    }
                }
                Reference::RecordLevel(r) => {
                    match refset.get_record_scopes(&r.record) {
                        None => {
                            return Err(AnalyzeError {
                                kind: AnalyzeErrorKind::RecordNotFound {
                                    record: r.record.clone(),
                                },
                                position: attr.position,
                            });
                        }
                        Some(scopes) => {
                            // An empty scopes set at this point indicates a bug in the
                            // analyzer code rather than a result of user input, valid
                            // or otherwise
                            assert!(scopes.len() > 0, "record scopes should not be empty");

                            // If there is only a single record from any scope with a matching
                            // name, then the reference is valid.
                            //
                            // If there are multiple scopes with a record of the same name,
                            // then an unqualified reference is ambiguous unless the current
                            // table scope has a record of the same name.
                            if scopes.len() > 1 && !scopes.contains(parent_scope) {
                                return Err(AnalyzeError {
                                    kind: AnalyzeErrorKind::AmbiguousRecord {
                                        record: r.record.clone(),
                                    },
                                    position: attr.position,
                                });
                            }
                        }
                    }
                }
                Reference::ColumnLevel(_) => unreachable!(),
            }
        }
    }

    Ok(())
}
