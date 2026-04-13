# Auto-start Sway on tty1 (desktop role only)
if [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ] && [ "$XDG_VTNR" = "1" ] && command -v sway >/dev/null; then
  exec sway
fi

# Auto-start tmux (create new session for each terminal)
if command -v tmux >/dev/null && [ -z "$TMUX" ] && [ -n "$PS1" ] && [[ ! "$TERM" =~ screen ]] && [[ ! "$TERM" =~ tmux ]]; then
  # Clean up idle unattached sessions (1 window, 1 pane, just shell)
  tmux list-sessions -F '#{session_name} #{session_attached} #{session_windows}' 2>/dev/null | \
    while read name attached windows; do
      if [[ "$attached" == "0" && "$windows" == "1" ]]; then
        panes=$(tmux list-panes -t "$name" -F '#{pane_current_command}' 2>/dev/null)
        if [[ "$panes" == "zsh" || "$panes" == "bash" ]]; then
          tmux kill-session -t "$name" 2>/dev/null
        fi
      fi
    done
  tmux new-session
fi

# PATH - our scripts first, then .local/bin for pip/cargo/etc
export PATH="$HOME/linux-config/bin:$HOME/.local/bin:$PATH"
export PASSWORD_STORE_DIR="$HOME/.password-store"

# Neovim runtime (only set if path exists - containers use wrapper script)
[[ -d "$HOME/.local/nvim-runtime" ]] && export VIMRUNTIME="$HOME/.local/nvim-runtime/usr/share/nvim/runtime"
export PASSWORD_STORE_CLIP_CMD="wl-copy"
export GPG_TTY=$(tty)

# XDG runtime dir (required for rootless podman, pipewire, etc)
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"

# D-Bus session (for headless sway)
[[ -f "$XDG_RUNTIME_DIR/dbus-env" ]] && source "$XDG_RUNTIME_DIR/dbus-env"

# Plugins
source /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh
source /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh

# Starship prompt
eval "$(starship init zsh)"

# History
HISTFILE=~/.zsh_history
HISTSIZE=10000
SAVEHIST=10000
setopt appendhistory
setopt sharehistory
setopt hist_ignore_dups
setopt hist_ignore_space

# Yazi wrapper - cd to directory on exit
function y() {
  local tmp="$(mktemp -t "yazi-cwd.XXXXXX")" cwd
  yazi "$@" --cwd-file="$tmp"
  if cwd="$(command cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
    builtin cd -- "$cwd"
  fi
  rm -f -- "$tmp"
}

# Source aliases
[[ -f ~/.zsh_aliases ]] && source ~/.zsh_aliases

# Alias reminder - inline terminal hints (opt-in)
# Shows dim "↳ gs" hint when you type "git status"
# Enable: export ALIAS_REMINDER=1
[[ -f ~/.zsh_alias_reminder ]] && source ~/.zsh_alias_reminder

# MOTD - show on interactive login
[[ -o login ]] && command -v motd >/dev/null && motd
alias pshell='TERM=xterm podman exec -it'
