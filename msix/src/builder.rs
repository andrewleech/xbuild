use crate::{manifest::*, Msix};
use anyhow::Result;
use std::path::{Path, PathBuf};
use xcommon;

/// Builder for creating MSIX packages with an ergonomic API
pub struct MsixBuilder {
    manifest: AppxManifest,
    output_path: PathBuf,
    compress: bool,
    icon_path: Option<PathBuf>,
    executable_path: Option<PathBuf>,
    assets: Vec<(PathBuf, PathBuf, xcommon::ZipFileOptions)>,
    directories: Vec<(PathBuf, PathBuf, xcommon::ZipFileOptions)>,
    signer: Option<xcommon::Signer>,
}

impl MsixBuilder {
    /// Create a new MSIX builder with the given output path
    pub fn new<P: AsRef<Path>>(output_path: P) -> Self {
        Self {
            manifest: AppxManifest::default(),
            output_path: output_path.as_ref().to_path_buf(),
            compress: true,
            icon_path: None,
            executable_path: None,
            assets: Vec::new(),
            directories: Vec::new(),
            signer: None,
        }
    }

    /// Set the package identity
    pub fn identity(mut self, name: &str, version: &str, publisher: &str) -> Self {
        self.manifest.identity.name = Some(name.to_string());
        self.manifest.identity.version = Some(version.to_string());
        self.manifest.identity.publisher = Some(publisher.to_string());
        self.manifest.identity.processor_architecture = Some("x64".to_string());
        self
    }

    /// Set the package properties
    pub fn properties(mut self, display_name: &str, description: &str) -> Self {
        self.manifest.properties.display_name = Some(display_name.to_string());
        self.manifest.properties.publisher_display_name = Some(display_name.to_string());
        self.manifest.properties.description = Some(description.to_string());
        self.manifest.properties.logo = Some("Images\\StoreLogo.png".to_string());
        self
    }

    /// Set the application details
    pub fn application(mut self, id: &str, executable: &str, display_name: &str, description: &str) -> Self {
        let mut app = Application::default();
        app.id = Some(id.to_string());
        app.executable = Some(executable.to_string());
        app.entry_point = Some("Windows.FullTrustApplication".to_string());
        
        app.visual_elements.display_name = Some(display_name.to_string());
        app.visual_elements.description = Some(description.to_string());
        app.visual_elements.background_color = Some("transparent".to_string());
        app.visual_elements.logo_44x44 = Some("Images\\Square44x44Logo.png".to_string());
        app.visual_elements.logo_150x150 = Some("Images\\Square150x150Logo.png".to_string());
        
        let mut default_tile = DefaultTile::default();
        default_tile.logo_310x150 = Some("Images\\Wide310x150Logo.png".to_string());
        app.visual_elements.default_tile = Some(default_tile);
        
        app.visual_elements.splash_screen = Some(SplashScreen {
            image: "Images\\SplashScreen.png".to_string(),
        });

        self.manifest.applications.application = vec![app];
        self
    }

    /// Add capabilities
    pub fn capabilities(mut self, capabilities: Vec<&str>) -> Self {
        self.manifest.capabilities = capabilities
            .into_iter()
            .map(|cap| match cap {
                "runFullTrust" => Capability::Restricted {
                    name: cap.to_string(),
                },
                _ => Capability::Capability {
                    name: cap.to_string(),
                },
            })
            .collect();
        self
    }

    /// Add default Windows Desktop target device family
    pub fn default_target_device_family(mut self) -> Self {
        self.manifest.dependencies.target_device_family = vec![TargetDeviceFamily::default()];
        self
    }

    /// Add default English resource
    pub fn default_resource(mut self) -> Self {
        self.manifest.resources.resource = vec![Resource {
            language: "en-US".to_string(),
        }];
        self
    }

    /// Set the application icon (will be scaled to all required sizes)
    pub fn icon<P: AsRef<Path>>(mut self, icon_path: P) -> Self {
        self.icon_path = Some(icon_path.as_ref().to_path_buf());
        self
    }

    /// Set the main executable
    pub fn executable<P: AsRef<Path>>(mut self, exe_path: P) -> Self {
        self.executable_path = Some(exe_path.as_ref().to_path_buf());
        self
    }

    /// Add a file to the package
    pub fn add_file<P: AsRef<Path>, Q: AsRef<Path>>(
        mut self,
        source: P,
        dest: Q,
        opts: xcommon::ZipFileOptions,
    ) -> Self {
        self.assets.push((
            source.as_ref().to_path_buf(),
            dest.as_ref().to_path_buf(),
            opts,
        ));
        self
    }

