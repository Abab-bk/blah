use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::interpreter::parse_code;
use crate::shared::Operation;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Manifest {
    pub name: String,

    #[serde(default)]
    pub features: HashMap<String, String>,

    #[serde(default)]
    pub dependencies: HashMap<String, DepSpec>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DepSpec {
    pub git: Option<String>,
    pub path: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct PackageInfo {
    manifest: Manifest,
    dir: PathBuf,
}

pub struct Resolver {
    packages: HashMap<String, PackageInfo>,
}

impl Resolver {
    pub fn new(project_dir: &Path) -> Option<Self> {
        let manifest_path = project_dir.join("bark.toml");
        if !manifest_path.exists() {
            return None;
        }

        let root_manifest = load_manifest(&manifest_path).unwrap();
        let packages_dir = project_dir.join(".packages");
        let mut packages = HashMap::new();

        packages.insert(
            root_manifest.name.clone(),
            PackageInfo {
                manifest: root_manifest,
                dir: project_dir.to_path_buf(),
            },
        );

        loop {
            let mut added = false;
            let snapshot: Vec<(String, Manifest, PathBuf)> = packages
                .iter()
                .map(|(k, v)| (k.clone(), v.manifest.clone(), v.dir.clone()))
                .collect();

            for (_, manifest, pkg_dir) in &snapshot {
                for (dep_name, spec) in &manifest.dependencies {
                    if let Some(_) = packages.get(dep_name) {
                        continue;
                    }

                    let dep_dir = if let Some(path) = &spec.path {
                        pkg_dir.join(path)
                    } else {
                        packages_dir.join(dep_name)
                    };

                    if let Some(git) = &spec.git {
                        if !dep_dir.join("bark.toml").exists() {
                            if dep_dir.exists() {
                                fs::remove_dir_all(&dep_dir).unwrap();
                            }
                            let status = std::process::Command::new("git")
                                .args(["clone", git, dep_dir.to_str().unwrap()])
                                .status()
                                .expect("git is required for dependency resolution");
                            assert!(status.success(), "failed to clone dependency: {}", dep_name);
                        }
                    }

                    let dep_manifest = load_manifest(&dep_dir.join("bark.toml")).unwrap();
                    packages.insert(
                        dep_name.clone(),
                        PackageInfo {
                            manifest: dep_manifest,
                            dir: dep_dir,
                        },
                    );
                    added = true;
                }
            }

            if !added {
                break;
            }
        }

        Some(Resolver { packages })
    }

    pub fn resolve(&self, source: &str) -> String {
        let re = regex::Regex::new(r"@([\w-]+)/([\w-]+)").unwrap();
        let mut result = source.to_string();

        for _ in 0..100 {
            let new_result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    let pkg_name = caps.get(1).unwrap().as_str();
                    let feat_name = caps.get(2).unwrap().as_str();

                    if let Some(info) = self.packages.get(pkg_name) {
                        if let Some(feat_path) = info.manifest.features.get(feat_name) {
                            let full_path = info.dir.join(feat_path);
                            if let Ok(content) = fs::read_to_string(&full_path) {
                                return content;
                            }
                        }
                    }
                    caps.get(0).unwrap().as_str().to_string()
                })
                .to_string();

            if new_result == result {
                break;
            }
            result = new_result;
        }

        result
    }
}

fn load_manifest(path: &Path) -> Result<Manifest, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let deserialized: Manifest = toml::from_str(&content).map_err(|e| e.to_string())?;
    Ok(deserialized)
}

pub fn parse_source(path: &Path) -> Vec<Operation> {
    let raw = fs::read_to_string(path).unwrap();
    let project_dir = path.parent().unwrap();
    let expanded = Resolver::new(project_dir)
        .map(|r| r.resolve(&raw))
        .unwrap_or(raw);
    parse_code(&expanded)
}
