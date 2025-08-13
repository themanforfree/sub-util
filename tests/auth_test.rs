use sub_util::*;

#[test]
fn test_auth_config_disabled_allows_access() {
    let config = AppConfig {
        auth: Some(AuthConfig {
            enabled: false,
            token: Some("some-token".to_string()),
        }),
        ..Default::default()
    };
    
    // When auth is disabled, access should be allowed regardless of token
    assert!(is_access_allowed(&config, None));
    assert!(is_access_allowed(&config, Some("wrong-token".to_string())));
    assert!(is_access_allowed(&config, Some("some-token".to_string())));
}

#[test]
fn test_auth_config_enabled_requires_correct_token() {
    let config = AppConfig {
        auth: Some(AuthConfig {
            enabled: true,
            token: Some("correct-token".to_string()),
        }),
        ..Default::default()
    };
    
    // When auth is enabled, only correct token should be allowed
    assert!(!is_access_allowed(&config, None));
    assert!(!is_access_allowed(&config, Some("wrong-token".to_string())));
    assert!(is_access_allowed(&config, Some("correct-token".to_string())));
}

#[test]
fn test_auth_config_enabled_without_configured_token() {
    let config = AppConfig {
        auth: Some(AuthConfig {
            enabled: true,
            token: None,
        }),
        ..Default::default()
    };
    
    // When auth is enabled but no token is configured, access should be denied
    assert!(!is_access_allowed(&config, None));
    assert!(!is_access_allowed(&config, Some("any-token".to_string())));
}

#[test]
fn test_no_auth_config_allows_access() {
    let config = AppConfig {
        auth: None,
        ..Default::default()
    };
    
    // When no auth config is provided, access should be allowed
    assert!(is_access_allowed(&config, None));
    assert!(is_access_allowed(&config, Some("any-token".to_string())));
}

#[test]
fn test_auth_config_with_empty_token() {
    let config = AppConfig {
        auth: Some(AuthConfig {
            enabled: true,
            token: Some("".to_string()),
        }),
        ..Default::default()
    };
    
    // When auth is enabled with empty token, access should be denied
    assert!(!is_access_allowed(&config, None));
    assert!(!is_access_allowed(&config, Some("".to_string())));
    assert!(!is_access_allowed(&config, Some("any-token".to_string())));
}

// Helper function to simulate the authentication logic
fn is_access_allowed(app_config: &AppConfig, provided_token: Option<String>) -> bool {
    if let Some(auth_config) = &app_config.auth {
        if auth_config.enabled {
            let provided = provided_token.as_deref().unwrap_or("");
            let expected = auth_config.token.as_deref().unwrap_or("");
            
            if expected.is_empty() {
                return false; // Server misconfiguration
            }
            
            if provided.is_empty() {
                return false; // No token provided
            }
            
            return provided == expected;
        }
    }
    // If authentication is not configured or disabled, allow access
    true
}