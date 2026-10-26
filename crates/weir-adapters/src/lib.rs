//! The built-in adapters of the statement in agentplot/blueprints
//! `design/weir/requirements.md`, held once so every host runs one copy of
//! each (61 to 64; C.6; section 9).
//!
//! The folder drop and the meeting file are adapters like any other: each
//! is a function from paths to an adapter's lines, which a host takes as it
//! takes any adapter's (61, 62, 64). The signals-folder import is no
//! adapter: it reads a body of captures made before their inbox existed
//! into import lines, which a host's import operation alone writes (63,
//! 75a).
//!
//! Every function answers from its arguments alone and writes only into the
//! raw place it is handed (core.1).
