pub type Name = String;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sql {
    pub stmt_list: Vec<Stmt>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stmt {
    System(SystemStmt),
    Database(DatabaseStmt),
    Table(TableStmt),
    Index(IndexStmt),
    Alter(AlterStmt),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemStmt {
    ShowDatabases,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatabaseStmt {
    CreateDatabase {
        db_name: Name,
    },
    DropDatabase {
        db_name: Name,
    },
    UseDatabase {
        db_name: Name,
    },
    ShowTables,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableStmt {
    CreateTable {
        tb_name: Name,
        field_list: Vec<Field>,
    },
    DropTable {
        tb_name: Name,
    },
    Desc {
        tb_name: Name,
    },
    Insert {
        tb_name: Name,
        value_lists: Vec<Vec<Value>>,
    },
    Delete {
        tb_name: Name,
        where_clause: Option<Vec<WhereClause>>,
    },
    Update {
        tb_name: Name,
        set_clause: Vec<SetClause>,
        where_clause: Option<Vec<WhereClause>>,
    },
    Select {
        table_list: Vec<Name>,
        selector: Selector,
        where_clause: Option<Vec<WhereClause>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexStmt {
    CreateIndex {
        idx_name: Name,
        tb_name: Name,
        column_list: Vec<Name>, 
    }, 
    DropIndex {
        idx_name: Name,
    },
    AlterAddIndex {
        idx_name: Name,
        tb_name: Name,
        column_list: Vec<Name>, 
    }, 
    AlterDropIndex {
        idx_name: Name,
        tb_name: Name,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlterStmt {
    AddColumn {
        tb_name: Name,
        field: Field,
    },
    DropColumn {
        tb_name: Name,
        col_name: Name,
    },
    ChangeColumn {
        tb_name: Name,
        col_name: Name,
        field: Field,
    },
    RenameTable {
        tb_name: Name,
        new_name: Name,
    },
    AddPrimaryKey {
        tb_name: Name,
        column_list: Vec<Name>,
    },
    DropPrimaryKey {
        tb_name: Name,
    },
    AddConstraintPrimaryKey {
        tb_name: Name,
        pk_name: Name,
        column_list: Vec<Name>,
    },
    DropConstraintPrimaryKey {
        tb_name: Name,
        pk_name: Name,
    },
    AddConstraintForeignKey {
        tb_name: Name,
        fk_name: Name,
        column_list: Vec<Name>,
        foreign_tb_name: Name,
        foreign_column_list: Vec<Name>,
    },
    DropConstraintForeignKey {
        tb_name: Name,
        fk_name: Name,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Field {
    ColumnField {
        col_name: Name,
        ty: Type,
        not_null: bool,
        default_value: Option<Value>,
    },

    PrimaryKeyField {
        column_list: Vec<Name>,
    },

    ForeignKeyField {
        col_name: Name,
        foreign_tb_name: Name, 
        foreign_col_name: Name,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Int(i64), 
    Varchar(i64),
    Date,
    Float,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    // 1, 2, 3, 4, 5
    Int(String),
    Str(String),
    Float(String),
    Date(String),
    Null,
}

impl Value {
    pub fn is_null(&self) -> bool {
        *self == Value::Null
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WhereClause {
    IsAssert {
        col: Column,
        null: bool,
    },
    Comparison {
        col: Column,
        op: Op,
        expr: Expr,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    pub tb_name: Option<Name>,
    pub col_name: Name,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Value(Value),
    Column(Column),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    Equal,
    NotEqual,
    LessEqual,
    GreaterEqual,
    Less,
    Greater,
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetClause {
    pub col_name: Name,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selector {
    All,
    Columns(Vec<Column>),
}