    /// Add a directory to the package
    pub fn add_directory<P: AsRef<Path>, Q: AsRef<Path>>(
        mut self,
        source: P,
        dest: Q,
        opts: xcommon::ZipFileOptions,
    ) -> Self {
        self.directories.push((
            source.as_ref().to_path_buf(),
            dest.as_ref().to_path_buf(),
            opts,
        ));
        self
    }

    /// Set whether to compress the package
    pub fn compress(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }

    /// Set a custom signer (defaults to debug signer)
    pub fn signer(mut self, signer: xcommon::Signer) -> Self {
        self.signer = Some(signer);
        self
    }

    /// Build the MSIX package
    pub fn build(self) -> Result<PathBuf> {
        // Create the MSIX instance
        let mut msix = Msix::new(self.output_path.clone(), self.manifest, self.compress)?;

        // Add icon if provided
        if let Some(icon_path) = &self.icon_path {
            msix.add_icon(icon_path)?;
        }

        // Add main executable if provided
        if let Some(exe_path) = &self.executable_path {
            let exe_name = exe_path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid executable path"))?;
            msix.add_file(exe_path, exe_name.as_ref(), xcommon::ZipFileOptions::Compressed)?;
        }

        // Add all files
        for (source, dest, opts) in &self.assets {
            msix.add_file(source, dest, *opts)?;
        }

        // Add all directories
        for (source, dest, opts) in &self.directories {
            msix.add_directory(source, dest, *opts)?;
        }

        // Finish and sign
        msix.finish(self.signer)?;

        Ok(self.output_path)
    }
}

