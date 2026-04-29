-- nvim-notify — replaces nvim's default `:messages`-style notification
-- area with a top-right toast UI: rounded borders, level-colored
-- left bar, fade-in / slide-out animation, auto-dismiss after a
-- short timeout. Anything that calls `vim.notify(...)` (LSP
-- diagnostics, hardtime.nvim's "you pressed h too soon", lazy.nvim
-- install reports, custom keymap notifications) inherits the new
-- look without per-caller changes.
--
-- Why this plugin and not noice.nvim: noice.nvim is the "replace
-- the entire UI surface" option (cmdline, message area, popups,
-- search, etc.) and is great but heavier — more config surface,
-- more edge cases with cmdline integrations. nvim-notify is the
-- targeted "make notifications look nice" tool, which is the
-- specific need here. If we ever want the cmdline overhaul too,
-- noice.nvim composes on top of nvim-notify cleanly.
--
-- The trickier piece — preserving the existing vim.notify filter:
-- `lua/config/options.lua` installs a wrapper around vim.notify
-- that drops noisy mason / lspconfig / "not found" startup
-- messages. nvim-notify, when it loads, calls
-- `vim.notify = require("notify")`, which would *replace* (not
-- compose with) that wrapper, silently restoring the spam. The
-- `config` function below sets up nvim-notify and then re-applies
-- the same filter on top of the new vim.notify, so the visual
-- upgrade lands without regressing the noise-suppression. (The
-- options.lua wrapper still does the early-startup window before
-- lazy.nvim has loaded any plugin; this re-wrap takes over after.)
return {
  "rcarriga/nvim-notify",
  -- VeryLazy: don't delay the first frame for the notification
  -- system. Notifications that fire before this loads still go
  -- through the default vim.notify (the options.lua wrapper),
  -- which is acceptable — it's the "during nvim startup" window
  -- and there's nothing visually distinguishable to the user.
  event = "VeryLazy",
  config = function()
    local notify = require("notify")
    notify.setup({
      -- Visual settings, all chosen to match the catppuccin-mocha
      -- theme rendered into colorscheme.lua and keep the toast
      -- surface unobtrusive while still readable at a glance.
      --
      -- `render = "compact"` — single-line summary on the first
      --   row plus message body below; the "default" renderer
      --   wastes vertical space with a big icon column. Compact
      --   is the right choice for short notifications like
      --   hardtime hints and LSP diagnostics counts.
      -- `stages = "fade_in_slide_out"` — smooth animation, less
      --   jarring than the default pop-in. Frame budget is the
      --   `fps` field below.
      -- `timeout = 3000` ms — enough time to actually read a
      --   short notification, not so long that stale info
      --   stacks up while you're focused on code.
      -- `top_down = false` — newest notification at the top,
      --   older ones slide down. Reads naturally for a stack.
      -- `background_colour = "#1e1e2e"` — catppuccin-mocha base.
      --   Without this nvim-notify warns at startup
      --   ("Treating background as default ...") because it
      --   can't infer the background from the Normal hl group
      --   in some scheme load orders. Pinning the literal silences
      --   the warning and avoids a transparent-bg fallback that
      --   would look odd over the editor window.
      -- `fps = 30` — animation framerate. Default; pinned so a
      --   future upstream default change doesn't shift it.
      -- `level = vim.log.levels.INFO` — drop DEBUG/TRACE entirely.
      --   Plugins occasionally emit verbose debug notifications
      --   and surfacing them in the toast UI is just noise.
      render = "compact",
      stages = "fade_in_slide_out",
      timeout = 3000,
      top_down = false,
      background_colour = "#1e1e2e",
      fps = 30,
      level = vim.log.levels.INFO,
    })

    -- Re-apply the same filter that lives in lua/config/options.lua,
    -- but this time on top of nvim-notify (which has just replaced
    -- vim.notify with itself). Keep the patterns in sync with
    -- options.lua — if you add a new pattern there, add it here too,
    -- otherwise the toast UI will start showing what the legacy
    -- vim.notify wrapper was filtering out.
    --
    -- We capture `notify` (the module's notify function, what
    -- nvim-notify itself sets vim.notify to) so we forward to it
    -- directly, sidestepping any other plugin that might also have
    -- monkey-patched vim.notify in the meantime. Functionally
    -- equivalent to forwarding to `vim.notify` after this assignment,
    -- but more robust to load-order surprises.
    vim.notify = function(msg, level, opts)
      if type(msg) == "string" then
        if
          msg:match("mason")
          or msg:match("lspconfig")
          or msg:match("not found")
          or msg:match("spawn")
        then
          return
        end
      end
      notify(msg, level, opts)
    end
  end,
}
