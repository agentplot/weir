_: {
  # A tracked file no formatter claims and no exclude names fails the check:
  # a new file type is decided in the change that adds it.
  settings.on-unmatched = "fatal";

  # What no formatter owns, each with its reason.
  settings.excludes = [
    # Prose: the README is Vale's (`prose`), and no formatter here rewraps
    # prose.
    "*.md"
    # Written by cargo and devenv in their own layouts.
    "*.lock"
    # The license text, kept as issued.
    "LICENSE"
    # Lists in gitignore syntax, Vale's own configuration and its word
    # lists, which no formatter reads.
    ".gitignore"
    ".vale.ini"
    "styles/config/vocabularies/**"
    # Build outputs and caches.
    "target/**"
    ".devenv/**"
    ".moon/cache/**"
  ];

  # The crates' Rust, with the pinned toolchain's rustfmt (devenv.nix).
  programs.rustfmt.enable = true;
  programs.rustfmt.edition = "2021";

  # Shell scripts, linted by shellcheck before shfmt lays them out.
  programs.shellcheck.enable = true;
  programs.shfmt.enable = true;
  programs.shfmt.indent_size = 2;
  settings.formatter.shellcheck.pipeline = "shell";
  settings.formatter.shellcheck.priority = 1;
  settings.formatter.shfmt.pipeline = "shell";
  settings.formatter.shfmt.priority = 2;

  programs.deadnix.enable = true;
  programs.nixfmt.enable = true;
  settings.formatter.deadnix.pipeline = "nix";
  settings.formatter.deadnix.priority = 1;
  settings.formatter.nixfmt.pipeline = "nix";
  settings.formatter.nixfmt.priority = 2;

  # .prototools is TOML and .versionrc convco's YAML, under names with no
  # extension.
  programs.taplo.enable = true;
  settings.formatter.taplo.includes = [ ".prototools" ];
  programs.yamlfmt.enable = true;
  settings.formatter.yamlfmt.includes = [ ".versionrc" ];
  programs.yamlfmt.settings.formatter = {
    type = "basic";
    retain_line_breaks_single = true;
  };
}
