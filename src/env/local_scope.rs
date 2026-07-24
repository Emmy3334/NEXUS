//! Function-scoped shell locals (save/restore around function calls).

use super::ShellEnvironment;

use std::collections::BTreeMap;

/// One function invocation's `local` / exported bindings to restore on leave.
#[derive(Debug, Default)]
pub(crate) struct LocalFrame {
    /// Prior local per name (`None` = was unset before `local` / `typeset`).
    saved: BTreeMap<String, Option<String>>,
    /// Prior exported value when `typeset -x` touched the export map.
    saved_vars: BTreeMap<String, Option<String>>,
}

impl ShellEnvironment {
    pub(crate) fn push_local_frame(&mut self) {
        self.local_frames.push(LocalFrame::default());
    }

    pub(crate) fn pop_local_frame(&mut self) {
        let Some(frame) = self.local_frames.pop() else {
            return;
        };
        for (name, prior) in frame.saved {
            match prior {
                Some(value) => self.set_local(name, value),
                None => {
                    let _ = self.unset_local(&name);
                }
            }
        }
        for (name, prior) in frame.saved_vars {
            match prior {
                Some(value) => self.set(name, value),
                None => {
                    let _ = self.unset(&name);
                }
            }
        }
    }

    /// Declare a function-local; save prior once per name in the top frame.
    pub(crate) fn declare_local(&mut self, name: &str, value: &str) {
        let prior = self.get_local(name).map(str::to_owned);
        if let Some(frame) = self.local_frames.last_mut() {
            frame.saved.entry(name.to_owned()).or_insert(prior);
        }
        self.set_local(name, value);
    }

    /// Function-local plus export (`typeset -x`); restore both on leave.
    pub(crate) fn declare_local_export(&mut self, name: &str, value: &str) {
        let prior_local = self.get_local(name).map(str::to_owned);
        let prior_var = self.get(name).map(str::to_owned);
        if let Some(frame) = self.local_frames.last_mut() {
            frame.saved.entry(name.to_owned()).or_insert(prior_local);
            frame.saved_vars.entry(name.to_owned()).or_insert(prior_var);
        }
        self.set_local(name, value);
        self.set(name, value);
    }
}
