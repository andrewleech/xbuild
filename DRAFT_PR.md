# [DRAFT] Add ergonomic builder pattern and comprehensive documentation

## Summary

This PR enhances the `msix` crate with a modern, ergonomic builder pattern API and comprehensive documentation to make MSIX package creation more accessible for build tools and developers.

## Key Changes

### 🏗️ Builder Pattern API
- **New `MsixBuilder`** with fluent, chainable API
- **Convenience function** `msix(output_path)` for quick setup
- **Sensible defaults** for common MSIX package requirements
- **Type-safe configuration** with compile-time validation

### 📚 Comprehensive Documentation
- **Crate-level documentation** with quick start examples
- **API documentation** for all public functions
- **Usage examples** for both builder and low-level APIs
- **Feature overview** highlighting key capabilities

### 🔌 Better Integration
- **Public exports** of `ZipFileOptions` and `Signer` for easier consumption
- **Backward compatibility** - all existing APIs remain unchanged
- **Zero breaking changes** to existing consumers

## Example Usage

### Before (Low-level API)
```rust
let manifest = AppxManifest::default();
// Manual manifest configuration...
let mut msix = Msix::new(PathBuf::from("output.msix"), manifest, true)?;
msix.add_icon("icon.png")?;
msix.finish(None)?;
```

### After (Builder Pattern)
```rust
use msix::msix;

let package_path = msix("output.msix")
    .identity("com.example.myapp", "1.0.0.0", "CN=Example Corp")
    .properties("My Application", "A sample application")
    .application("MyApp", "myapp.exe", "My Application", "A sample application")
    .capabilities(vec!["internetClient", "runFullTrust"])
    .default_target_device_family()
    .default_resource()
    .icon("icon.png")
    .executable("myapp.exe")
    .build()?;
```

## Benefits

- **Reduced boilerplate** - Common MSIX configurations in 10 lines vs 50+
- **Better discoverability** - IDE autocomplete guides users through options
- **Fewer errors** - Type safety and validation prevent common mistakes
- **Easier integration** - Build tools can integrate without deep MSIX knowledge
- **Maintained compatibility** - Existing code continues to work unchanged

## Testing

- ✅ All existing tests pass
- ✅ New builder pattern unit tests
- ✅ Documentation examples compile and run
- ✅ Backward compatibility verified

## Impact

This change makes the `msix` crate significantly more approachable for:
- Build tool integration (Tauri, cargo-bundle, etc.)
- Direct developer usage
- CI/CD pipeline automation
- Cross-platform MSIX package generation

The builder pattern follows Rust ecosystem conventions (similar to `reqwest::Client`, `tokio::process::Command`, etc.) making it familiar to Rust developers.

## Files Changed

- `src/builder.rs` - New builder pattern implementation
- `src/lib.rs` - Updated exports and documentation
- Tests and examples updated

---

**Note**: This is a draft PR for testing purposes. The implementation has been thoroughly tested in integration with Tauri's bundler system and provides a solid foundation for modern MSIX package creation in Rust.