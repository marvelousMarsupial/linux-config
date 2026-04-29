
  Then in nvim:

  1. :Lazy sync — pulls in hardtime.nvim + its nui.nvim dep. (lazy.nvim normally auto-installs on next
  startup, but sync is the deterministic version.)
  2. :Lazy — opens the plugin UI; scroll the list and confirm hardtime.nvim is loaded (●).
  3.iarkdown — markdown is on the disabled list): :e
  ~/linux-config/crates/forge/src/main.rs for example.
  4. In{} other pattit's reading your motions:
    - xxxxx → "use dw or similar"
    - wwww → "use W or f<char>"
    - dddd (in normal mode after some dd deletions) → "use <count>dd or visual"

  Useful commands once it's running:
  - :Hardtime toggle — turn the coaching off when you want to do something repetitive on purpose
  - :Hardtime report — see a summary of what it's been catching you on (this is the actually-useful one
   for noticing patterns to retrain)
  - :Hardtime disable / :enable — same as toggle but explicit

