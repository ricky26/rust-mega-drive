use std::{env, io};
use std::path::PathBuf;
use duct::cmd;

pub struct LLVMConfig {
    llvm_config_exe: PathBuf,
}

impl LLVMConfig {
    /// Fetch llvm-config from the environment. This will check for the
    /// LLVM_CONFIG environment variable, then fall-back to llvm-config on the
    /// PATH.
    pub fn from_environment() -> io::Result<LLVMConfig> {
        let llvm_config_exe = match env::var("LLVM_CONFIG") {
            Ok(s) => s.into(),
            Err(env::VarError::NotPresent) => "llvm-config".into(),
            Err(env::VarError::NotUnicode(_)) =>
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "LLVM_CONFIG is not valid utf-8"))?,
        };

        Ok(LLVMConfig::new(llvm_config_exe))
    }

    /// Create a new instance of LLVMConfig given the path to the llvm-config
    /// executable.
    pub fn new(llvm_config_exe: PathBuf) -> LLVMConfig {
        LLVMConfig { llvm_config_exe }
    }

    /// Get the path for LLVM binaries.
    pub fn bin_path(&self) -> anyhow::Result<PathBuf> {
        Ok(cmd!(&self.llvm_config_exe, "--bindir").read()?.into())
    }

    /// Return the path to clang.
    pub fn clang(&self) -> anyhow::Result<PathBuf> {
        Ok(self.bin_path()?.join("clang"))
    }

    /// Return the path to LLD with ld syntax.
    pub fn ld_lld(&self) -> anyhow::Result<PathBuf> {
        Ok(self.bin_path()?.join("ld.lld"))
    }

    /// Return the path to llvm-objcopy.
    pub fn objcopy(&self) -> anyhow::Result<PathBuf> {
        Ok(self.bin_path()?.join("llvm-objcopy"))
    }
}