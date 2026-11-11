# SpecSync documentation content and standalone redirects

The canonical marketing and documentation pages are on the
[CorvidLabs hub](https://corvidlabs.xyz/spec-sync/). Maintained documentation content is in
`src/content/docs/`. This Astro project builds the retired standalone site's redirects and
badges at https://corvidlabs.github.io/spec-sync; its root redirects to the hub.
Building or deploying these redirects does not deploy the hub's content.

## Dev

    bun install
    bun run dev        # localhost:4321/spec-sync/

## Build

    bun run build      # writes site/dist/

## Test

    bun test
