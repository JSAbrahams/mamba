# Nushell configuration for working on Mamba.
#
# Copy this to .config/nushell/config.nu to customise it; that path is gitignored and takes
# precedence, so your own version never shows up as a working-tree change.
#
# Deliberately absent: wrappers around the build, test, lint and coverage commands.
# Those live in devbox.json as `devbox run <script>`, which CI calls too, so duplicating them
# here would just create a second definition that can drift out of step.
# Run `devbox run` with no arguments to list them.

# ---------------------------------------------------------------------------------------------
# Shell behaviour
# ---------------------------------------------------------------------------------------------

# Assign individual keys rather than replacing $env.config wholesale, so defaults survive.
$env.config.show_banner = false

# SQLite history is per-command rather than per-line, and records the working directory, exit
# code and duration alongside each entry. That makes it searchable rather than just scrollable.
$env.config.history.file_format = "sqlite"
$env.config.history.max_size = 1_000_000
$env.config.history.sync_on_enter = true
# Keep history shared across concurrent shells, so a command run in one is visible in another.
$env.config.history.isolation = false
# A leading space keeps a command out of history, for anything with a secret in it.
$env.config.history.ignore_space_prefixed = true

# Fuzzy completion matches `crg-nxt` to `cargo nextest`, which suits long cargo invocations.
$env.config.completions.algorithm = "fuzzy"
$env.config.completions.sort = "smart"
$env.config.completions.case_sensitive = false

# Show externals in a different colour when they actually resolve on PATH. Useful here, because
# it tells you at a glance whether you are inside the devbox environment or outside it.
$env.config.highlight_resolved_externals = true

# Let the terminal track the working directory and mark command boundaries.
$env.config.shell_integration.osc2 = true
$env.config.shell_integration.osc7 = true
$env.config.shell_integration.osc133 = true

# `rm` moves to the trash where the platform supports it. Pass `--permanent` to bypass.
$env.config.rm.always_trash = false

$env.config.cursor_shape.emacs = "line"
$env.config.cursor_shape.vi_insert = "line"
$env.config.cursor_shape.vi_normal = "block"

# Ctrl-o opens the current line in an editor, for anything too long to edit comfortably inline.
$env.config.buffer_editor = ($env.EDITOR? | default "vim")

# ---------------------------------------------------------------------------------------------
# Completions for external commands
# ---------------------------------------------------------------------------------------------

# carapace supplies completions for cargo, git, gh, jq and several hundred others.
# It is provided by devbox. The guard keeps this config working if it is ever removed, in which
# case Nushell falls back to file completion.
if (which carapace | is-not-empty) {
  $env.CARAPACE_BRIDGES = "zsh,fish,bash,inshellisense"

  $env.config.completions.external = {
    enable: true
    max_results: 100
    completer: {|spans|
      carapace $spans.0 nushell ...$spans
        | from json
        | if ($in | default [] | where value =~ '^-.*ERR$' | is-empty) { $in } else { null }
    }
  }
}

# ---------------------------------------------------------------------------------------------
# Project paths
# ---------------------------------------------------------------------------------------------

# Consumed by the coverage module in the Starship prompt below.
$env.PROJECT_TARGET = ($env.WORKSPACE | path join "target")

# ---------------------------------------------------------------------------------------------
# Aliases
# ---------------------------------------------------------------------------------------------

# Git, which devbox does not wrap.
alias gst = git status --short --branch
alias gl = git log --oneline --graph --decorate -20
alias ga = git add
alias gc = git commit
alias gp = git push
alias gd = git diff
alias gds = git diff --staged

# GitHub, for reading the project's open work. See CLAUDE.md for how these feed into context.
alias prs = gh pr list --state open
alias issues = gh issue list --state open

# ---------------------------------------------------------------------------------------------
# Project commands
# ---------------------------------------------------------------------------------------------

# Transpile a single Mamba file and show the Python it produces, without leaving a build
# directory behind. This is the fastest way to answer "what does this compile to".
def mamba-py [
  file: path                  # the .mamba file to transpile
  --annotate (-a)             # emit type annotations, which is still a buggy code path
] {
  let out = (mktemp -d)
  try {
    if $annotate {
      ^cargo run --quiet -- --annotate -i $file -o $out
    } else {
      ^cargo run --quiet -- -i $file -o $out
    }
    ls ($out | path join "**" "*.py") | each {|f| open --raw $f.name } | str join "\n"
  } catch {|e|
    $"transpile failed: ($e.msg)"
  }
}

# Which table-driven test cases are currently ignored, and why. Handy before picking up work,
# since an `ignore[...]` reason is this project's record of what is not implemented yet.
def mamba-ignored [] {
  glob tests/**/*.rs
    | each {|f|
        open --raw $f
          | lines
          | enumerate
          | where item =~ '=> ignore\['
          | each {|l|
              let case = ($l.item | parse --regex '"(?<a>[^"]+)",\s*"(?<b>[^"]+)"')
              {
                file: ($f | path relative-to $env.WORKSPACE)
                line: ($l.index + 1)
                case: (if ($case | is-empty) { "" } else { $"($case.0.a)/($case.0.b)" })
                reason: ($l.item | parse --regex 'ignore\["(?<r>[^"]*)"\]' | get -o 0.r | default "")
              }
            }
      }
    | flatten
}

# ---------------------------------------------------------------------------------------------
# ssh agent
# ---------------------------------------------------------------------------------------------

def --env start-ssh-agent [] {
  ^ssh-agent -c
      | lines
      | first 2
      | parse "setenv {name} {value};"
      | transpose -r
      | into record
      | load-env

  # now prompt user for password
  ssh-add
}

# ---------------------------------------------------------------------------------------------
# Starship prompt
# ---------------------------------------------------------------------------------------------

# Tell Starship which shell to target
$env.STARSHIP_SHELL = "nu"

if (($env.WORKSPACE | path join ".config/starship.toml") | path exists) {
  $env.STARSHIP_CONFIG = ($env.WORKSPACE | path join ".config/starship.toml")
} else {
  $env.STARSHIP_CONFIG = ($env.WORKSPACE | path join ".config/starship.example.toml")
}

# Create a left-side prompt via Starship
def create_left_prompt [] {
  starship prompt --cmd-duration $env.CMD_DURATION_MS --status=$env.LAST_EXIT_CODE
}

# Hook it into Nushell's prompt
$env.PROMPT_COMMAND       = { || create_left_prompt }
$env.PROMPT_COMMAND_RIGHT = ""

$env.PROMPT_INDICATOR           = ""
$env.PROMPT_INDICATOR_VI_INSERT = ": "
$env.PROMPT_INDICATOR_VI_NORMAL = "〉"
$env.PROMPT_MULTILINE_INDICATOR = "::: "
