-- hardtime.nvim — coach the user out of inefficient motions.
--
-- Watches for sequences like `jjjjj` / `wwww` / `xxxxx` and replaces
-- them with a brief notification suggesting the better keystroke
-- (`5j` / `}` / `<C-d>`, `W` / `f<char>`, `dw` / `cc`, …). Over time
-- this re-trains the muscle memory away from "press the same key
-- repeatedly until you get there" and toward modal vim's actual
-- vocabulary.
--
-- Tradeoff worth being honest about: hardtime *is* annoying on
-- purpose. It will block your fifth `j` press for a fraction of a
-- second and pop a hint. That friction is the feature — without it
-- the plugin does nothing — but it'd be miserable in any context
-- where repeated low-level motion is *appropriate* (typing prose,
-- scrolling through a file picker, editing a markdown note line by
-- line). The `disabled_filetypes` list below extends the upstream
-- defaults to cover the user's specific noisy buffers (Obsidian /
-- markdown notes, telescope pickers, fterm shells, yazi popups, …)
-- so the coaching only fires while editing actual code.
--
-- Repo: https://github.com/m4xshen/hardtime.nvim
-- Notification UI: requires MunifTanjim/nui.nvim (declared as a dep
-- below so lazy.nvim pulls it in even if no other plugin uses it).
return {
  "m4xshen/hardtime.nvim",
  -- VeryLazy: load after the UI is up so the first `:e <file>` isn't
  -- delayed by hardtime's setup. The plugin only matters once the
  -- user is actually editing.
  event = "VeryLazy",
  dependencies = { "MunifTanjim/nui.nvim" },
  opts = {
    -- Defaults that match upstream:
    --   max_time   = 1000  (window in ms over which key repeats are counted)
    --   max_count  = 3     (allow `jjj` — three is a real rhythm — but
    --                       block the fourth so `jjjj` triggers a hint)
    --   disable_mouse = true (force keyboard discipline; mouse clicks
    --                         in nvim windows are silently no-op'd)
    --   hint = true        (virtual-text hints for "you could have
    --                       used X here" on motions that *worked* but
    --                       were suboptimal)
    --   notification = true (popup when a restricted key is pressed
    --                        beyond max_count — this is the one that
    --                        catches `jjjjj` etc.)
    --   restriction_mode = "block" (the fifth `j` is dropped, not
    --                                 just notified — the friction is
    --                                 what teaches the motion)
    --
    -- We set these explicitly rather than relying on defaults so a
    -- future upstream change to a default doesn't silently shift the
    -- coaching aggressiveness from under us.
    max_time = 1000,
    max_count = 3,
    disable_mouse = true,
    hint = true,
    notification = true,
    restriction_mode = "block",

    -- Filetypes where hardtime should NOT fire. The upstream default
    -- list covers `qf`, `netrw`, `NvimTree`, `lazy`, `mason`,
    -- `oil`, `noice`, `notify`, etc. We extend it with this user's
    -- specific noisy contexts:
    --
    --   * markdown — Obsidian notes are prose; jjjj through a
    --     paragraph is the right motion, not a sin.
    --   * TelescopePrompt — picker navigation is k/j by design.
    --   * fterm — the floating terminal (plugins/fterm.lua); shell
    --     interaction has its own keystroke economics.
    --   * yazi — the yazi.nvim popup (plugins/editor.lua); same
    --     reasoning as TelescopePrompt.
    --   * gitcommit — short commit messages, no need to coach.
    --   * help — help buffers are read-only browsing.
    --
    -- Append rather than replace so we get upstream's curated
    -- baseline plus our additions.
    disabled_filetypes = {
      "qf",
      "netrw",
      "NvimTree",
      "lazy",
      "mason",
      "oil",
      "noice",
      "notify",
      "TelescopePrompt",
      "TelescopeResults",
      "markdown",
      "fterm",
      "yazi",
      "gitcommit",
      "help",
    },
  },
}
