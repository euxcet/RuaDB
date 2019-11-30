use prettytable::{Table, Row, Cell};

pub struct RuaResult {
    res: Result<(Option<Vec<Vec<String>>>, Option<String>), String>,
}

impl RuaResult {
    pub fn default() -> Self {
        Self {
            res: Ok((None, None))
        }
    }
    pub fn ok(v: Option<Vec<Vec<String>>>, e: String) -> Self {
        Self {
            res: Ok((v, Some(e)))
        }
    }

    pub fn err(e: String) -> Self {
        Self {
            res: Err(e)
        }
    }

    pub fn is_ok(&self) -> bool {
        self.res.is_ok()
    }
    
    pub fn is_err(&self) -> bool {
        self.res.is_err()
    }
}

pub struct RuaLogger {
}

impl RuaLogger {
    pub fn new() -> Self {
        Self {}
    }

    pub fn log(&self, r: &RuaResult) {
        match &r.res {
            Ok(e) => {
                let (t, e) = e;
                if let Some(t) = t {
                    let mut table = Table::new();
                    for r in t {
                        let mut row = Row::default();
                        for c in r {
                            row.add_cell(Cell::new(c.as_str()));
                        }
                        table.add_row(row);
                    }
                    table.printstd();
                }
                if let Some(e) = e {
                    println!("OK, {}", e);
                } else {
                    println!("OK");
                }
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}