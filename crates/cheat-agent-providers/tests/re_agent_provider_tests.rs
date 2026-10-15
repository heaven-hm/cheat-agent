use cheat_agent_core::CheatError;
use cheat_agent_providers::ReAgentProvider;
use serde_json::Value;

#[tokio::test]
async fn test_re_agent_provider_lifecycle_and_unattached_fails_closed() {
    let provider = ReAgentProvider::new();

    // Inspect before attach fails with NotConnected
    let res = provider.inspect("req_1");
    assert!(matches!(res, Err(CheatError::NotConnected)));

    // Regions before attach fails with NotConnected
    let res = provider.query_regions("req_2");
    assert!(matches!(res, Err(CheatError::NotConnected)));

    // Read before attach fails with NotConnected
    let res = provider.read_memory("req_3", 0x1000, 4);
    assert!(matches!(res, Err(CheatError::NotConnected)));

    // Write preview before attach fails with NotConnected
    let res = provider.write_preview("req_4", 0x1000, &[0x00], &[0x01]);
    assert!(matches!(res, Err(CheatError::NotConnected)));

    // Write commit before attach fails with NotConnected
    let res = provider.write_commit("req_5", 0x1000, &[0x00], &[0x01], true);
    assert!(matches!(res, Err(CheatError::NotConnected)));

    // Attach to nonexistent PID fails
    let res = provider.attach("req_attach_invalid", 9999999, None);
    assert!(matches!(res, Err(CheatError::NotConnected)));
}

#[tokio::test]
async fn test_re_agent_provider_evidence_envelope_schema_conformance() {
    let provider = ReAgentProvider::new();

    let self_exe = std::env::current_exe().expect("current test executable path");
    let test_game_str = self_exe.to_str().expect("valid path string");

    let envelope = provider.verify_trainer("req_verify_1", test_game_str)
        .expect("verify_trainer should succeed");

    assert_eq!(envelope.request_id, "req_verify_1");
    assert_eq!(envelope.operation, "trainer.verify");
    assert_eq!(envelope.status, "success");
    assert!(!envelope.limitations.is_empty());

    // Serialize to JSON and check required schema keys
    let val: Value = serde_json::to_value(&envelope).expect("serialization to JSON");
    assert!(val.get("request_id").is_some());
    assert!(val.get("timestamp").is_some());
    assert!(val.get("operation").is_some());
    assert!(val.get("status").is_some());
    assert!(val.get("observations").is_some());
    assert!(val.get("artifacts").is_some());
    assert!(val.get("limitations").is_some());

    // Check non-existent trainer
    let envelope_err = provider.verify_trainer("req_verify_2", "nonexistent_trainer_path.exe")
        .expect("verify_trainer should return envelope even if missing");
    assert_eq!(envelope_err.status, "error");
    assert_eq!(envelope_err.observations["exists"], false);
}
