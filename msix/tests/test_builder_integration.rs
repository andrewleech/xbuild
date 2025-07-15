use msix::msix;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_builder_creates_valid_msix() {
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().join("test_integration.msix");
    let exe_path = temp_dir.path().join("app.exe");
    
    // Create dummy files
    fs::write(&exe_path, b"MZ\x90\x00").unwrap(); // Minimal PE header
    
    // Build the MSIX package without icon (to avoid PNG complexity in tests)
    let result = msix(&output_path)
        .identity("com.integration.test", "1.0.0.0", "CN=IntegrationTest")
        .properties("Integration Test App", "Testing full MSIX creation")
        .application("TestApp", "app.exe", "Test Application", "Integration test app")
        .capabilities(vec!["internetClient", "runFullTrust"])
        .default_target_device_family()
        .default_resource()
        .executable(&exe_path)
        .compress(true)
        .build();
    
    // Verify the package was created
    assert!(result.is_ok(), "Failed to build MSIX: {:?}", result);
    assert!(output_path.exists(), "MSIX file was not created");
    assert!(output_path.metadata().unwrap().len() > 0, "MSIX file is empty");
}

#[test]
fn test_builder_with_multiple_assets() {
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().join("test_assets.msix");
    let exe_path = temp_dir.path().join("app.exe");
    
    // Create asset directory structure
    let assets_dir = temp_dir.path().join("assets");
    fs::create_dir(&assets_dir).unwrap();
    fs::write(assets_dir.join("config.json"), b"{}").unwrap();
    fs::write(assets_dir.join("data.txt"), b"test data").unwrap();
    
    
    // Create executable
    fs::write(&exe_path, b"MZ\x90\x00").unwrap();
    
    // Build with assets
    let result = msix(&output_path)
        .identity("com.assets.test", "1.0.0.0", "CN=AssetsTest")
        .properties("Assets Test App", "Testing asset packaging")
        .application("AssetsApp", "app.exe", "Assets Application", "Assets test app")
        .executable(&exe_path)
        .add_file(&assets_dir.join("config.json"), "assets/config.json", xcommon::ZipFileOptions::Compressed)
        .add_file(&assets_dir.join("data.txt"), "assets/data.txt", xcommon::ZipFileOptions::Compressed)
        .build();
    
    assert!(result.is_ok(), "Failed to build MSIX with assets: {:?}", result);
    assert!(output_path.exists());
}

#[test]
fn test_builder_error_handling() {
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().join("test_error.msix");
    let missing_exe = temp_dir.path().join("missing.exe");
    
    // Try to build with missing executable
    let result = msix(&output_path)
        .identity("com.error.test", "1.0.0.0", "CN=ErrorTest")
        .properties("Error Test App", "Testing error handling")
        .application("ErrorApp", "missing.exe", "Error Application", "Error test app")
        .executable(&missing_exe) // This file doesn't exist
        .build();
    
    // Should fail gracefully
    assert!(result.is_err(), "Should fail with missing executable");
}

#[test] 
fn test_minimal_valid_package() {
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().join("minimal.msix");
    let exe_path = temp_dir.path().join("minimal.exe");
    
    // Create minimal executable
    fs::write(&exe_path, b"MZ\x90\x00").unwrap();
    
    // Build minimal package with only required fields
    let result = msix(&output_path)
        .identity("com.minimal.test", "1.0.0.0", "CN=MinimalTest")
        .properties("Minimal App", "Minimal test")
        .application("MinimalApp", "minimal.exe", "Minimal", "Minimal app")
        .executable(&exe_path)
        .build();
    
    assert!(result.is_ok(), "Failed to build minimal MSIX: {:?}", result);
    assert!(output_path.exists());
}

#[test]
fn test_package_with_custom_manifest() {
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().join("custom.msix");
    let exe_path = temp_dir.path().join("custom.exe");
    
    fs::write(&exe_path, b"MZ\x90\x00").unwrap();
    
    // Create package with custom identity format
    let result = msix(&output_path)
        .identity("com.custom.test", "2.1.0.0", "CN=CustomTest, O=TestOrg, C=US")
        .properties("Custom App", "Custom manifest test")
        .application("CustomApp", "custom.exe", "Custom Application", "Testing custom manifest")
        .executable(&exe_path)
        .build();
    assert!(result.is_ok(), "Failed to build custom MSIX: {:?}", result);
}