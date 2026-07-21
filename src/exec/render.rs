//! Lossless-enough source rendering for launching an isolated NEXUS child.

use crate::parse::{CommandList, Pipeline, PipelineCommand, Redirect, RedirectKind, SimpleCommand};

pub(super) fn render_list(list: &CommandList<'_>) -> String {
    let mut source = String::new();
    for (index, pipeline) in list.pipelines.iter().enumerate() {
        source.push_str(&render_pipeline(pipeline));
        if pipeline.background {
            source.push_str(" &");
        } else if index + 1 < list.pipelines.len() {
            source.push_str(" ;");
        }
    }
    source
}

pub(super) fn render_pipeline(pipeline: &Pipeline<'_>) -> String {
    pipeline
        .commands
        .iter()
        .map(render_command)
        .collect::<Vec<_>>()
        .join(" | ")
}

fn render_command(command: &PipelineCommand<'_>) -> String {
    match command {
        PipelineCommand::Simple(simple) => render_simple(simple),
        PipelineCommand::Subshell { list, redirects } => {
            let mut source = format!("( {} )", render_list(list));
            append_redirects(&mut source, redirects);
            source
        }
    }
}

fn render_simple(simple: &SimpleCommand<'_>) -> String {
    let mut source = simple.argv.join(" ");
    append_redirects(&mut source, &simple.redirects);
    source
}

fn append_redirects(source: &mut String, redirects: &[Redirect<'_>]) {
    for redirect in redirects {
        source.push(' ');
        source.push_str(redirect_symbol(redirect.kind));
        source.push(' ');
        source.push_str(redirect.path);
    }
}

const fn redirect_symbol(kind: RedirectKind) -> &'static str {
    match kind {
        RedirectKind::Read => "<",
        RedirectKind::Write => ">",
        RedirectKind::Append => ">>",
        RedirectKind::Heredoc => "<<",
    }
}
