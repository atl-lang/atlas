use super::super::*;
// ============================================================================

#[test]
fn test_config_parse_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    std::fs::write(&config_path, "{\"host\": \"localhost\", \"port\": 8080}").unwrap();

    let code = format!(
        r#"
        let configStr: string = read_file("{}");
        let config: json = parse_json(configStr)?;
        let host: string = config["host"].as_string();
        host
    "#,
        path_for_atlas(&config_path)
    );
    assert_eval_string_with_io(&code, "localhost");
}

#[test]
fn test_config_extract_port() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    std::fs::write(&config_path, "{\"port\": 3000}").unwrap();

    let code = format!(
        r#"
        let configStr: string = read_file("{}");
        let config: json = parse_json(configStr)?;
        let port: number = config["port"].as_number();
        port
    "#,
        path_for_atlas(&config_path)
    );
    assert_eval_number_with_io(&code, 3000.0);
}

#[test]
fn test_config_validate_required_fields() {
    let code = r#"
        let configStr: string = "{\"host\": \"localhost\", \"port\": 8080}";
        let config: json = parse_json(configStr)?;
        let hasHost: bool = !json_is_null(config["host"]);
        let hasPort: bool = !json_is_null(config["port"]);
        hasHost && hasPort
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_config_missing_field_default() {
    let code = r#"
        let configStr: string = "{\"host\": \"localhost\"}";
        let config: json = parse_json(configStr)?;
        let port: json = config["port"];
        let mut portValue: number = 8080.0;
        if (!json_is_null(port)) {
            portValue = port.as_number();
        }
        portValue
    "#;
    assert_eval_number_with_io(code, 8080.0);
}

#[test]
fn test_config_nested_settings() {
    let code = r#"
        let configStr: string = "{\"database\": {\"host\": \"db.local\", \"port\": 5432}}";
        let config: json = parse_json(configStr)?;
        let db: json = config["database"];
        let dbHost: string = db["host"].as_string();
        dbHost
    "#;
    assert_eval_string_with_io(code, "db.local");
}

#[test]
fn test_config_boolean_flags() {
    let code = r#"
        let configStr: string = "{\"debug\": true, \"production\": false}";
        let config: json = parse_json(configStr)?;
        let debug: bool = config["debug"].as_bool();
        let prod: bool = config["production"].as_bool();
        debug && !prod
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_config_array_values() {
    let code = r#"
        let configStr: string = "{\"allowed_hosts\": [\"localhost\", \"127.0.0.1\"]}";
        let config: json = parse_json(configStr)?;
        let hosts: json = config["allowed_hosts"];
        let first: string = hosts[0].as_string();
        first
    "#;
    assert_eval_string_with_io(code, "localhost");
}

#[test]
fn test_config_write_updated() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    std::fs::write(&config_path, "{\"version\": 1}").unwrap();

    let code = format!(
        r#"
        let configStr: string = read_file("{}");
        let config: json = parse_json(configStr)?;
        let version: number = config["version"].as_number();
        let newVersion: number = version + 1.0;
        let updated: string = "{{\"version\":" + str(newVersion) + "}}";
        write_file("{}", updated);

        let result: string = read_file("{}");
        let newConfig: json = parse_json(result)?;
        let finalVersion: number = newConfig["version"].as_number();
        finalVersion
    "#,
        path_for_atlas(&config_path),
        path_for_atlas(&config_path),
        path_for_atlas(&config_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_config_merge_defaults() {
    let code = r#"
        let userConfig: string = "{\"host\": \"custom.com\"}";
        let defaults: string = "{\"host\": \"localhost\", \"port\": 8080, \"debug\": false}";

        let user: json = parse_json(userConfig)?;
        let def: json = parse_json(defaults)?;

        let hostUser: json = user["host"];
        let portUser: json = user["port"];

        let mut finalHost: string = user["host"].as_string();
        if (json_is_null(hostUser)) {
            finalHost = def["host"].as_string();
        }

        let mut finalPort: number = def["port"].as_number();
        if (!json_is_null(portUser)) {
            finalPort = user["port"].as_number();
        }

        finalHost + ":" + str(finalPort)
    "#;
    assert_eval_string_with_io(code, "custom.com:8080");
}

#[test]
fn test_config_prettify_for_humans() {
    let code = r#"
        let compact: string = "{\"host\":\"localhost\",\"port\":8080}";
        let pretty: string = prettify_json(compact, 2.0);
        includes(pretty, "\n") && includes(pretty, "  ")
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_config_array_length() {
    let code = r#"
        let configStr: string = "{\"servers\": [\"server1\", \"server2\", \"server3\"]}";
        let config: json = parse_json(configStr)?;
        let servers: json = config["servers"];
        let s0: string = servers[0].as_string();
        let s1: string = servers[1].as_string();
        let s2: string = servers[2].as_string();
        len(s0) > 0.0 && len(s1) > 0.0 && len(s2) > 0.0
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_config_environment_specific() {
    let code = r#"
        let configStr: string = "{\"env\": \"production\", \"debug\": false}";
        let config: json = parse_json(configStr)?;
        let env: string = config["env"].as_string();
        let debug: bool = config["debug"].as_bool();
        let isProd: bool = env == "production";
        isProd && !debug
    "#;
    assert_eval_bool_with_io(code, true);
}
