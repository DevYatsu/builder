use xshell::Shell;

mod cmake;
pub mod custom;
mod docker;
mod dotnet;
mod flutter;
mod go;
mod gradle;
mod javascript;
mod just;
mod make;
mod maven;
mod python;
mod rust;
mod swift;
mod uv;
mod zig;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildOptions {
    pub run: bool,
    pub release: bool,
    pub test: bool,
    pub select_system: bool,
    pub select_command: bool,
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

pub trait BuildSystem: std::fmt::Debug {
    fn detect(&self, sh: &Shell) -> bool;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn execute(&self, sh: &Shell, options: &BuildOptions) -> crate::error::Result<Option<String>>;
}

pub fn get_systems() -> Vec<Box<dyn BuildSystem>> {
    vec![
        Box::new(rust::RustBuild),
        Box::new(make::MakeBuild),
        Box::new(just::JustBuild),
        Box::new(cmake::CMakeBuild),
        Box::new(javascript::JavaScriptBuild),
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
