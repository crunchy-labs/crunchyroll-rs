#!/usr/bin/env bash

set -euo pipefail

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

log() { echo ":: $*" >&2; }
fail() { log "ERROR: $*"; exit 1; }

version_gt() {
  [ "$1" != "$2" ] && [ "$(printf '%s\n%s\n' "$1" "$2" | sort -V | tail -1)" = "$1" ]
}

aptoide_info() {
  feature="$1"
  response="$(curl -sSf "https://ws2-cache.aptoide.com/api/7/apps/search?query=crunchyroll&cdn=web&q=bXlDUFU9YXJtNjQtdjhhLGFybWVhYmktdjdhLGFybWVhYmkmbGVhbmJhY2s9MA&aab=1&limit=1&apk_required_features=${feature}")"

  vername="$(echo "$response" | jq -r '.datalist.list.[0].file.vername // empty')"
  vercode="$(echo "$response" | jq -r '.datalist.list.[0].file.vercode // empty')"
  url="$(echo "$response" | jq -r '.datalist.list.[0].file.path // empty')"

  [ -n "$vername" ] && [ -n "$vercode" ] && [ -n "$url" ] || fail "could not parse aptoide info for feature=$feature"

  echo "$vername $vercode $url"
}

extract_credentials() {
  apk_path="$1"

  output="$(curl -sSf https://raw.githubusercontent.com/crunchy-labs/crunchyroll-scripts/refs/heads/master/apk-credentials-extract.sh | bash -s -- "$apk_path" --info-stderr)"

  client_id="$(echo "$output" | grep -oP '(?<=^client id: ).+')"
  basic_auth="$(echo "$output" | grep -oP '(?<=basic auth credentials: ).+')"

  [ -n "$client_id" ] && [ -n "$basic_auth" ] || fail "could not extract credentials from $apk_path"

  echo "$client_id $basic_auth"
}

github_output() {
  key="$1"
  value="$2"

  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    echo "${key}=${value}" >> "$GITHUB_OUTPUT"
  fi
  log "${key}=${value}"
}

update_tv() {
  vername="$1"
  vercode="$2"
  url="$3"

  crate_vername="$(grep -oP '(?<=const ANDROID_TV_USER_AGENT: &'"'"'static str = "Crunchyroll/ANDROIDTV/)[^_]+' src/crunchyroll.rs || true)"
  crate_vername="${crate_vername:-0}"

  if ! version_gt "$vername" "$crate_vername"; then
    log "TV version unchanged: $vername"
    echo "false"
    return
  fi

  log "New TV version detected: $vername (was: $crate_vername)"
  log "Downloading TV apk"
  curl -sSfL -o "$SCRATCH/tv.apk" "$url"

  log "Extracting TV credentials"
  creds="$(extract_credentials "$SCRATCH/tv.apk")" || fail "could not extract TV credentials"
  read -r _client_id basic_auth <<< "$creds"

  log "Updating src/crunchyroll.rs"
  sed -i -E "s|(pub const ANDROID_TV_BASIC_AUTH_TOKEN: &'static str = \").+(\";)|\1${basic_auth}\2|" src/crunchyroll.rs
  sed -i -E "s|(pub const ANDROID_TV_USER_AGENT: &'static str = \"Crunchyroll/ANDROIDTV/).+( \()|\1${vername}_${vercode}\2|" src/crunchyroll.rs

  echo "true"
}

update_phone() {
  vername="$1"
  _vercode="$2"
  url="$3"

  cur_ua_ver="$(grep -oP '(?<=const ANDROID_PHONE_USER_AGENT: &str = "Crunchyroll/)[0-9.]+' examples/sso-login/src/main.rs || true)"
  cur_ua_ver="${cur_ua_ver:-0}"

  if ! version_gt "$vername" "$cur_ua_ver"; then
    log "Phone version unchanged: $vername"
    echo "false"
    return
  fi

  log "New phone version detected: $vername (was: $cur_ua_ver)"
  log "Downloading phone apk"
  curl -SfL -o "$SCRATCH/phone.apk" "$url"

  log "Extracting phone credentials"
  creds="$(extract_credentials "$SCRATCH/phone.apk")" || fail "could not extract phone credentials"
  read -r sso_client_id basic_auth <<< "$creds"

  log "Updating examples/sso-login/src/main.rs"
  sed -i -E "s|(const ANDROID_PHONE_BASIC_AUTH: &str = \").+(\";)|\1${basic_auth}\2|" examples/sso-login/src/main.rs
  sed -i -E "s|(const ANDROID_PHONE_SSO_CLIENT_ID: &str = \").+(\";)|\1${sso_client_id}\2|" examples/sso-login/src/main.rs
  sed -i -E "s|(const ANDROID_PHONE_USER_AGENT: &str = \"Crunchyroll/)[0-9.]+|\1${vername}|" examples/sso-login/src/main.rs

  echo "true"
}

# === main ===
log "Fetching TV apk info"
info="$(aptoide_info "android.software.leanback")" || fail "could not fetch TV apk info"
read -r tv_vername tv_vercode tv_url <<< "$info"

log "Fetching phone apk info"
info="$(aptoide_info "android.hardware.faketouch")" || fail "could not fetch phone apk info"
read -r phone_vername phone_vercode phone_url <<< "$info"

tv_updated="$(update_tv "$tv_vername" "$tv_vercode" "$tv_url")"
phone_updated="$(update_phone "$phone_vername" "$phone_vercode" "$phone_url")"

if [ "$tv_updated" = "true" ] || [ "$phone_updated" = "true" ]; then
  changed="true"
else
  changed="false"
fi
github_output changed "$changed"
github_output tv_updated "$tv_updated"
github_output phone_updated "$phone_updated"
github_output tv_vername "$tv_vername"
github_output phone_vername "$phone_vername"

if [ "$changed" = "false" ]; then
  log "Nothing to update"
fi
