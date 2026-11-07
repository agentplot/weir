{
  pkgs,
  lib,
  config,
  ...
}:

let
  # proto from its own release build: nixpkgs trails it by minor versions or
  # more. Linux takes the static musl build, so no loader needs patching.
  proto =
    let
      version = "0.63.1";
      system = pkgs.stdenv.hostPlatform.system;
      target =
        {
          aarch64-darwin = "aarch64-apple-darwin";
          x86_64-darwin = "x86_64-apple-darwin";
          aarch64-linux = "aarch64-unknown-linux-musl";
          x86_64-linux = "x86_64-unknown-linux-musl";
        }
        .${system} or (throw "proto ${version}: no release build for ${system}");
      hashes = {
        aarch64-apple-darwin = "8bfde4c862381eeb6e82d0858bb6215732cbd1e384e4eae45da8fa08095e0ffd";
        x86_64-apple-darwin = "aaaff78c7125de1b765f00177f35da75ee16fb1363e07632ace27213ff5d893b";
        aarch64-unknown-linux-musl = "111f5b6a38d3979ebafbc095c8d91ef69c99d1f018addb3582179d4ab91b3441";
        x86_64-unknown-linux-musl = "8596b98ccd833d1203e71931ba25e866d3239f9d3859c922a035a7b830f29d17";
      };
    in
    pkgs.stdenvNoCC.mkDerivation {
      pname = "proto";
      inherit version;
      src = pkgs.fetchurl {
        url = "https://github.com/moonrepo/proto/releases/download/v${version}/proto_cli-${target}.tar.xz";
        sha256 = hashes.${target};
      };
      sourceRoot = "proto_cli-${target}";
      installPhase = ''
        runHook preInstall
        install -Dm755 -t $out/bin proto proto-shim
        runHook postInstall
      '';
    };

  # The application toolchain .prototools pins (moon), on PATH: what the
  # shell activates, and what a git hook, which runs outside any shell a
  # session opened, activates for itself. proto's home is this repository's,
  # in the git common dir so every worktree shares it, and only this
  # repository's pins are read.
  protoHome = ''
    PROTO_HOME="$(${lib.getExe pkgs.git} rev-parse --path-format=absolute --git-common-dir)/proto"
    export PROTO_HOME
  '';
  activate = ''
    ${protoHome}
    eval "$(${proto}/bin/proto activate bash --export --no-bin --config-mode upwards 2>/dev/null)"
  '';

  # Vale checks the README's prose (.vale.ini, styles/Weir). From its own
  # release: nixpkgs trails it. Bump the version and the four hashes from
  # the release's checksums file.
  valeVersion = "3.24.0";
  valeAssets = {
    aarch64-darwin = {
      os = "macOS_arm64";
      sha256 = "87b513f26499f6657c15cf7175ba5b058a7c1ff2bb2d4e07265db2dd43451e44";
    };
    x86_64-darwin = {
      os = "macOS_64-bit";
      sha256 = "eeb39e86f1daac27cc2a77fef3af1824d50559bdc22329fb25fb921c80c79472";
    };
    aarch64-linux = {
      os = "Linux_arm64";
      sha256 = "104cbd349279e50c4cb40adb70adf345d93cc4a3a0191f9e68bdccb97086b582";
    };
    x86_64-linux = {
      os = "Linux_64-bit";
      sha256 = "867534ddb678abca7f214bc4706ead0922bd5252ffe8035fc227013f66c8b214";
    };
  };
  valeAsset = valeAssets.${pkgs.stdenv.hostPlatform.system};
  vale = pkgs.stdenvNoCC.mkDerivation {
    pname = "vale";
    version = valeVersion;
    src = pkgs.fetchurl {
      url = "https://github.com/errata-ai/vale/releases/download/v${valeVersion}/vale_${valeVersion}_${valeAsset.os}.tar.gz";
      inherit (valeAsset) sha256;
    };
    sourceRoot = ".";
    installPhase = "install -Dm755 vale $out/bin/vale";
  };

  # OpenSpec, which writes and validates the changes and specs, and writes
  # the Claude Code skills and commands the tree tracks: the release flume,
  # boomdrive and sluice pin, built as nixpkgs builds its older one. Bump
  # the version, then each hash from the mismatch the build reports, then
  # write the skills and commands under .claude/ again with it, `sh
  # check-openspec-claude.sh --write` in the shell, since root:openspec-claude
  # holds them to this release (record 0005).
  openspecVersion = "1.14.3";
  openspec = pkgs.openspec.overrideAttrs (
    finalAttrs: _: {
      version = openspecVersion;
      src = pkgs.fetchFromGitHub {
        owner = "Fission-AI";
        repo = "OpenSpec";
        tag = "v${finalAttrs.version}";
        hash = "sha256-tdTX0nkDe/aNZgLckh6/B6BtoSb40w0Y5qzg+d1LjHU=";
      };
      pnpmDeps = pkgs.fetchPnpmDeps {
        inherit (finalAttrs) pname version src;
        pnpm = pkgs.pnpm_10;
        fetcherVersion = 3;
        hash = "sha256-fKm4sBdTDeQfIiE0k301jtRbiuJrBE7YIEBu8RZxuXI=";
      };
    }
  );

  # devenv's own script that makes the files it declares, Claude Code's
  # among them, run from the repository's root.
  writeFiles = ''
    cd ${lib.escapeShellArg config.devenv.root} || exit
    ${config.tasks."devenv:files".exec}
  '';

  # treefmt's rustfmt: the pinned toolchain's.
  rustfmt = pkgs.writeShellApplication {
    name = "rustfmt";
    text = ''
      ${activate}
      exec rustfmt "$@"
    '';
  };
