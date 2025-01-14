use sqlparser::ast::{AlterTableOperation, ObjectName};

use std::sync::Arc;

use super::Binder;
use crate::binder::{lower_case_name, split_name, BindError};
use crate::planner::operator::alter_table::add_column::AddColumnOperator;
use crate::planner::operator::alter_table::drop_column::DropColumnOperator;
use crate::planner::operator::scan::ScanOperator;
use crate::planner::operator::Operator;
use crate::planner::LogicalPlan;
use crate::storage::Transaction;

impl<'a, T: Transaction> Binder<'a, T> {
    pub(crate) fn bind_alter_table(
        &mut self,
        name: &ObjectName,
        operation: &AlterTableOperation,
    ) -> Result<LogicalPlan, BindError> {
        let table_name: Arc<String> = Arc::new(split_name(&lower_case_name(name))?.1.to_string());

        if let Some(table) = self.context.table(table_name.clone()) {
            let plan = match operation {
                AlterTableOperation::AddColumn {
                    column_keyword: _,
                    if_not_exists,
                    column_def,
                } => {
                    let plan = ScanOperator::build(table_name.clone(), table);

                    LogicalPlan {
                        operator: Operator::AddColumn(AddColumnOperator {
                            table_name,
                            if_not_exists: *if_not_exists,
                            column: self.bind_column(column_def)?,
                        }),
                        childrens: vec![plan],
                    }
                }
                AlterTableOperation::DropColumn {
                    column_name,
                    if_exists,
                    ..
                } => {
                    let plan = ScanOperator::build(table_name.clone(), table);
                    let column_name = column_name.value.clone();

                    LogicalPlan {
                        operator: Operator::DropColumn(DropColumnOperator {
                            table_name,
                            if_exists: *if_exists,
                            column_name,
                        }),
                        childrens: vec![plan],
                    }
                }
                AlterTableOperation::DropPrimaryKey => todo!(),
                AlterTableOperation::RenameColumn {
                    old_column_name: _,
                    new_column_name: _,
                } => todo!(),
                AlterTableOperation::RenameTable { table_name: _ } => todo!(),
                AlterTableOperation::ChangeColumn {
                    old_name: _,
                    new_name: _,
                    data_type: _,
                    options: _,
                } => todo!(),
                AlterTableOperation::AlterColumn {
                    column_name: _,
                    op: _,
                } => todo!(),
                _ => todo!(),
            };

            Ok(plan)
        } else {
            Err(BindError::InvalidTable(format!(
                "not found table {}",
                table_name
            )))
        }
    }
}
