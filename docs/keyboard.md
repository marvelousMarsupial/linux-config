# Keyboard

## Home row modifiers

Setup installs keyd and `system/keyd/laptop.conf` to `/etc/keyd/laptop.conf`.
On the built-in laptop keyboard, a tapped key types its letter, and a held key
acts as a modifier:

| Keys | Held |
| --- | --- |
| `a` and `;` | Super |
| `s` and `l` | Alt |
| `d` and `k` | Control |
| `f` and `j` | Shift |

keyd matches the keyboard by ID. `0001:0001` is the usual internal keyboard
("AT Translated Set 2 keyboard"), so external keyboards, such as a split board
with its own home row modifiers, are left alone. To check the ID, run
`sudo keyd monitor` and type on the laptop keys. If yours differs, edit the
`[ids]` line in `/etc/keyd/laptop.conf` and run `sudo keyd reload`. A rerun of
setup backs up the edited file and restores the repo version, so change the
repo copy too.

`lettermod` needs keyd 2.5 or newer. It keeps a key as a letter when it comes
within 150 ms of the previous keypress, so fast typing stays letters. A key
held 200 ms becomes a modifier. Raise the numbers if letters turn into
modifiers by mistake.

keyd hands the remapped keys to sway through its own virtual keyboard. The
sway `input type:keyboard` block still applies to it. A rule that names the
laptop keyboard by its identifier does not, so point such rules at the keyd
virtual keyboard instead (`swaymsg -t get_inputs` lists it).

If Debian does not offer keyd, setup prints a notice and skips this step.
