use crate::DatabaseError;

#[derive(Clone, Copy, Debug)]
pub struct SchemaFile {
    pub path: &'static str,
    pub contents: &'static str,
}

pub const SCHEMA_FILES: &[SchemaFile] = &[
    SchemaFile {
        path: "schema/user/001_user.surql",
        contents: include_str!("../schema/user/001_user.surql"),
    },
    SchemaFile {
        path: "schema/auth/001_account.surql",
        contents: include_str!("../schema/auth/001_account.surql"),
    },
    SchemaFile {
        path: "schema/provider/001_provider_credential.surql",
        contents: include_str!("../schema/provider/001_provider_credential.surql"),
    },
    SchemaFile {
        path: "schema/auth/002_virtual_api_key.surql",
        contents: include_str!("../schema/auth/002_virtual_api_key.surql"),
    },
    SchemaFile {
        path: "schema/model/001_model_catalog.surql",
        contents: include_str!("../schema/model/001_model_catalog.surql"),
    },
    SchemaFile {
        path: "schema/request/001_request_log.surql",
        contents: include_str!("../schema/request/001_request_log.surql"),
    },
];

pub fn validate_schema_files() -> Result<(), DatabaseError> {
    for schema_file in SCHEMA_FILES {
        if schema_file.contents.trim().is_empty() {
            return Err(DatabaseError::SchemaBootstrap(format!(
                "schema file {} is empty",
                schema_file.path
            )));
        }

        if !schema_file
            .contents
            .to_ascii_lowercase()
            .contains("if not exists")
        {
            return Err(DatabaseError::SchemaBootstrap(format!(
                "schema file {} must use IF NOT EXISTS",
                schema_file.path
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{SCHEMA_FILES, validate_schema_files};

    #[test]
    fn schema_files_are_valid() {
        validate_schema_files().expect("schema files must validate");
    }

    #[test]
    fn schema_files_load_in_expected_order() {
        let ordered_paths: Vec<_> = SCHEMA_FILES.iter().map(|file| file.path).collect();

        assert_eq!(
            ordered_paths,
            vec![
                "schema/user/001_user.surql",
                "schema/auth/001_account.surql",
                "schema/provider/001_provider_credential.surql",
                "schema/auth/002_virtual_api_key.surql",
                "schema/model/001_model_catalog.surql",
                "schema/request/001_request_log.surql",
            ]
        );
    }
}
