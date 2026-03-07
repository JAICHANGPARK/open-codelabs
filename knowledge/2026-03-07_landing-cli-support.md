# 2026-03-07 Landing CLI Support

## Summary
- Added visible `oc` CLI support to the landing page so visitors can see that Open Codelabs supports CLI-first setup and operations.

## Changes
- Added a new `CLI` tab to the landing quickstart section.
- The CLI quickstart now shows the `oc` flow for install, local stack launch, and connect/auth usage.
- Added a new feature card for CLI-first operations alongside AI, live ops, security, and open-source positioning.
- Expanded the quickstart copy to mention Docker, Podman, and the `oc` CLI together.

## Verification
- `cd /Users/jaichang/Documents/GitHub/open-codelabs/landing && bun run check`
- `cd /Users/jaichang/Documents/GitHub/open-codelabs/landing && bun run build`

## Notes
- `bun run check` and `bun run build` passed.
- Temporary dependency manifest updates caused by Bun execution were reverted so this work unit only changes landing copy and layout.
