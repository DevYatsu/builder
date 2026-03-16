use crate::error::Result;
use xshell::Shell;

mod rust;
mod make;
mod just;
mod cmake;
mod node;
mod bun;
mod deno;
mod go;
mod uv;
mod python;
mod swift;
mod flutter;
mod docker;
mod maven;
mod gradle;
mod zig;
mod dotnet;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildOptions {
    pub run: bool,
    pub release: bool,
    pub test: bool,
}

impl BuildOptions {
    pub fn verb(&self) -> &'static str {
        if self.test {
            "test"
        } else if self.run {
            "run"
        } else {
            "build"
        }
    }
}

pub trait BuildSystem {
    fn detect(&self, sh: &Shell) -> bool;
    fn name(&self) -> &'static str;
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> Result<()>;
}

pub fn get_systems() -> Vec<Box<dyn BuildSystem>> {
    vec![
        Box::new(rust::RustBuild),
        Box::new(make::MakeBuild),
        Box::new(just::JustBuild),
        Box::new(cmake::CMakeBuild),
        Box::new(node::NodeBuild),
        Box::new(bun::BunBuild),
        Box::new(deno::DenoBuild),
        Box::new(go::GoBuild),
        Box::new(uv::UvBuild),
        Box::new(python::PythonBuild),
        Box::new(swift::SwiftBuild),
        Box::new(flutter::FlutterBuild),
        Box::new(docker::DockerBuild),
        Box::new(maven::MavenBuild),
        Box::new(gradle::GradleBuild),
        Box::new(zig::ZigBuild),
        Box::new(dotnet::DotnetBuild),
    ]
}
