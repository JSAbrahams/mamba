$env.config.show_banner = false

# Aliases

alias gc   = git commit
alias gst  = git status
alias gp   = git push
alias ga   = git add
alias gl   = git log --oneline

# Startship prompt

# Tell Starship which shell to target
$env.STARSHIP_SHELL = "nu"

# Create a left-side prompt via Starship
def create_left_prompt [] {
  starship prompt --cmd-duration $env.CMD_DURATION_MS --status=$env.LAST_EXIT_CODE
}

# Hook it into Nushell’s prompt
$env.PROMPT_COMMAND       = { || create_left_prompt }
$env.PROMPT_COMMAND_RIGHT = ""

# Tweak the indicators (optional)
$env.PROMPT_INDICATOR           = ""
$env.PROMPT_INDICATOR_VI_INSERT = ": "
$env.PROMPT_INDICATOR_VI_NORMAL = "〉"
$env.PROMPT_MULTILINE_INDICATOR = "::: "
