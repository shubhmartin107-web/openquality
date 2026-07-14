use regex::Regex;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ColumnLineage {
    pub table: String,
    pub column: String,
    pub source_table: String,
    pub source_column: String,
    pub transformation: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TableLineage {
    pub table: String,
    pub upstream_tables: Vec<String>,
    pub downstream_tables: Vec<String>,
    pub column_lineage: Vec<ColumnLineage>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SqlStatement {
    pub source_tables: Vec<String>,
    pub target_table: Option<String>,
    pub column_mappings: Vec<ColumnMapping>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ColumnMapping {
    pub target_column: String,
    pub source_columns: Vec<(String, String)>,
    pub expression: Option<String>,
}

pub struct LineageParser {
    table_ref_re: Regex,
    create_table_re: Regex,
    insert_into_re: Regex,
    select_cols_re: Regex,
    as_alias_re: Regex,
    _from_table_re: Regex,
    _join_table_re: Regex,
    _cte_re: Regex,
}

impl LineageParser {
    pub fn new() -> Self {
        Self {
            table_ref_re: Regex::new(r#"(?i)(?:from|join)\s+([a-z_][a-z0-9_.]*)"#).unwrap(),
            create_table_re: Regex::new(r#"(?i)create\s+(?:or\s+replace\s+)?(?:temp\s+)?table\s+(?:if\s+not\s+exists\s+)?([a-z_][a-z0-9_.]*)"#).unwrap(),
            insert_into_re: Regex::new(r#"(?i)insert\s+(?:overwrite\s+)?into\s+([a-z_][a-z0-9_.]*)"#).unwrap(),
            select_cols_re: Regex::new(r#"(?i)select\s+(.+?)(\s+(?:from|where|group|order|limit|having)\s|;|$)"#).unwrap(),
            as_alias_re: Regex::new(r#"(?i)([a-z_][a-z0-9_.]*)\s+as\s+([a-z_][a-z0-9_]*)(?:\s|,|$)"#).unwrap(),
            _from_table_re: Regex::new(r#"(?i)from\s+([a-z_][a-z0-9_.]*)"#).unwrap(),
            _join_table_re: Regex::new(r#"(?i)join\s+([a-z_][a-z0-9_.]*)(?:\s+(?:as\s+)?([a-z_][a-z0-9_]*))?"#).unwrap(),
            _cte_re: Regex::new(r#"(?i)with\s+([a-z_][a-z0-9_]*)\s+as\s*\("#).unwrap(),
        }
    }

    pub fn parse_sql(&self, sql: &str) -> SqlStatement {
        let target_table = self
            .create_table_re
            .captures(sql)
            .or_else(|| self.insert_into_re.captures(sql))
            .map(|c| c[1].to_string());

        let source_tables = self.extract_tables(sql);
        let column_mappings = self.extract_column_mappings(sql);

        SqlStatement {
            source_tables,
            target_table,
            column_mappings,
        }
    }

    fn extract_tables(&self, sql: &str) -> Vec<String> {
        let mut tables = Vec::new();
        for cap in self.table_ref_re.captures_iter(sql) {
            let table = cap[1].to_string();
            if !tables.contains(&table) {
                tables.push(table);
            }
        }
        tables
    }

    fn extract_column_mappings(&self, sql: &str) -> Vec<ColumnMapping> {
        let mut mappings = Vec::new();

        if let Some(select_cap) = self.select_cols_re.captures(sql) {
            let select_body = select_cap[1].to_string();

            for part in select_body.split(',') {
                let part = part.trim();
                if part.is_empty() || part.eq_ignore_ascii_case("*") {
                    continue;
                }

                if let Some(as_cap) = self.as_alias_re.captures(part) {
                    let source = as_cap[1].to_string();
                    let alias = as_cap[2].to_string();
                    let source_parts: Vec<&str> = source.rsplitn(2, '.').collect();
                    if source_parts.len() == 2 {
                        mappings.push(ColumnMapping {
                            target_column: alias,
                            source_columns: vec![(
                                source_parts[1].to_string(),
                                source_parts[0].to_string(),
                            )],
                            expression: Some(part.to_string()),
                        });
                    } else {
                        mappings.push(ColumnMapping {
                            target_column: alias,
                            source_columns: vec![],
                            expression: Some(source.to_string()),
                        });
                    }
                } else {
                    let col = part.trim().to_string();
                    let col_parts: Vec<&str> = col.rsplitn(2, '.').collect();
                    let target = if col_parts.len() == 2 {
                        col_parts[0].to_string()
                    } else {
                        col.clone()
                    };
                    mappings.push(ColumnMapping {
                        target_column: target,
                        source_columns: if col_parts.len() == 2 {
                            vec![(col_parts[1].to_string(), col_parts[0].to_string())]
                        } else {
                            vec![]
                        },
                        expression: Some(col),
                    });
                }
            }
        }

        mappings
    }

    pub fn build_table_lineage(&self, statements: &[SqlStatement]) -> Vec<TableLineage> {
        let mut upstream: HashMap<String, HashSet<String>> = HashMap::new();
        let mut column_lineages: Vec<ColumnLineage> = Vec::new();

        for stmt in statements {
            if let Some(ref target) = stmt.target_table {
                let entry = upstream.entry(target.clone()).or_default();
                for src in &stmt.source_tables {
                    entry.insert(src.clone());
                }
                for mapping in &stmt.column_mappings {
                    for (src_table, src_col) in &mapping.source_columns {
                        column_lineages.push(ColumnLineage {
                            table: target.clone(),
                            column: mapping.target_column.clone(),
                            source_table: src_table.clone(),
                            source_column: src_col.clone(),
                            transformation: mapping.expression.clone(),
                        });
                    }
                }
            }
        }

        let mut downstream: HashMap<String, HashSet<String>> = HashMap::new();
        for (target, sources) in &upstream {
            for source in sources {
                downstream
                    .entry(source.clone())
                    .or_default()
                    .insert(target.clone());
            }
        }

        let mut all_tables: HashSet<String> = HashSet::new();
        for stmt in statements {
            if let Some(ref target) = stmt.target_table {
                all_tables.insert(target.clone());
            }
            for src in &stmt.source_tables {
                all_tables.insert(src.clone());
            }
        }

        all_tables
            .into_iter()
            .map(|table| TableLineage {
                upstream_tables: upstream
                    .get(&table)
                    .map(|s| {
                        let mut v: Vec<_> = s.iter().cloned().collect();
                        v.sort();
                        v
                    })
                    .unwrap_or_default(),
                downstream_tables: downstream
                    .get(&table)
                    .map(|s| {
                        let mut v: Vec<_> = s.iter().cloned().collect();
                        v.sort();
                        v
                    })
                    .unwrap_or_default(),
                column_lineage: column_lineages
                    .iter()
                    .filter(|c| c.table == table)
                    .cloned()
                    .collect(),
                table,
            })
            .collect()
    }
}

impl Default for LineageParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_table_as_select() {
        let sql = "CREATE TABLE analytics.orders AS SELECT o.id, o.amount, u.name AS user_name FROM raw.orders o JOIN raw.users u ON o.user_id = u.id";
        let parser = LineageParser::new();
        let stmt = parser.parse_sql(sql);
        assert_eq!(stmt.target_table, Some("analytics.orders".into()));
        assert!(stmt.source_tables.contains(&"raw.orders".into()));
        assert!(stmt.source_tables.contains(&"raw.users".into()));
    }

    #[test]
    fn test_parse_insert_into() {
        let sql = "INSERT INTO output.daily_sales SELECT date, SUM(amount) as total FROM raw.transactions GROUP BY date";
        let parser = LineageParser::new();
        let stmt = parser.parse_sql(sql);
        assert_eq!(stmt.target_table, Some("output.daily_sales".into()));
        assert!(stmt.source_tables.contains(&"raw.transactions".into()));
    }

    #[test]
    fn test_column_mappings_with_alias() {
        let sql = "CREATE TABLE dest AS SELECT a.col1, b.col2 AS renamed, a.col3 + b.col4 AS combined FROM src1 a JOIN src2 b ON a.id = b.id";
        let parser = LineageParser::new();
        let stmt = parser.parse_sql(sql);
        assert!(!stmt.column_mappings.is_empty());

        let renamed = stmt
            .column_mappings
            .iter()
            .find(|m| m.target_column == "renamed");
        assert!(renamed.is_some());
    }

    #[test]
    fn test_build_table_lineage() {
        let parser = LineageParser::new();
        let stmts = vec![
            parser.parse_sql("CREATE TABLE staging.users AS SELECT id, name FROM raw.users"),
            parser.parse_sql("CREATE TABLE analytics.active_users AS SELECT id, name FROM staging.users WHERE active = true"),
        ];
        let lineage = parser.build_table_lineage(&stmts);
        let active = lineage
            .iter()
            .find(|t| t.table == "analytics.active_users")
            .unwrap();
        assert!(active.upstream_tables.contains(&"staging.users".into()));
        let staging = lineage.iter().find(|t| t.table == "staging.users").unwrap();
        assert!(
            staging
                .downstream_tables
                .contains(&"analytics.active_users".into())
        );
    }

    #[test]
    fn test_parse_select_star() {
        let sql = "CREATE TABLE t AS SELECT * FROM src";
        let parser = LineageParser::new();
        let stmt = parser.parse_sql(sql);
        assert_eq!(stmt.target_table, Some("t".into()));
        assert_eq!(stmt.source_tables, vec!["src"]);
        assert!(stmt.column_mappings.is_empty());
    }

    #[test]
    fn test_parse_cte() {
        let sql = "WITH cte AS (SELECT id FROM raw) CREATE TABLE t AS SELECT * FROM cte";
        let parser = LineageParser::new();
        let stmt = parser.parse_sql(sql);
        assert_eq!(stmt.target_table, Some("t".into()));
        assert!(
            stmt.source_tables.contains(&"raw".into())
                || stmt.source_tables.contains(&"cte".into())
        );
    }
}
