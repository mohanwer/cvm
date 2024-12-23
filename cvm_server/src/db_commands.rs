// Table Names
pub static TABLE_CLIENTS: &str = "clients";
pub static TABLE_APPLICATION_VERSIONS: &str = "application_versions";

// Queries
pub static QUERY_CLIENT: &str = r#"
    SELECT id, app_id, updated_at, version, enabled, created_at FROM clients WHERE id = $1
"#;

pub static UPDATE_CLIENT: &str = r#"
    UPDATE clients SET version = $1, updated_at=now() WHERE id = $2
"#;

pub static INSERT_CLIENT: &str = r#"
    INSERT INTO clients (app_id, version, build_version)
    VALUES ($1, $2, $3)
    RETURNING id, app_id, created_at, updated_at, build_version, version, enabled;
"#;

pub static DELETE_CLIENT_BY_ID: &str = "DELETE FROM clients WHERE id = $1;";

pub static QUERY_APPLICATION_VERSION: &str = r#"
    SELECT id, app_id, version, latest
    FROM application_versions
    WHERE id = $1
"#;

pub static QUERY_LATEST_BUILD_VERSION: &str = r#"
    select ab.id, app_version_id, build_version, success_count, failed_count, url, disabled
    from application_builds ab
    inner join application_versions av on ab.app_version_id = av.id
    where av.latest = true and ab.build_version = $1 and av.app_id = $2 and ab.disabled = false
"#;

pub static QUERY_APPLICATION_BUILD_VERSION: &str = r#"
    select ab.id, app_version_id, build_version, success_count, failed_count, url, disabled
    from application_builds ab
        inner join application_versions av on ab.app_version_id = av.id
    where av.app_id = $1 and av.version = $2 and ab.build_version = $3 and ab.disabled = false
"#;

pub static INSERT_APPLICATION_VERSION: &str = r#"
    INSERT INTO application_versions (app_id, version, latest)
    VALUES ($1, $2, $3)
    RETURNING id, app_id, version, latest;
 "#;

pub static UPDATE_APPLICATION_BUILD_SUCCESS: &str = r#"
    UPDATE application_builds
    SET success_count = success_count + 1
    WHERE id = $1;
"#;

pub static UPDATE_APPLICATION_BUILD_FAILURE: &str = r#"
    UPDATE application_builds
    SET failed_count = failed_count + 1
    WHERE id = $1;
"#;

pub static INSERT_APPLICATION_BUILD: &str = r#"
    INSERT INTO application_builds (url, build_version, app_version_id)
    VALUES ($1, $2, $3)
    RETURNING id, app_version_id, success_count, failed_count, build_version, url, disabled
"#;

pub static QUERY_APPLICATION_BY_ID: &str = r#"
    SELECT id, name, description, created_at
    FROM applications
    WHERE id = $1
"#;

pub static INSERT_INTO_APPLICATION: &str = r#"
    INSERT INTO applications (name, description)
    VALUES ($1, $2)
    RETURNING id, name, description, created_at
"#;

pub static DELETE_APPLICATION: &str = r#"
    DELETE FROM applications
    WHERE id = $1
"#;

pub static QUERY_ADVISORY_LOCK: &str = "SELECT pg_try_advisory_xact_lock($1);";
pub static QUERY_ADVISORY_UNLOCK: &str = "SELECT pg_try_advisory_xact_lock($1);";