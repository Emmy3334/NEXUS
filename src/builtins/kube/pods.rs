//! `@kube pods` listing.

use super::{args, print};
use crate::builtins::BuiltinResult;
use crate::kube::{self, PodScope};

use std::io::{self, Write};

pub(super) fn run(
    args: &[String],
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let scope = match args::pod_scope(args) {
        Ok(scope) => scope,
        Err(msg) => {
            writeln!(stderr, "@kube: {msg}")?;
            return Ok(BuiltinResult::Status(1));
        }
    };
    let hint = match &scope {
        PodScope::All => "cluster",
        PodScope::Default => "default namespace",
        PodScope::Namespace(ns) => ns.as_str(),
    };
    print::table(kube::list_pods_table(&scope), hint, stdout, stderr)
}
