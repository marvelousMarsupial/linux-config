# PATH - set BEFORE the sway auto-exec so sway and every child it spawns
# (.desktop files run by fuzzel, scripts called from binds, etc.) inherit
# ~/.local/bin and the sbin paths. Without this our binds hit "command not
# found" because Debian's default PATH for non-root users is bare bones.
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/local/sbin:/usr/sbin:/sbin:/usr/local/bin:/usr/bin:/sbin:/bin"

# Sway and every app it launches inherit the same toolkit settings.
if [[ -r "$HOME/.config/environment.d/appearance.conf" ]]; then
  source "$HOME/.config/environment.d/appearance.conf"
  export GTK_THEME
fi

# Auto-start Sway on tty1
if [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ] && [ "$XDG_VTNR" = "1" ] && command -v sway >/dev/null; then
  exec systemd-cat --identifier=sway sway
fi

# Locale for Unicode and emoji. LANG alone leaves LC_* overrides available.
export LANG=en_US.UTF-8

export GPG_TTY=$(tty)

# XDG runtime dir for desktop services
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"

# D-Bus session (for headless sway)
# Import only the address; runtime files are data, never shell code.
if [[ -f "$XDG_RUNTIME_DIR/dbus-env" && -O "$XDG_RUNTIME_DIR/dbus-env" && ! -L "$XDG_RUNTIME_DIR/dbus-env" ]]; then
  while IFS= read -r _dbus_line; do
    if [[ "$_dbus_line" == 'export DBUS_SESSION_BUS_ADDRESS='* ]]; then
      _dbus_address="${_dbus_line#export DBUS_SESSION_BUS_ADDRESS=}"
      _dbus_address="${_dbus_address#\"}"
      _dbus_address="${_dbus_address%\"}"
      _dbus_address="${_dbus_address#\'}"
      _dbus_address="${_dbus_address%\'}"
      if [[ "$_dbus_address" == unix:* && "$_dbus_address" != *'$'* && "$_dbus_address" != *'`'* ]]; then
        export DBUS_SESSION_BUS_ADDRESS="$_dbus_address"
      fi
    fi
  done < "$XDG_RUNTIME_DIR/dbus-env"
  unset _dbus_line _dbus_address
fi

# Completion first; fzf-tab wraps it before the other plugins wrap ZLE widgets.
autoload -Uz compinit
(( ${+_comps} )) || compinit
zstyle ':completion:*:descriptions' format '[%d]'
if command -v fzf >/dev/null && [[ -r "$HOME/.local/share/zsh/fzf-tab/fzf-tab.plugin.zsh" ]]; then
  zstyle ':completion:*' menu no
  zstyle ':fzf-tab:*' fzf-flags --height=60% --layout=reverse --border
  zstyle ':fzf-tab:complete:*' fzf-preview 'if [[ -d $realpath ]]; then
    ls -Ahl --color=always -- "$realpath"
  elif [[ -f $realpath ]]; then
    batcat --color=always --paging=never --style=numbers --line-range=:200 -- "$realpath"
  fi'
  source "$HOME/.local/share/zsh/fzf-tab/fzf-tab.plugin.zsh"
fi

# Load only fzf's key bindings, leaving Tab completion to fzf-tab. Enable Ctrl+R
# history search without adding the separate file/directory picker shortcuts.
if command -v fzf >/dev/null && [[ -r /usr/share/doc/fzf/examples/key-bindings.zsh ]]; then
  FZF_CTRL_T_COMMAND='' FZF_ALT_C_COMMAND='' source /usr/share/doc/fzf/examples/key-bindings.zsh
fi

# Plugins: suggestions try history first, then the completion system.
ZSH_AUTOSUGGEST_STRATEGY=(history completion)
source /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh
source /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh

# Starship prompt
eval "$(starship init zsh)"

# fnm (Fast Node Manager) shell integration. Loaded conditionally so
# the same .zshrc works with or without fnm installed.
# `command -v fnm` keeps initialization harmless when
# absent. `--use-on-cd` makes fnm auto-switch the active Node version
# when you `cd` into a directory containing a `.nvmrc` /
# `.node-version` file, which is the whole point of using fnm over a
# manual `nvm use`. The init also exports `FNM_MULTISHELL_PATH` and
# prepends it to `$PATH` (this happens AFTER our explicit PATH export
# at the top of the file, which is intentional — fnm's PATH manipulation
# needs to win for the active Node version to take precedence over
# any system Node).
[[ -d "$HOME/.local/share/fnm" ]] && export PATH="$HOME/.local/share/fnm:$PATH"
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

# Arrow keys search from the start of the line through the cursor. Keep the
# cursor at the prefix so repeated Up/Down presses use the same search text.
# Cover normal and application cursor keys in Emacs and vi insert mode.
for _history_keymap in emacs viins; do
  bindkey -M "$_history_keymap" '^[[A' history-beginning-search-backward
  bindkey -M "$_history_keymap" '^[OA' history-beginning-search-backward
  bindkey -M "$_history_keymap" '^[[B' history-beginning-search-forward
  bindkey -M "$_history_keymap" '^[OB' history-beginning-search-forward
done
unset _history_keymap

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

# Foot shell integration: mark command output boundaries.
if [[ -o interactive && "$TERM" == foot* ]]; then
  autoload -Uz add-zsh-hook
  _foot_out_end()   { builtin zle || print -n '\e]133;D\e\\' }
  _foot_out_start() { print -n '\e]133;C\e\\' }
  add-zsh-hook precmd _foot_out_end
  add-zsh-hook preexec _foot_out_start
fi
