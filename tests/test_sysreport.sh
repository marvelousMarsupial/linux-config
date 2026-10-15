#!/usr/bin/env bash
# Exercise escaping, section failure isolation, private modes and retention.
set -euo pipefail
repo=$(cd -- "$(dirname -- "$0")/.." && pwd)
temporary=$(mktemp -d)
trap 'rm -rf -- "$temporary"' EXIT
mkdir -p "$temporary/.local/bin" "$temporary/.local/share/sysreport"
cat > "$temporary/.local/bin/hostname" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "<script>&\"'"
STUB
cat > "$temporary/.local/bin/systemctl" <<'STUB'
#!/usr/bin/env bash
echo 'fixture: systemd unavailable' >&2
exit 1
STUB
chmod +x "$temporary/.local/bin/hostname" "$temporary/.local/bin/systemctl"
for n in $(seq -w 1 16); do touch "$temporary/.local/share/sysreport/report-20000101T0000$n.html"; done
HOME="$temporary" "$repo/dotfiles/.local/bin/sysreport" >/dev/null
page="$temporary/.local/share/sysreport/latest.html"
grep -q '&lt;script&gt;&amp;&quot;&#39;' "$page"
! grep -q '<script>' "$page"
grep -q 'Network basics' "$page"
grep -q 'systemctl unavailable or failed' "$page"
grep -q '<article class="tile warn"><h3>Backups</h3>' "$page"
! grep -q '<article class="tile ok"><h3>Updates</h3>' "$page"
grep -q '<details><summary>Updates</summary>' "$page"
[[ $(grep -c '<article class="tile ' "$page") == 8 ]]
[[ $(stat -c %a "$page") == 600 ]]
[[ $(stat -c %a "$(dirname "$page")") == 700 ]]
[[ $(find "$(dirname "$page")" -name 'report-*.html' | wc -l) == 14 ]]
! grep -q 'Green: confirmed' "$page"
before=$(sha256sum "$page")
HOME="$temporary" "$repo/dotfiles/.local/bin/sysreport" --demo >/dev/null
[[ $(sha256sum "$page") == "$before" ]]
[[ $(grep -c '<article class="tile ok">' "$(dirname "$page")/demo.html") == 8 ]]
grep -q 'Demo: sample data' "$(dirname "$page")/demo.html"
[[ $(stat -c %a "$(dirname "$page")/demo.html") == 600 ]]
# Healthy service responses must turn the real Services tile green.
cat > "$temporary/.local/bin/systemctl" <<'STUB'
#!/usr/bin/env bash
if [[ " $* " == *' show '* ]]; then
    if [[ " $* " == *' --value '* ]]; then
        echo success
    else
        printf '%s\n' 'Result=success' 'ExecMainStatus=0'
    fi
fi
exit 0
STUB
HOME="$temporary" "$repo/dotfiles/.local/bin/sysreport" >/dev/null
grep -q '<article class="tile ok"><h3>Services</h3>' "$page"
grep -q '<article class="tile warn"><h3>Backups</h3>' "$page"
# A failed SMART query must not hide memory and disk usage.
cat > "$temporary/.local/bin/smartctl" <<'STUB'
#!/usr/bin/env bash
echo 'Smartctl open device failed: Permission denied' >&2
exit 2
STUB
cat > "$temporary/.local/bin/lsblk" <<'STUB'
#!/usr/bin/env bash
echo 'vda disk'
STUB
cat > "$temporary/.local/bin/df" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' 'Filesystem Type Size Used Avail Use% Mounted on' '/dev/vda ext4 100G 20G 80G 20% /'
STUB
chmod +x "$temporary/.local/bin/smartctl" "$temporary/.local/bin/lsblk" "$temporary/.local/bin/df"
HOME="$temporary" "$repo/dotfiles/.local/bin/sysreport" >/dev/null
grep -q 'ERROR: command failed (exit 2): smartctl' "$page"
grep -q '<article class="tile ok"><h3>Resources</h3>' "$page"
grep -q '<article class="tile warn"><h3>Disk health</h3>' "$page"
# An OpenSnitch database is read directly and read-only.
mkdir -p "$temporary/.config/opensnitch"
printf '[database]\nfile=%s\n' "$temporary/opensnitch.db" > "$temporary/.config/opensnitch/settings.conf"
python3 - "$temporary/opensnitch.db" <<'PY'
import sqlite3, sys
db = sqlite3.connect(sys.argv[1])
db.execute("CREATE TABLE connections (time TEXT, action TEXT, process TEXT, dst_host TEXT, dst_ip TEXT, dst_port INTEGER)")
db.execute("CREATE TABLE rules (created TEXT, name TEXT, action TEXT, operator_operand TEXT, operator_data TEXT)")
db.execute("INSERT INTO connections VALUES (datetime('now','localtime'), 'deny', '/usr/bin/curl', 'example.invalid', '192.0.2.1', 443)")
db.commit()
PY
HOME="$temporary" "$repo/dotfiles/.local/bin/sysreport" >/dev/null
grep -q 'Saved connections: 1' "$page"
grep -q 'example.invalid' "$page"
grep -q '<article class="tile ok"><h3>OpenSnitch</h3>' "$page"
bash "$repo/scripts/install-receipt" "$repo" "$temporary" fixture
grep -q 'Installation scope: fixture' "$temporary/.local/state/linux-config/install-info"
[[ $(stat -c %a "$temporary/.local/state/linux-config/install-info") == 600 ]]
echo 'Escaping, failure isolation, modes, retention, OpenSnitch and receipt checks passed.'
