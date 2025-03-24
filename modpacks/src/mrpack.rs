pub mod modrinth_index {
    use dwldutil::{DLBuilder, DLFile, DLHashes};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ModrinthIndex {
        dependencies: HashMap<String, String>,
        files: Vec<File>,
        #[serde(alias = "formatVersion")]
        format_version: u32,
        game: String,
        name: String,
        #[serde(alias = "versionId")]
        version_id: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct File {
        downloads: Vec<String>,
        env: Environment,
        #[serde(alias = "fileSize")]
        file_size: u64,
        hashes: Hashes,
        path: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Environment {
        client: String,
        server: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Hashes {
        sha1: String,
        sha512: String,
    }

    pub struct ModrinthIndexBuilder {
        dependencies: HashMap<String, String>,
        files: Vec<File>,
        format_version: u32,
        game: String,
        name: String,
        version_id: String,
    }

    impl ModrinthIndexBuilder {
        pub fn new(name: String, version_id: String) -> Self {
            ModrinthIndexBuilder {
                dependencies: HashMap::new(),
                files: Vec::new(),
                format_version: 1,
                game: "minecraft".to_string(),
                name,
                version_id,
            }
        }

        pub fn add_dependency(mut self, id: String, version: String) -> Self {
            self.dependencies.insert(id, version);
            self
        }

        pub fn add_file(mut self, file: File) -> Self {
            self.files.push(file);
            self
        }

        pub fn format_version(mut self, version: u32) -> Self {
            self.format_version = version;
            self
        }

        pub fn build(self) -> ModrinthIndex {
            ModrinthIndex {
                dependencies: self.dependencies,
                files: self.files,
                format_version: self.format_version,
                game: self.game,
                name: self.name,
                version_id: self.version_id,
            }
        }
    }

    impl ModrinthIndex {
        pub fn deserialize(data: &str) -> Result<Self, serde_json::Error> {
            serde_json::from_str(data)
        }
        pub fn serialize(&self) -> Result<String, serde_json::Error> {
            serde_json::to_string(self)
        }
        pub fn new_builder(name: String, version_id: String) -> ModrinthIndexBuilder {
            ModrinthIndexBuilder::new(name, version_id)
        }
        pub fn files_to_dl(&self) -> DLBuilder {
            let files = self
                .files
                .iter()
                .map(|file| {
                    DLFile::new()
                        .with_url(&file.downloads.get(0).unwrap().clone())
                        .with_path(&file.path.clone())
                        .with_size(file.file_size)
                        .with_hashes(
                            DLHashes::new()
                                .sha1(&file.hashes.sha1.clone())
                                .sha512(&file.hashes.sha512.clone()),
                        )
                })
                .collect();
            DLBuilder::from_files(files)
        }
    }
}
