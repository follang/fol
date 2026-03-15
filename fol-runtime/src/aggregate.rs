//! Runtime trait contracts for backend-generated aggregate types.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolNamedValue {
    name: &'static str,
    rendered_value: String,
}

impl FolNamedValue {
    pub fn new(name: &'static str, rendered_value: impl Into<String>) -> Self {
        Self {
            name,
            rendered_value: rendered_value.into(),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn rendered_value(&self) -> &str {
        &self.rendered_value
    }
}

pub trait FolRecord {
    fn fol_record_name(&self) -> &'static str;

    fn fol_record_fields(&self) -> Vec<FolNamedValue>;
}

pub fn module_name() -> &'static str {
    "aggregate"
}

#[cfg(test)]
mod tests {
    use super::{FolNamedValue, FolRecord};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DemoPoint {
        x: i64,
        y: i64,
    }

    impl FolRecord for DemoPoint {
        fn fol_record_name(&self) -> &'static str {
            "Point"
        }

        fn fol_record_fields(&self) -> Vec<FolNamedValue> {
            vec![
                FolNamedValue::new("x", self.x.to_string()),
                FolNamedValue::new("y", self.y.to_string()),
            ]
        }
    }

    #[test]
    fn record_trait_contract_preserves_backend_authored_field_order() {
        let point = DemoPoint { x: 3, y: 7 };
        let fields = point.fol_record_fields();

        assert_eq!(point.fol_record_name(), "Point");
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name(), "x");
        assert_eq!(fields[0].rendered_value(), "3");
        assert_eq!(fields[1].name(), "y");
        assert_eq!(fields[1].rendered_value(), "7");
    }
}
