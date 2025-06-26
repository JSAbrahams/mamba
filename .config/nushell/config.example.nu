$env.config.show_banner = false

$env.PROJECT_TARGET = ($env.WORKSPACE + "/target")

# Aliases

alias gc   = git commit
alias gst  = git status
alias gp   = git push
alias ga   = git add
alias gl   = git log --oneline

alias coverage-lcov = cargo llvm-cov nextest --lcov --summary-only --output-path ($env.PROJECT_TARGET + "/cov.lcov")
alias coverage-json = cargo llvm-cov nextest --json --summary-only --output-path ($env.PROJECT_TARGET + "/cov.json")
alias coverage      = coverage-json
alias cov           = coverage

# Startship prompt

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

# Hook it into Nushell’s prompt
$env.PROMPT_COMMAND       = { || create_left_prompt }
$env.PROMPT_COMMAND_RIGHT = ""

$env.PROMPT_INDICATOR           = ""
$env.PROMPT_INDICATOR_VI_INSERT = ": "
$env.PROMPT_INDICATOR_VI_NORMAL = "〉"
$env.PROMPT_MULTILINE_INDICATOR = "::: "
