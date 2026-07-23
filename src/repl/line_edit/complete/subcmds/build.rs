//! Build systems and task runners.

pub(super) fn names(cmd: &str) -> Option<&'static [&'static str]> {
    match cmd {
        "cmake" => Some(CMAKE),
        "meson" => Some(MESON),
        "bazel" | "bazelisk" => Some(BAZEL),
        "gradle" => Some(GRADLE),
        "mvn" => Some(MVN),
        "sbt" => Some(SBT),
        "just" => Some(JUST),
        "task" => Some(TASK),
        "wasm-pack" => Some(WASM_PACK),
        "buck2" | "buck" => Some(BUCK),
        "please" => Some(PLEASE),
        "xcodebuild" => Some(XCODEBUILD),
        "fastlane" => Some(FASTLANE),
        "pod" => Some(POD),
        _ => None,
    }
}

const CMAKE: &[&str] = &[
    "--build",
    "--install",
    "--open",
    "--find-package",
    "-E",
    "-L",
    "-N",
    "-P",
];

const MESON: &[&str] = &[
    "setup",
    "configure",
    "compile",
    "test",
    "install",
    "dist",
    "introspect",
    "init",
    "rewrite",
    "subprojects",
    "wrap",
    "devenv",
    "env2mfile",
    "help",
];

const BAZEL: &[&str] = &[
    "analyze-profile",
    "aquery",
    "build",
    "canonicalize-flags",
    "clean",
    "coverage",
    "cquery",
    "dump",
    "fetch",
    "help",
    "info",
    "license",
    "mobile-install",
    "mod",
    "print_action",
    "query",
    "run",
    "shutdown",
    "sync",
    "test",
    "version",
];

const GRADLE: &[&str] = &[
    "assemble",
    "build",
    "buildDependents",
    "buildNeeded",
    "check",
    "clean",
    "components",
    "dependencies",
    "dependencyInsight",
    "dependentComponents",
    "help",
    "init",
    "jar",
    "javadoc",
    "model",
    "projects",
    "properties",
    "tasks",
    "test",
    "wrapper",
];

const MVN: &[&str] = &[
    "clean",
    "compile",
    "test",
    "package",
    "verify",
    "install",
    "deploy",
    "site",
    "validate",
    "dependency:tree",
    "dependency:resolve",
    "versions:display-dependency-updates",
    "help:effective-pom",
];

const SBT: &[&str] = &[
    "about", "clean", "compile", "console", "doc", "exit", "help", "inspect", "new", "package",
    "publish", "reload", "run", "test", "testOnly", "update",
];

const JUST: &[&str] = &[
    "--list",
    "--summary",
    "--evaluate",
    "--show",
    "--variables",
    "--choose",
    "--completions",
    "--dump",
    "--fmt",
    "--init",
    "--man",
];

const TASK: &[&str] = &[
    "--list",
    "--list-all",
    "--init",
    "--summary",
    "--taskfile",
    "--watch",
];

const WASM_PACK: &[&str] = &["build", "pack", "publish", "test", "new", "login"];

const BUCK: &[&str] = &[
    "audit", "build", "clean", "cquery", "help", "install", "kill", "query", "root", "run",
    "server", "targets", "test", "uquery",
];

const PLEASE: &[&str] = &[
    "build", "clean", "exec", "hash", "help", "init", "query", "run", "test", "update", "watch",
];

const XCODEBUILD: &[&str] = &[
    "-project",
    "-workspace",
    "-scheme",
    "-target",
    "-configuration",
    "-sdk",
    "-destination",
    "clean",
    "build",
    "test",
    "archive",
    "analyze",
];

const FASTLANE: &[&str] = &[
    "init",
    "new_action",
    "lanes",
    "list",
    "actions",
    "action",
    "docs",
    "enable_auto_complete",
    "add_plugin",
    "install_plugins",
    "update_plugins",
    "search_plugins",
    "env",
    "run",
];

const POD: &[&str] = &[
    "cache",
    "deintegrate",
    "env",
    "init",
    "install",
    "ipc",
    "lib",
    "list",
    "outdated",
    "plugins",
    "repo",
    "search",
    "setup",
    "spec",
    "update",
];
