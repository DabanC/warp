# Local-only Warp Fork Design

## Summary

This design turns the Warp codebase into a downstream, local-only terminal application. The fork removes cloud account functionality from the source and dependency graph as aggressively as practical, while preserving the core terminal, local settings, file-based local workflows, and user-configured AI.

The fork is maintained as a reusable downstream patch stack. Each local-only change is split into semantic commits and exported to `patches/local-only/*.patch` so future upstream `master` updates can be synced, patched, verified, and packaged with a repeatable workflow.

## Goals

- Remove login, registration, account, team, billing, subscription, upgrade, Warp cloud, and sharing functionality.
- Remove or prune cloud-related crates and modules from the Cargo workspace dependency graph wherever practical.
- Preserve core terminal functionality.
- Preserve local settings.
- Preserve file-based local workflows only.
- Preserve AI through user-owned configuration:
  - API key.
  - Base URL.
  - Model name, if required by the current AI path.
- Make the existing API key/BYOK capability available without membership or plan gating.
- Add a Base URL field to the existing AI/API key settings UI.
- Ensure the app does not perform background external network requests by default.
- Allow network requests only when the user explicitly uses AI, and only to the configured AI Base URL.
- Produce a desktop installable artifact through a repeatable packaging workflow.
- Maintain downstream changes as patch commits and patch files for easy migration across upstream updates.

## Non-goals

- Do not preserve Warp cloud accounts.
- Do not preserve remote anonymous users.
- Do not preserve teams, organizations, or cloud workspaces.
- Do not preserve billing, Stripe portal, subscription, usage, quota, or upgrade flows.
- Do not preserve Warp Drive cloud objects.
- Do not preserve cloud notebooks.
- Do not preserve cloud object sharing, permissions, guests, link permissions, or object transfer.
- Do not preserve cloud sync or GraphQL subscriptions.
- Do not preserve managed secrets or cloud secret providers.
- Do not preserve cloud, ambient, or scheduled agents.
- Do not preserve platform/team-scoped API key management.
- Do not preserve telemetry, analytics uploads, crash reporting, update checks, or remote feature config fetches.
- Do not build a multi-provider AI plugin framework in the first version.
- Do not automate GitHub Release creation in the first version.

## User-visible behavior

On first launch, the application starts directly into the local terminal experience. It does not ask the user to sign in, create an account, join a team, configure billing, accept cloud sync, or enable telemetry.

Settings remain local. Account, team, billing, upgrade, cloud drive, cloud notebook, sharing, and managed secret UI entries are removed or unreachable.

AI configuration is available to every user. The AI settings area exposes API key and Base URL fields. If the current AI flow requires an explicit model name, the settings UI also exposes a model field or reuses the existing model selection mechanism without remote model fetching.

If AI is not configured, AI features show a local configuration error that points the user to the AI settings. The app must not open login, membership, upgrade, quota, or out-of-credits flows.

## Network boundary

Default app startup must not make background external network requests.

Forbidden by default:

- Warp auth endpoints.
- Warp GraphQL endpoints.
- Warp billing endpoints.
- Warp cloud object endpoints.
- Warp telemetry or analytics endpoints.
- Crash reporting submission.
- Update checks.
- Remote feature flag or remote config fetches.
- Cloud sync subscriptions.
- Server-side AI quota or model-list requests.

Allowed:

- User-triggered AI requests to the configured AI Base URL.
- User-triggered external browser navigation, such as clicking documentation links. Browser navigation is not considered app background networking.

## Source deletion strategy

The fork prioritizes deleting cloud/account code from the source and dependency graph, not merely hiding UI. Deletion should follow dependency boundaries rather than scattered UI-only edits.

### Primary deletion domains

1. Auth and account flows
   - Login, registration, SSO, device auth, session cookies, Firebase refresh/custom tokens, auth modals, login slides, persisted remote users, and remote anonymous user creation.

2. Billing, plan, quota, and upgrade flows
   - Billing modules, billing settings pages, usage pages, Stripe portal calls, quota UI, overage UI, upgrade modals, customer-type checks, delinquency checks, and paid-plan gating.

