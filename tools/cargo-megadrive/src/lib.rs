use std::path::{PathBuf, Path};
use std::fs;
use std::env;
use std::ffi::{OsStr, OsString};
use duct::cmd;
use cargo_metadata::MetadataCommand;
use anyhow::anyhow;
use crate::metadata::{Metadata};

mod llvm_config;
mod metadata;

pub struct Builder {
    cargo_metadata: cargo_metadata::Metadata,
    output: Option<PathBuf>,
    target: String,
    target_triple: String,
    profile: String,
    linker_script: PathBuf,
    entry: Option<PathBuf>,
    verbose: bool,
}

impl Builder {
    /// Create a new ROM builder.
    pub fn new(manifest_path: Option<impl Into<PathBuf>>) -> anyhow::Result<Builder> {
        let sdk_home = env::var("MEGADRIVE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| "/usr/share/megadrive".into());

        let mut cargo_metadata = MetadataCommand::new();

        if let Some(manifest_path) = manifest_path {
            cargo_metadata.manifest_path(manifest_path);
        }

        let cargo_metadata = cargo_metadata.exec()?;
        let mut target = env::var("MEGADRIVE_TARGET")
            .unwrap_or_else(|_| "m68k-none-eabi".into());

        let mut target_json = sdk_home.join("targets");
        target_json.push(&target);
        target_json.set_extension("json");
        if fs::metadata(&target_json).map_or(false, |m| m.is_file()) {
            target = target_json.to_string_lossy().into_owned();
        }

        let root_package = cargo_metadata.root_package()
            .ok_or(anyhow!("missing root package"))?;
        let package_dir = root_package.manifest_path.parent()
            .unwrap_or(Path::new(""));

        let metadata = if let Some(c) = root_package.metadata.get("megadrive") {
            serde_json::from_value(c.clone())?
        } else {
            Metadata::default()
        };

        let entry = metadata.entry_assembly.or_else(|| {
            let default_entry_path = package_dir.join("entry.S");
            let has_entry = fs::metadata(&default_entry_path).map_or(false, |x| x.is_file());
            if has_entry { Some(default_entry_path.to_owned()) } else { None }
        });

        let linker_script = metadata.linker_script
            .unwrap_or_else(|| sdk_home.join("ldscripts/megadrive.x"));

        let target_triple = {
            let filename =
                Path::new(&target)
                    .file_name()
                    .map(Path::new);
            let stem = filename
                .and_then(|f| f.file_stem())
                .map(|p| Path::new(p));
            stem.and_then(|s| s.to_str())
                .unwrap_or("").to_string()
        };

        Ok(Builder {
            cargo_metadata,
            target,
            target_triple,
            linker_script,
            entry,
            output: None,
            profile: "release".into(),
            verbose: false,
        })
    }

    /// Enable verbose logging.
    pub fn verbose(mut self, v: bool) -> Self {
        self.verbose = v;
        self
    }

    /// Build the ROM.
    pub fn build(self) -> anyhow::Result<()> {
        let llvm_config = llvm_config::LLVMConfig::from_environment()?;
        let root = self.cargo_metadata.root_package().unwrap();
        let out_dir = {
            let mut path = self.cargo_metadata.target_directory.clone();
            path.push(&self.target_triple);
            path.push(&self.profile);
            path
        };

        let staticlib = {
            let mut path = out_dir.clone();
            path.push(format!("lib{}.a", &root.name));
            path
        };
        let elf = {
            let mut path = out_dir.clone();
            path.push(&root.name);
            path.set_extension("elf");
            path
        };
        let output = self.output.clone()
            .unwrap_or_else(|| {
                let mut path = self.cargo_metadata.target_directory.clone();
                path.push(&self.target_triple);
                path.push(&self.profile);
                path.push(&root.name);
                path.set_extension("md");
                path
            });

        let verbose_flag = OsString::from(if self.verbose { "-v" } else { "" });
        cmd!("cargo", "build", "--manifest-path", &root.manifest_path,
            "-Z", "unstable-options", "-Z", "build-std=core",
            "--profile", &self.profile, "--target", &self.target, verbose_flag)
            .run()?;

        let mut objects = vec![];
        if let Some(entry) = self.entry.as_ref() {
            cmd!(llvm_config.clang()?, "-target", &self.target_triple,
                "-c", entry).dir(&out_dir).run()?;

            let mut object = out_dir.clone();
            object.push(entry.file_name().unwrap_or(OsStr::new("")));
            object.set_extension("o");
            objects.push(object);
        }

        let mut link_args: Vec<OsString> = vec![
            "--gc-sections".into(),
            "-o".into(), OsString::from(&elf), "-T".into(), self.linker_script.into()];
        link_args.extend(objects.into_iter().map(|o| o.into_os_string()));
        link_args.push(OsString::from(staticlib));
        cmd(llvm_config.ld_lld()?, &link_args).run()?;

        cmd!(llvm_config.objcopy()?, "-O", "binary", &elf, &output).run()?;
        Ok(())
    }
}
