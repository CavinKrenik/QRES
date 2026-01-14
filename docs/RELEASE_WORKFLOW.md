# QRES Release Workflow Checklist

**Target Version:** v15.3.0  
**Release Type:** Minor (Feature Release)

---

## Pre-Release

### 1. Version Bumping

- [ ] Update `web/package.json`
- [ ] Update `web/src-tauri/Cargo.toml`n": "15.3.0"
  ```

- [ ] Update `web/src-tauri/tauri.conf.json`
  ```json
  "version": "15.3.0"
  ```

- [ ] Update `bindings/python/pyproject.toml` (if Python bindings changed)
  ```toml
  version = "15.3.0"
  ```

- [ ] Update `crates/qres_core/Cargo.toml`
  ```toml
  version = "15.3.0"
  ```

- [ ] Update `CITATION.cff`
  ```yaml
  version: v15.3.0
  date-released: 2026-01-11
  ```

### 2. Documentation Updates

- [ ] Update `README.md` Quick Start (if commands changed)
- [ ] Update `web/README.md` with new features
- [ ] Add entry to `docs/IMPLEMENTATION_STATUS.md` version table
- [ ] Add entry to `docs/ROADMAP.md` version history
- [ ] Create `docs/releases/RELEASE_v15.3.0.md`
- [ ] Update `docs/BENCHMARKS.md` with new metrics

### 3. Changelog

- [ ] Generate changelog from commits:
  ```bash
  git log v15.2.0..HEAD --oneline > CHANGELOG_DRAFT.md
  ```
- [ ] Categorize into: Features, Improvements, Bug Fixes, Breaking Changes

---

## Testing

### 4. Automated Tests

- [ ] Run Rust unit tests:
  ```bash
  cargo test --workspace
  ```

- [ ] Run Python binding tests:
  ```bash
  cd bindings/python && pytest tests/
  ```

- [ ] Run WASM build verification:
  ```bash
  cd crates/qres_wasm && wasm-pack build --target web
  ```

- [ ] Run frontend checks:
  ```bash
  cd web && npm run check
  ```

### 5. Manual Testing

- [ ] Launch dev server: `npm run dev`
- [ ] Verify IoT Dashboard loads
- [ ] Test "Connect to Swarm" toggle
- [ ] Test "Trigger Regime Change" button
- [ ] Navigate to all tabs (IoT Stream, Network Map, Neural Graph)
- [ ] Test Tauri build: `npm run tauri build`

---

## Release

### 6. Git Operations

- [ ] Ensure working directory is clean:
  ```bash
  git status
  ```

- [ ] Create release commit:
  ```bash
  git add -A
  git commit -m "chore: release v15.3.0"
  ```

- [ ] Create annotated tag:
  ```bash
  git tag -a v15.3.0 -m "QRES Edge Monitor v15.3.0 - Edge Visualization"
  ```

- [ ] Push with tags:
  ```bash
  git push origin main --tags
  ```

### 7. GitHub Release

- [ ] Go to [Releases](https://github.com/CavinKrenik/QRES/releases)
- [ ] Click "Draft a new release"
- [ ] Select tag `v15.3.0`
- [ ] Title: `QRES Edge Monitor v15.3.0`
- [ ] Copy content from `docs/releases/RELEASE_v15.3.0.md`
- [ ] Attach binaries (if applicable)
- [ ] Publish release

### 8. Zenodo DOI (Optional)

- [ ] Zenodo auto-creates DOI from GitHub release
- [ ] Update `CITATION.cff` with new DOI
- [ ] Update README badges

---

## Post-Release

### 9. Verification

- [ ] Confirm GitHub Actions passed
- [ ] Test `git clone` + `npm install` + `npm run dev` from fresh directory
- [ ] Verify docs render correctly on GitHub

### 10. Communication

- [ ] Announce on project channels (if applicable)
- [ ] Update portfolio/website with new version

---

## Rollback (If Needed)

```bash
# Delete remote tag
git push origin :refs/tags/v15.3.0

# Delete local tag
git tag -d v15.3.0

# Revert commit
git revert HEAD

# Force push (use with caution)
git push origin main --force
```

---

**Responsible:** @CavinKrenik  
**Estimated Time:** ~2 hours