3. Cloud objects, Warp Drive, cloud notebooks, and sharing
   - Cloud object hydration, incremental sync, live subscriptions, guests, permissions, link permissions, object actions, trash/move/transfer flows, cloud notebooks, and Warp Drive UI.

4. Team, organization, and cloud workspace layer
   - Team membership, invites, discovery, roles, workspace metadata GraphQL queries, team settings, team workflows, cloud workflows, and team billing state.

5. Managed secrets and platform keys
   - Managed secrets crates, cloud secret provider integrations, managed secret GraphQL operations, platform/team-scoped API keys, and related UI.

6. Cloud/account-dependent agents and remote services
   - Cloud environments, ambient agents, scheduled agents, task identity tokens, task secrets, task attachments, and remote server pieces that require account auth.

7. Telemetry, crash reporting, update checks, and remote feature config
   - Event upload, analytics collection, analytics banners, crash submission, update check, remote config, and feature flag fetches.

### Candidate crates and modules

Candidate crates/modules for removal or pruning include, but are not limited to:

- `app/src/auth/**`
- `app/src/billing/**`
- billing and team settings UI modules
- `crates/graphql`
- `crates/warp_graphql_schema`
- `crates/warp_server_client`
- `crates/firebase`
- `crates/managed_secrets`
- cloud object, sharing, and server-client modules
- cloud/account-dependent remote agent modules

The exact set is determined by the dependency graph during implementation. If a crate cannot be removed immediately without destabilizing core terminal functionality, remove its runtime entry points first and record the crate as deferred cleanup.

### Deletion order

For each deletion domain:

1. Locate app-level imports and UI entry points.
2. Remove or replace app-level call sites.
3. Remove server/cloud client adapters used only by that domain.
4. Remove dependency declarations from app crates.
5. Remove workspace members when no longer referenced.
6. Remove build scripts, features, schema generation, or test references tied to removed crates.
7. Run compilation checks before moving to the next domain.

Each semantic patch commit should leave the project in a buildable state when practical. If a large dependency break requires a short multi-commit sequence, the sequence should be treated as one checkpoint and resolved before moving to unrelated domains.

## Local identity

The app no longer has a remote user. If internal code requires a stable identity, introduce a local-only identity.

Recommended model:

- `LocalIdentity`, `InstallationId`, or `LocalUser`.
- Generated once on first launch.
- Persisted locally.
- Never uploaded.
- Not linked to an account.
- Not called a remote anonymous user.

Uses:

- Local data namespacing.
- Compatibility with internal APIs that require a user-like identifier.
- Local session ownership.
- Optional local AI/conversation metadata ownership.

The app must not create a remote anonymous user.

## Local settings

Settings remain local and machine-scoped.

Remove or prune settings tied to:

- Account sync.
- Remote workspace sync.
- Team settings.
- Billing and usage settings.
- Telemetry upload settings.
- Server-side AI quota or billing cycles.

Preserve or add settings for:

- Terminal behavior.
- Appearance and editor behavior.
- Local workflows.
- AI API key.
- AI Base URL.
- AI model name if needed.

Because this fork is always local-only, no local-only mode toggle is required.

## File-based local workflows

Only file-based local workflows are preserved.

Preserve:

- Workflows loaded from repository files, local directories, or local user configuration.
- Workflow execution paths that do not depend on cloud object UIDs, team ownership, sharing permissions, or GraphQL mutations.

Remove or disable:

- Team workflows.
- Cloud workflows.
- Workflow sharing.
- Workflow owner transfer.
- Cloud workflow gallery if it depends on Warp services.
- Workspace/team-scoped workflow metadata.

If the existing Workflow UI strongly depends on cloud object infrastructure, the first version may omit that UI and preserve only the file-based loading/execution path.

## AI local-only design

AI reuses the existing API key/BYOK capability rather than introducing a new provider framework.

### AI settings

The existing AI/API key settings UI is updated so every user can configure AI without account or membership checks.

Settings fields:

- API key.
- Base URL.
- Model name if the current request path requires one.

Base URL examples:

- `https://api.openai.com/v1`
- `https://openrouter.ai/api/v1`
- `http://localhost:11434/v1`
- `http://localhost:1234/v1`

The Base URL is treated as the OpenAI-compatible API root. The client appends the endpoint path required by the implemented request path.

### AI request routing

AI requests must use the user-configured Base URL and API key. They must not use Warp-hosted AI endpoints or Warp account credentials.

Remove or bypass:

- Server-side request usage refresh.
- Request limit info queries.
- Free available model queries.
- AI overage queries.
- Bonus grants.
- Negative-feedback credit refund flows.
- Upgrade-required model gating.
- Out-of-requests gating for BYOK.
- Global AI analytics collection and AI analytics banners.

### Model behavior

The first version should not fetch model lists from Warp services.

Acceptable first-version approaches:

- Reuse an existing local model list if one exists.
- Let the user manually enter the model name.
- Add a minimal local model provider if existing UI requires model metadata.

### AI error handling

Errors are local configuration or provider errors, not Warp account errors.

- Missing API key: prompt the user to configure an API key.
- Missing Base URL: prompt the user to configure a Base URL.
- Missing model: prompt the user to configure a model.
- Connection failure: show endpoint connection error.
- 401/403: tell the user to check their configured API key or provider permissions.
- 429: tell the user their configured provider rate-limited the request.
- 5xx: show provider/server error.
- Invalid response format: show OpenAI-compatible response parsing error.

No AI error should trigger Warp login, reauthentication, logout, quota exceeded, membership, or upgrade UI.

## Local AI conversation/history

Cloud conversation sync is removed.

If the existing code already has local AI conversation persistence, preserve it. If it depends on cloud metadata, remove the cloud sync path and either preserve current-session history or introduce local persistence later.

The first version does not need to migrate cloud conversations.

## Patch stack design

The downstream fork is maintained as semantic patch commits.

Recommended patch order:

1. `local-only: add downstream patch workflow`
2. `local-only: remove auth and account flows`
3. `local-only: introduce local identity`
4. `local-only: remove billing and upgrade flows`
5. `local-only: remove team workspace cloud layer`
6. `local-only: remove cloud drive sharing and notebooks`
7. `local-only: keep only file workflows`
8. `local-only: remove managed secrets platform keys and cloud agents`
9. `local-only: remove telemetry crash update and remote config`
10. `local-only: make BYOK AI settings always available`
11. `local-only: route AI to custom OpenAI-compatible endpoint`
12. `local-only: prune cloud crates from workspace`
13. `local-only: add local-only verification and packaging checks`

The exact order may change if dependency removal requires it, but commits must remain semantic and reviewable.

## Patch files

Patch files are exported to:

- `patches/local-only/`

Expected files:

- `series`
- `README.md`
- numbered patch files such as `0001-add-downstream-patch-workflow.patch`

The `series` file defines application order.

## Workflow scripts

Add semi-automated scripts under:

- `script/local-only/`

Required scripts:

### `script/local-only/export-patches`

Exports the current local-only patch stack to `patches/local-only/` and updates `series`.

### `script/local-only/apply-patches`

Applies patches listed in `patches/local-only/series` onto a fresh branch based on upstream `master`. The script stops on conflicts and lets the maintainer resolve them.

### `script/local-only/verify`

Runs local-only verification:

- Compilation checks or the best available local-only subset check.
- Static scan for forbidden Warp service endpoints.
- Static scan for forbidden auth, billing, cloud, telemetry, update, and remote-config entry points.
- Checks that AI API key and Base URL settings exist.
- Checks that file-based local workflow paths remain present.

### `script/local-only/package`

Runs `verify` and then invokes the project’s existing desktop packaging flow to produce an installable desktop artifact.

### Optional `script/local-only/release-check`

Adds stricter pre-release checks such as artifact naming, manual smoke checklist output, and checksum generation.

## Syncing upstream

For each upstream update:

1. Fetch upstream.
2. Create a new branch from upstream `master`.
3. Run `script/local-only/apply-patches` or cherry-pick the semantic patch commits.
4. Resolve conflicts according to the current patch’s semantic domain.
5. Run `script/local-only/verify`.
6. Run `script/local-only/package`.
7. Manually smoke test the desktop app.
8. Promote the branch as the new local-only release source.