/// Convenience function to create a new MSIX builder
pub fn msix<P: AsRef<Path>>(output_path: P) -> MsixBuilder {
    MsixBuilder::new(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_builder_pattern() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test.msix");

        let builder = msix(&output_path)
            .identity("com.example.app", "1.0.0.0", "CN=Example")
            .properties("Example App", "An example application")
            .application("ExampleApp", "app.exe", "Example App", "An example application")
            .capabilities(vec!["internetClient", "runFullTrust"])
            .default_target_device_family()
            .default_resource()
            .compress(true);

        // Verify manifest is properly configured
        assert_eq!(builder.manifest.identity.name, Some("com.example.app".to_string()));
        assert_eq!(builder.manifest.identity.version, Some("1.0.0.0".to_string()));
        assert_eq!(builder.manifest.properties.display_name, Some("Example App".to_string()));
        assert_eq!(builder.manifest.applications.application.len(), 1);
        assert_eq!(builder.manifest.capabilities.len(), 2);
    }

    #[test]
    fn test_builder_with_icon() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test_icon.msix");
        let icon_path = temp_dir.path().join("icon.png");
        
        // Create a dummy icon file
        fs::write(&icon_path, b"dummy icon data").unwrap();

        let builder = msix(&output_path)
            .identity("com.test.icon", "1.0.0.0", "CN=Test")
            .properties("Icon Test", "Testing icon functionality")
            .application("IconApp", "iconapp.exe", "Icon App", "Icon test app")
            .icon(&icon_path);

        assert_eq!(builder.icon_path, Some(icon_path));
    }

    #[test]
    fn test_builder_with_executable() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test_exe.msix");
        let exe_path = temp_dir.path().join("app.exe");
        
        // Create a dummy executable file
        fs::write(&exe_path, b"dummy exe data").unwrap();

        let builder = msix(&output_path)
            .identity("com.test.exe", "1.0.0.0", "CN=Test")
            .properties("Exe Test", "Testing executable functionality")
            .application("ExeApp", "app.exe", "Exe App", "Exe test app")
            .executable(&exe_path);

        assert_eq!(builder.executable_path, Some(exe_path));
    }

    #[test]
    fn test_builder_with_assets() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test_assets.msix");
        let asset_path = temp_dir.path().join("data.txt");
        
        // Create a dummy asset file
        fs::write(&asset_path, b"asset data").unwrap();

        let builder = msix(&output_path)
            .identity("com.test.assets", "1.0.0.0", "CN=Test")
            .properties("Asset Test", "Testing asset functionality")
            .application("AssetApp", "app.exe", "Asset App", "Asset test app")
            .add_file(&asset_path, "data/data.txt", xcommon::ZipFileOptions::Compressed);

        assert_eq!(builder.assets.len(), 1);
        assert_eq!(builder.assets[0].0, asset_path);
        assert_eq!(builder.assets[0].1, PathBuf::from("data/data.txt"));
    }

    #[test]
    fn test_builder_with_directory() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test_dir.msix");
        let dir_path = temp_dir.path().join("content");
        
        // Create a directory with files
        fs::create_dir(&dir_path).unwrap();
        fs::write(dir_path.join("file1.txt"), b"file 1").unwrap();
        fs::write(dir_path.join("file2.txt"), b"file 2").unwrap();

        let builder = msix(&output_path)
            .identity("com.test.dir", "1.0.0.0", "CN=Test")
            .properties("Dir Test", "Testing directory functionality")
            .application("DirApp", "app.exe", "Dir App", "Dir test app")
            .add_directory(&dir_path, "content", xcommon::ZipFileOptions::Compressed);

        assert_eq!(builder.directories.len(), 1);
        assert_eq!(builder.directories[0].0, dir_path);
        assert_eq!(builder.directories[0].1, PathBuf::from("content"));
    }

    #[test]
    fn test_builder_compression_settings() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test_compress.msix");

        let builder_compressed = msix(&output_path)
            .identity("com.test.compress", "1.0.0.0", "CN=Test")
            .compress(true);
        assert_eq!(builder_compressed.compress, true);

        let builder_uncompressed = msix(&output_path)
            .identity("com.test.nocompress", "1.0.0.0", "CN=Test")
            .compress(false);
        assert_eq!(builder_uncompressed.compress, false);
    }

    #[test]
    fn test_multiple_applications() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test_multi_app.msix");

        let mut builder = msix(&output_path)
            .identity("com.test.multi", "1.0.0.0", "CN=Test")
            .properties("Multi App", "Multiple applications")
            .application("App1", "app1.exe", "Application 1", "First app");
        
        // Add second application manually
        let mut app2 = Application::default();
        app2.id = Some("App2".to_string());
        app2.executable = Some("app2.exe".to_string());
        app2.entry_point = Some("Windows.FullTrustApplication".to_string());
        app2.visual_elements.display_name = Some("Application 2".to_string());
        app2.visual_elements.description = Some("Second app".to_string());
        app2.visual_elements.background_color = Some("transparent".to_string());
        app2.visual_elements.logo_150x150 = Some("Images\\Square150x150Logo.png".to_string());
        app2.visual_elements.logo_44x44 = Some("Images\\Square44x44Logo.png".to_string());
        app2.visual_elements.default_tile = Some(DefaultTile {
            logo_310x150: Some("Images\\Wide310x150Logo.png".to_string()),
            logo_310x310: Some("Images\\LargeTile.png".to_string()),
            logo_71x71: Some("Images\\SmallTile.png".to_string()),
            short_name: None,
            show_names_on_tiles: ShowNameOnTiles::default(),
        });
        builder.manifest.applications.application.push(app2);

        assert_eq!(builder.manifest.applications.application.len(), 2);
        assert_eq!(builder.manifest.applications.application[0].id, Some("App1".to_string()));
        assert_eq!(builder.manifest.applications.application[1].id, Some("App2".to_string()));
    }

    #[test]
    fn test_custom_capabilities() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test_capabilities.msix");

        let builder = msix(&output_path)
            .identity("com.test.capabilities", "1.0.0.0", "CN=Test")
            .properties("Capability Test", "Testing capabilities")
            .capabilities(vec![
                "internetClient",
                "internetClientServer", 
                "privateNetworkClientServer",
                "documentsLibrary",
                "runFullTrust"
            ]);

        assert_eq!(builder.manifest.capabilities.len(), 5);
        // Check that capabilities were added correctly
        let has_internet_client = builder.manifest.capabilities.iter().any(|cap| {
            match cap {
                Capability::Capability { name } => name == "internetClient",
                _ => false,
            }
        });
        let has_run_full_trust = builder.manifest.capabilities.iter().any(|cap| {
            match cap {
                Capability::Restricted { name } => name == "runFullTrust",
                _ => false,
            }
        });
        assert!(has_internet_client);
        assert!(has_run_full_trust);
    }

    #[test]
    fn test_version_parsing() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test_version.msix");

        // Test various version formats
        let versions = vec![
            "1.0.0.0",
            "2.5.10.0",
            "99.99.99.99",
            "0.1.0.0",
        ];

        for version in versions {
            let builder = msix(&output_path)
                .identity("com.test.version", version, "CN=Test");
            assert_eq!(builder.manifest.identity.version, Some(version.to_string()));
        }
    }
}