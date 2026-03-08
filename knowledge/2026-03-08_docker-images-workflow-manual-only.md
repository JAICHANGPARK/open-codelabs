# 2026-03-08 docker images workflow manual only

## Summary

- Changed the Docker image publishing workflow to run only when started manually.
- Removed automatic triggers on `main` pushes and version tag pushes.

## Files changed

- `.github/workflows/docker-images.yml`

## Reasoning

- The requested behavior was to stop publishing Docker images automatically when commits land on `main`.
- Keeping only `workflow_dispatch` makes the workflow explicit and operator-controlled from the GitHub Actions UI.

## Verification

- Reviewed the workflow trigger block after the change.
- Ran `git diff --check`.