Recommended branch naming:

- `local-only-YYYYMMDD`

## Packaging scope

The first release workflow produces a desktop installable artifact. It does not create a GitHub Release automatically.

Future optional additions:

- Artifact naming conventions.
- Checksums.
- Changelog generation.
- GitHub Release draft creation.

## Verification strategy

### Build verification

- The app must compile.
- Core terminal crates must remain available.
- Local settings must compile.
- File-based workflows must compile.
- AI settings and custom endpoint path must compile.
- Removed crates must not remain as unused workspace members or dependencies unless explicitly deferred.

### Static no-cloud verification

Scan for remaining references to:

- Warp auth endpoints.
- Warp GraphQL endpoints.
- Billing and Stripe portal calls.
- Cloud object sync and subscriptions.
- Telemetry upload endpoints.
- Crash reporting submission.
- Update check endpoints.
- Remote feature config fetches.
- Request limit and free model queries.
- Login, signup, upgrade, and quota UI entry points.

### Runtime no-network verification

Manual release smoke test:

1. Start the app with no AI configured.
2. Confirm there are no background external network requests.
3. Open terminal and settings.
4. Confirm no sign-in, team, billing, upgrade, Warp Drive cloud, sharing, or managed secret entry points are reachable.
5. Configure AI Base URL, API key, and model.
6. Trigger one AI request.
7. Confirm the only network request goes to the configured Base URL.
8. Confirm provider errors do not trigger Warp login, quota, or upgrade UI.

### AI verification

- Missing API key shows local configuration error.
- Missing Base URL shows local configuration error.
- Missing model shows local configuration error when model is required.
- 401/403 from the configured endpoint shows provider credential guidance.
- 429 shows provider rate-limit guidance.
- 5xx shows provider error.
- Invalid response shape shows OpenAI-compatible parsing guidance.
- No AI path queries Warp request limits, model availability, overages, billing, or account state.

## Risk management

High-risk areas:

- Removing GraphQL, server client, or auth crates may break app bootstrap.
- Workspace/team metadata may currently participate in startup or settings initialization.
- Settings UI may directly reference billing/auth/team types.
- AI model selection may depend on remote metadata.
- Feature flags may guard many unrelated UI paths.
- Telemetry event types may be deeply tied to UI actions.
- Remote server or SSH functionality may mix local transport with account auth.

Risk handling:

- Work in small semantic commits.
- Compile after each deletion domain.
- Prefer deleting call sites before deleting dependencies.
- If a crate cannot be removed immediately, remove runtime entry points and record deferred cleanup.
- Keep the final branch free of half-deleted functionality.
- Treat local terminal startup, local settings, file workflows, and custom-endpoint AI as the release gates.

## Deferred cleanup policy

The goal is aggressive source and dependency removal. However, preserving a buildable application takes priority over deleting every cloud-related file in one pass.

A cloud-related crate or module may be temporarily retained only if:

- It is no longer reachable at runtime by default.
- It does not perform background external networking.
- Its remaining dependency is documented in the implementation notes or verification output.
- It is targeted by a later cleanup patch.

## Acceptance criteria

The design is complete when the local-only fork satisfies all of the following:

- App builds and packages as a desktop artifact.
- App launches without login or account prompts.
- Core terminal works.
- Local settings work.
- File-based workflows remain available.
- Existing API key configuration is available without membership checks.
- AI Base URL is configurable in the AI/API key settings UI.
- AI requests use only the configured Base URL and API key.
- No Warp account, billing, cloud, telemetry, update, crash, or remote-config background requests occur by default.
- Login, billing, team, upgrade, Warp Drive cloud, sharing, cloud notebook, managed secret, quota, and out-of-credits UI surfaces are removed or unreachable.
- Patch stack can be exported to `patches/local-only/`.
- Patch stack can be applied to a fresh upstream branch through the semi-automated workflow.
- Verification and packaging scripts exist and document their checks.
