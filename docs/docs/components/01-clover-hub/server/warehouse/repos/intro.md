# Repos

The `@warehouse/repos` path holds an RFQDN tree of all git repositories that Clover has registered. On boot, Clover will prune repos that are not registered in the store.

Each repo contains a [Manifest](/docs/components/clover-hub/server/warehouse/repos/manifest/intro), and has a `.git` directory inside.
