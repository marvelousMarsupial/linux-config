# PATH - set BEFORE the sway auto-exec so sway and every child it spawns
# (.desktop files run by fuzzel, scripts called from binds, etc.) inherit
# ~/.local/bin and the sbin paths. Without this our binds hit "command not
# found" because Debian's default PATH for non-root users is bare bones.
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/local/sbin:/usr/sbin:/sbin:/usr/local/bin:/usr/bin:/sbin:/bin"

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

# Locale (required for Unicode/emoji in tmux)
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

export PASSWORD_STORE_DIR="$HOME/.password-store"

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

# fnm (Fast Node Manager) shell integration. Loaded conditionally so
# the same .zshrc is portable across hosts and containers — fnm ships
# in the dev container image but isn't installed on the host or on
# remote servers, and `command -v fnm` keeps the file innocuous when
# absent. `--use-on-cd` makes fnm auto-switch the active Node version
# when you `cd` into a directory containing a `.nvmrc` /
# `.node-version` file, which is the whole point of using fnm over a
# manual `nvm use`. The init also exports `FNM_MULTISHELL_PATH` and
# prepends it to `$PATH` (this happens AFTER our explicit PATH export
# at the top of the file, which is intentional — fnm's PATH manipulation
# needs to win for the active Node version to take precedence over
# any system Node).
if command -v fnm >/dev/null; then
  eval "$(fnm env --use-on-cd --shell zsh)"
fi

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
