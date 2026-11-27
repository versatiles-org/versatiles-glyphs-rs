# versatiles-glyphs-rs

- Claude may commit finished work itself, after `scripts/check.sh` passes. Pushing still needs the user's go-ahead.
- `versatiles-fonts` renders its archives with the latest release of this CLI, so a fix only reaches map users through new releases downstream. After a release that changes the rendered glyphs, recommend releasing `versatiles-fonts`, then `versatiles-frontend`. A change to the CLI's arguments can break the fonts build (`../versatiles-fonts/scripts/glyphs.ts`).
