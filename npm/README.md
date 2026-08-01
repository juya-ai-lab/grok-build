# @juya-ai-lab/grok-build

Terminal agent CLI from the [grok-build](https://github.com/juya-ai-lab/grok-build)
project, distributed as prebuilt binaries for Linux, macOS, and Windows
(x64 and arm64).

## Install

```sh
npm install -g @juya-ai-lab/grok-build
# or
pnpm add -g @juya-ai-lab/grok-build
```

The matching platform package is installed automatically through
`optionalDependencies`. No compilation and no install scripts are required,
so npm's install-script gating does not affect installation.

## Run

```sh
grok
# or, without installing
npx @juya-ai-lab/grok-build
```

Package versions mirror the GitHub release tags (for example `0.2.117`
corresponds to tag `v0.2.117`).