in
{
  # The system tools are this file's, pinned by devenv.lock; moon is
  # .prototools', installed by the weir:proto:install task on entering the
  # shell and activated by enterShell.
  packages = [
    pkgs.git
    pkgs.jq
    proto
    vale
    openspec
    # GNU recutils: the crates' tests hold every record they write to
    # `recfix`, as a write is held before it lands (core.24).
    pkgs.recutils
    # shellcheck: the shell's scripts, linted in the shell as treefmt lints
    # them before shfmt lays them out.
    pkgs.shellcheck
  ];

  # The commit hooks' config (.pre-commit-config.yaml) is linked at the top of
  # the worktree, as devenv's files task links it on `devenv shell`: the
  # script `devenv print-dev-env` prints runs enterShell and none of the
  # tasks, and a place is made ready by sourcing that script, so a place's
  # first commit passes the shared hooks before anyone enters its shell. A
  # file of the user's own there is left as devenv leaves it.
  #
  # moon's task archives (.moon/cache/outputs) are a link into the git common
  # dir, so every worktree shares them. The rest of .moon/cache records the
  # checkout's absolute paths and stays its own.
  #
  # devenv's files, Claude Code's below, are made here as well as by devenv's
  # own task: a place's readiness is sluice's equip, which sources the
  # script `devenv print-dev-env` prints and runs no task, and the agent
  # started after it reads them as it starts (record 0005).
  # Their words go to no output a caller reads the environment from, and a
  # working copy where they cannot be written still gets its shell.
  enterShell = ''
    if ! (${writeFiles}) >/dev/null; then
      echo "devenv.nix: Claude Code's files were not written in ${config.devenv.root}" >&2
    fi
    if _hooks_root="$(git rev-parse --show-toplevel 2>/dev/null)"; then
      _hooks_config="$_hooks_root/${config.git-hooks.configPath}"
      if [ -L "$_hooks_config" ] || [ ! -e "$_hooks_config" ]; then
        ln -sfn ${config.git-hooks.configFile} "$_hooks_config"
      fi
    fi
    unset _hooks_root _hooks_config
    if _moon_root="$(git rev-parse --show-toplevel 2>/dev/null)" &&
      _moon_common="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)"; then
      _moon_cache="$_moon_root/.moon/cache"
      [ -L "$_moon_cache" ] && rm -f "$_moon_cache"
      mkdir -p "$_moon_common/moon-outputs" "$_moon_cache"
      if [ ! -L "$_moon_cache/outputs" ]; then
        rm -rf "$_moon_cache/outputs"
        ln -s "$_moon_common/moon-outputs" "$_moon_cache/outputs"
      fi
      # moon puts $MOON_HOME/bin on every task's PATH too: this repository's,
      # beside proto's, so no moon of the user's runs in a task.
      MOON_HOME="$_moon_common/moon-home"
      export MOON_HOME
    fi
    unset _moon_root _moon_common _moon_cache
    ${activate}
  '';

  # The versions .prototools lists, installed on entering the shell, before
  # enterShell activates them. A task, not part of enterShell: the script
  # `devenv print-dev-env` prints holds enterShell and none of the tasks, so
  # a check run sourcing it never installs.
  tasks."weir:proto:install" = {
    exec = ''
      ${protoHome}
      ${proto}/bin/proto install --config-mode upwards
    '';
    before = [ "devenv:enterShell" ];
  };

  # Claude Code's project files (agentplot/boomdrive's record 0022; record
  # 0005): .mcp.json from the servers below and .claude/settings.json from
  # the rest, each a link devenv makes into the store and git ignores. The
  # repository's one server is devenv's hosted one, which searches packages
  # and devenv's options and starts no process on the host; it is named
  # here because a list of the repository's own replaces devenv's default.
  # Every server named is approved by name in the settings, so a session in
  # a trusted folder, a place among them, starts them and asks nothing.
  claude.code = {
    enable = true;
    mcpServers."mcp.devenv.sh" = {
      type = "http";
      url = "https://mcp.devenv.sh";
    };
    # devenv's own hook, on wherever git-hooks are, runs prek after every
    # edit. An agent here runs each tool through `devenv shell --`, so prek
    # is not on its session's PATH and the hook would fail at each edit;
    # the pre-commit hook runs treefmt on what a commit holds, and the gate
    # on what lands.
    hooks.git-hooks-run.enable = false;
  };
  files.${config.claude.code.settingsPath}.json.enabledMcpjsonServers =
    lib.attrNames config.claude.code.mcpServers;

  treefmt.enable = true;
  treefmt.config = {
    imports = [ ./treefmt.nix ];
    projectRootFile = "devenv.nix";
    programs.rustfmt.package = rustfmt;
  };

  scripts.prose = {
    description = "The README's prose, checked by Vale (.vale.ini)";
    exec = ''
      set -euo pipefail
      cd "$(git rev-parse --show-toplevel)"
      vale README.md "$@"
    '';
  };
  scripts.check-fast = {
    description = "Every check-fast task: moon run :#check-fast";
    exec = ''
      set -euo pipefail
      cd "$(git rev-parse --show-toplevel)"
      moon run ':#check-fast' "$@" </dev/null
    '';
  };
  # The floor under every commit, installed into the common hooks directory
  # on `devenv shell`. convco refuses a message that is not a conventional
  # commit (.versionrc).
  git-hooks.hooks = {
    convco = {
      enable = true;
      settings.configPath = ".versionrc";
    };
    treefmt.enable = true;
  };

  # devenv 2.4 installs the hook with an absolute config path into the hooks
  # directory every worktree shares; a plain `prek install` writes no path,
  # and prek reads the config at the top of the worktree the commit is in.
  # Remove this once devenv installs without the absolute path
  # (cachix/devenv#3270).
  tasks."devenv:git-hooks:install".exec = lib.mkForce ''
    set -euo pipefail
    hooks="$(${lib.getExe pkgs.git} rev-parse --path-format=absolute --git-path hooks)"
    for type in pre-commit commit-msg; do
      if ! grep -q 'File generated by prek' "$hooks/$type" 2>/dev/null || grep -q -- '--config=' "$hooks/$type"; then
        ${lib.getExe config.git-hooks.package} install --force -t "$type"
      fi
    done
  '';

  # A new place's first commit, made before anyone enters its shell: in an
  # empty repository holding the files the hooks read, the script `devenv
  # print-dev-env` prints for this checkout is sourced, as a place is made
  # ready, and commits are made through prek's hooks installed as above. The
  # hooks find their config: a message convco refuses is refused, and a
  # conventional one lands.
  scripts.hooks = {
    description = "A new place commits through the shared hooks before its shell is entered";
    exec = ''
      set -euo pipefail
      cd "$(git rev-parse --show-toplevel)"
      tmp="$(mktemp -d)"
      trap 'rm -rf "$tmp"' EXIT
      if ! env -i HOME="$HOME" PATH="$PATH" TMPDIR="''${TMPDIR:-/tmp}" \
        devenv print-dev-env >"$tmp/toolchain" 2>"$tmp/toolchain.log"; then
        cat "$tmp/toolchain.log" >&2
        exit 1
      fi
      mkdir "$tmp/place"
      cp devenv.nix treefmt.nix .versionrc "$tmp/place/"
      cd "$tmp/place"
      env -i HOME="$HOME" PATH="$PATH" TMPDIR="''${TMPDIR:-/tmp}" \
        GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 \
        bash -c '
          git init -q
          source "$1" >/dev/null 2>&1
          set -euo pipefail
          export PREK_HOME="$2"
          "$3" install -q -t pre-commit -t commit-msg
          git add devenv.nix treefmt.nix .versionrc
          commit() { git -c user.name=weir -c user.email=weir@localhost commit -q -m "$1"; }
          if commit "update" 2>/dev/null; then
            echo "hooks: a message convco refuses was committed" >&2
            exit 1
          fi
          commit "chore: a new place commits"
        ' hooks "$tmp/toolchain" "$tmp/prek" ${lib.getExe config.git-hooks.package}
    '';
  };
}
