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

pub trait FolEntry {
    fn fol_entry_name(&self) -> &'static str;

    fn fol_entry_variant_name(&self) -> &'static str;

    fn fol_entry_fields(&self) -> Vec<FolNamedValue>;
}

pub fn module_name() -> &'static str {
    "aggregate"
}

#[cfg(test)]
mod tests {
    use super::{FolEntry, FolNamedValue, FolRecord};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DemoPoint {
        x: i64,
        y: i64,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum DemoStatus {
        Ok { count: i64 },
        Err { label: &'static str },
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

    impl FolEntry for DemoStatus {
        fn fol_entry_name(&self) -> &'static str {
            "Status"
        }

        fn fol_entry_variant_name(&self) -> &'static str {
            match self {
                Self::Ok { .. } => "Ok",
                Self::Err { .. } => "Err",
            }
        }

        fn fol_entry_fields(&self) -> Vec<FolNamedValue> {
            match self {
                Self::Ok { count } => vec![FolNamedValue::new("count", count.to_string())],
                Self::Err { label } => vec![FolNamedValue::new("label", label.to_string())],
            }
        }
    }

    #[test]
    fn entry_trait_contract_preserves_variant_name_and_payload_shape() {
        let ok = DemoStatus::Ok { count: 7 };
        let err = DemoStatus::Err { label: "bad-input" };

        let ok_fields = ok.fol_entry_fields();
        let err_fields = err.fol_entry_fields();

        assert_eq!(ok.fol_entry_name(), "Status");
        assert_eq!(ok.fol_entry_variant_name(), "Ok");
        assert_eq!(ok_fields, vec![FolNamedValue::new("count", "7")]);

        assert_eq!(err.fol_entry_name(), "Status");
        assert_eq!(err.fol_entry_variant_name(), "Err");
        assert_eq!(err_fields, vec![FolNamedValue::new("label", "bad-input")]);
    }
}
