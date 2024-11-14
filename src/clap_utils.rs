use clap::ArgMatches;
use std::sync::{RwLock, RwLockReadGuard};

pub(crate) static SUB_COMMAND_ARG_MATCHES: once_cell::sync::Lazy<RwLock<Option<ArgMatches>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(None));

fn get_sub_command_arg_matches() -> Option<ArgMatches> {
    let lock: RwLockReadGuard<Option<ArgMatches>> = SUB_COMMAND_ARG_MATCHES.read().unwrap();

    lock.clone()
}

pub fn write_sub_command_arg_matches(arg_matches: ArgMatches) {
    let mut lock = SUB_COMMAND_ARG_MATCHES.write().unwrap();
    *lock = Some(arg_matches);
}

pub fn clap_get_one_or_fallback(flag: &str, fallback: &str) -> String {
    // Read the lock
    let lock = SUB_COMMAND_ARG_MATCHES.read().unwrap();
    // Check if the `ArgMatches` is present and return the flag value or fallback
    if let Some(args) = &*lock {
        args.get_one::<String>(flag)
            .unwrap_or(&fallback.to_string())
            .to_string()
    } else {
        fallback.to_string()
    }
}

pub fn clap_get_flag_or_false(flag: &str) -> bool {
    // Read the lock
    let lock = SUB_COMMAND_ARG_MATCHES.read().unwrap();
    // Check if the `ArgMatches` is present and return the flag value or `false`
    if let Some(args) = &*lock {
        args.get_flag(flag)
    } else {
        false
    }
}
