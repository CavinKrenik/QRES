# QRES Release Workflow Checklist

**Target Version:** v16.5.0  
**Release Type:** Feature (The Immune System)

---

## Pre-Release

### 1. Version Bumping

- [ ] Update `crates/qres_core/Cargo.toml`
  ```toml
  version = "16.5.0"
  ```

- [ ] Update `crates/qres_daemon/Cargo.toml`
  ```toml
  version = "16.5.0"
  ```

- [ ] Update `web/src-tauri/Cargo.toml`
  ```toml
  version = "16.5.0"
  ```

- [ ] Update `web/src-tauri/tauri.conf.json`
  ```json
  "version": "16.5.0"
  ```

- [ ] Update `CITATION.cff`
  ```yaml
  version: v16.5.0
  date-released: 2026-01-14
  ```

### 2. Documentation Updates

- [ ] Update `README.md`
    - [ ] Update Version references (v16.5)
    - [ ] Update Feature list (Immune System, Ghost Protocol)
    - [ ] Update Citation
- [ ] Update `docs/IMPLEMENTATION_STATUS.md`
- [ ] Update `docs/SECURITY_ROADMAP.md`
- [ ] Create/Update Release Notes
    - [ ] Prepend new notes to `docs/releases/RELEASE_NOTES.md`

### 3. Changelog & Commits

- [ ] Generate changelog from commits
- [ ] Review `git log` to ensure all security features are committed

---

## Testing

### 4. Automated Tests

- [ ] Run Rust unit tests (Core & Daemon):
  ```bash
  cargo test --workspace
  ```

- [ ] Run Security Verification:
  ```bash
  cargo test -p qres_core --lib privacy
  cargo test -p qres_core --lib secure_agg
  cargo test -p qres_core --lib zk_proofs
  ```

- [ ] Run Frontend checks:
  ```bash
  cd web && npm run check
  ```

### 5. Manual Testing

- [ ] Verify `reputation.json` creation on startup
- [ ] Verify "Gatekeeper" logs when entropy is high
- [ ] Check "GhostUpdate" packet serialization (if possible via logs)

---

## Release

### 6. Git Operations

- [ ] Ensure working directory is clean
- [ ] Create release commit:
  ```bash
  git add -A
  git commit -m "chore: release v16.5.0"
  ```

- [ ] Create annotated tag:
  ```bash
  git tag -a v16.5.0 -m "QRES v16.5.0 - The Immune System"
  ```

- [ ] Push with tags:
  ```bash
  git push origin main --tags
  ```

### 7. GitHub Release

- [ ] Go to [Releases](https://github.com/CavinKrenik/QRES/releases)
- [ ] Click "Draft a new release"
- [ ] Select tag `v16.5.0`
- [ ] Title: `QRES v16.5.0 - The Immune System`
- [ ] Copy content from `docs/releases/RELEASE_NOTES.md` (Top section)
- [ ] Publish release

### 8. Zenodo DOI

- [ ] Zenodo auto-creates DOI
- [ ] Update `README.md` badges if DOI changes (usually stays same concept DOI, version DOI updates automatically)

---

## Post-Release

### 9. Verification

- [ ] Confirm GitHub Actions passed
- [ ] Verify docs render correctly

---

## Rollback

```bash
git push origin :refs/tags/v16.5.0
git tag -d v16.5.0
git revert HEAD
git push origin main
```

**Responsible:** @CavinKrenik  
**Estimated Time:** ~1 hour
