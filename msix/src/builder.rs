use crate::{manifest::*, Msix};
use anyhow::Result;
use std::path::{Path, PathBuf};
use xcommon::{Signer, ZipFileOptions};

/// Builder for creating MSIX packages with an ergonomic API
pub struct MsixBuilder {
    manifest: AppxManifest,
    output_path: PathBuf,
    compress: bool,
    icon_path: Option<PathBuf>,
    executable_path: Option<PathBuf>,
    assets: Vec<(PathBuf, PathBuf, ZipFileOptions)>,
    directories: Vec<(PathBuf, PathBuf, ZipFileOptions)>,
    signer: Option<Signer>,
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
        opts: ZipFileOptions,
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
        opts: ZipFileOptions,
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
    pub fn signer(mut self, signer: Signer) -> Self {
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
            msix.add_file(exe_path, exe_name.as_ref(), ZipFileOptions::Compressed)?;
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
}