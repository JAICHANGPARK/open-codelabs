# GitHub Pages Deployment Guide

This document explains how to deploy MkDocs documentation to GitHub Pages automatically.

## Automated deployment setup

### 1. GitHub repository settings

#### Step 1: Enable GitHub Pages

1. Go to the GitHub repository
2. Click **Settings**
3. Click **Pages** in the left menu
4. In **Source**:
   - Select **GitHub Actions**

![GitHub Pages Settings](https://docs.github.com/assets/cb-47267/mw-1440/images/help/pages/github-actions-source.webp)

#### Step 2: Verify workflow files

These files should already exist:

```
.github/workflows/
|-- docs.yml              # auto deploy workflow
`-- docs-pr-preview.yml   # PR build check
```

### 2. Push to the repository

```bash
git add .
git commit -m "docs: setup MkDocs with GitHub Pages deployment"
git push origin main
```

### 3. Verify deployment

1. Open the **Actions** tab
2. Find the "Deploy MkDocs to GitHub Pages" workflow
3. After the build completes (1-2 minutes):
   - Confirm success
   - Check the URL in **Settings > Pages**

Deployed URL:
```
https://<username>.github.io/<repository-name>/
```

Example:
```
https://yourusername.github.io/open-codelabs/
```

## Workflow details

### docs.yml (auto deploy)

**Triggers**:
- Push to `main`
- Changes to `docs/`, `mkdocs.yml`, `requirements.txt`
- Manual run (workflow_dispatch)

**Steps**:
1. Checkout
2. Install Python
3. Install dependencies (`requirements.txt`)
4. Build MkDocs (`mkdocs build`)
5. Deploy to GitHub Pages

### docs-pr-preview.yml (PR checks)

**Triggers**:
- Pull request opened
- Documentation-related files changed

**Steps**:
1. Build MkDocs
2. Comment on PR if build succeeds
3. Show errors if build fails

## Customization

### Custom domain

1. Create `CNAME` in `docs/`:

```bash
echo "docs.yourdomain.com" > docs/CNAME
```

2. Add `site_url` in `mkdocs.yml`:

```yaml
site_url: https://docs.yourdomain.com/
```

3. DNS setup:
   - CNAME record: `docs` -> `<username>.github.io`

### Change base URL

If the repo name is not `open-codelabs`, update `mkdocs.yml`:

```yaml
site_url: https://<username>.github.io/<your-repo-name>/
```

## Troubleshooting

### Deployment fails

**Cause 1: GitHub Pages disabled**

**Fix**:
1. Settings > Pages
2. Set Source to "GitHub Actions"

**Cause 2: Permissions**

**Fix**:
1. Settings > Actions > General
2. "Workflow permissions"
3. Select "Read and write permissions"
4. Enable "Allow GitHub Actions to create and approve pull requests"

**Cause 3: Build errors**

**Fix**:
```bash
# Test locally
mkdocs build --strict

# Fix the errors
```

### 404 on pages

**Cause**: incorrect base URL.

**Fix**:

Check `site_url` in `mkdocs.yml`:

```yaml
# Correct format
site_url: https://yourusername.github.io/open-codelabs/

# Or custom domain
site_url: https://docs.yourdomain.com/
```

### CSS/JS not loading

**Cause**: relative path issue.

**Fix**:

Add in `mkdocs.yml`:

```yaml
use_directory_urls: true
```

### Images do not render

**Cause**: incorrect image paths.

**Fix**:

Reference images like this:

```markdown
![Image](../assets/image.png)

# Or absolute path
![Image](/assets/image.png)
```

Store images in `docs/assets/`.

## Workflow status badge

Add a badge to README.md:

```markdown
[![Documentation](https://github.com/<username>/<repo>/actions/workflows/docs.yml/badge.svg)](https://github.com/<username>/<repo>/actions/workflows/docs.yml)
```

## Manual deployment

You can also deploy manually.

### Option 1: Run from GitHub Actions

1. Open **Actions**
2. Select "Deploy MkDocs to GitHub Pages"
3. Click "Run workflow"
4. Confirm "Run workflow"

### Option 2: Deploy locally

```bash
# Use mkdocs-ghpages plugin
pip install mkdocs-git-revision-date-localized-plugin

# Deploy to gh-pages branch
mkdocs gh-deploy --force

# Deploy with message
mkdocs gh-deploy -m "docs: update documentation"
```

!!! warning "Note"
    Local deployment can conflict with GitHub Actions. Use GitHub Actions when possible.

## Deployment history

1. Open **Actions**
2. Select "Deploy MkDocs to GitHub Pages"
3. Review each run
4. Download logs and artifacts

## Best practices

### 1. Branch strategy

```bash
# Work on a feature branch
git checkout -b docs/update-installation-guide

# Edit docs
vim docs/getting-started/installation.md

# Commit
git commit -m "docs: update installation guide for M1 Mac"

# Open a PR
git push origin docs/update-installation-guide
```

### 2. Commit messages

```bash
# Good examples
docs: add architecture documentation
docs: fix typo in API reference
docs: update deployment guide with GitHub Actions

# Bad examples
Update docs
Fix
Docs update
```

### 3. PR review

- Open a PR for documentation changes
- Ensure the build checks pass
- Review content
- Merge and auto-deploy

## Security

### Use secrets (if needed)

Store sensitive data in GitHub Secrets:

1. Settings > Secrets and variables > Actions
2. "New repository secret"
3. Use in workflow:

```yaml
- name: Deploy
  env:
    API_KEY: ${{ secrets.API_KEY }}
  run: mkdocs gh-deploy
```

## Additional resources

- [GitHub Pages docs](https://docs.github.com/en/pages)
- [GitHub Actions docs](https://docs.github.com/en/actions)
- [MkDocs deployment guide](https://www.mkdocs.org/user-guide/deploying-your-docs/)
- [Material for MkDocs setup](https://squidfunk.github.io/mkdocs-material/setup/)

## Checklist

Before deployment:

- [ ] GitHub Pages set to "GitHub Actions"
- [ ] `site_url` is correct in `mkdocs.yml`
- [ ] `requirements.txt` includes all dependencies
- [ ] `mkdocs build --strict` passes locally
- [ ] `.github/workflows/docs.yml` exists
- [ ] Changes pushed to `main`

After deployment:

- [ ] Workflow success in Actions
- [ ] GitHub Pages URL accessible
- [ ] All pages render correctly
- [ ] Navigation works
- [ ] Search works
- [ ] Mobile rendering works

## Done

The docs are now auto-deployed:

1. Push changes to `main`
2. GitHub Actions builds and deploys
3. Changes appear in 1-2 minutes

Docs URL: `https://<username>.github.io/<repository>/`
