# scripts/

## `setup-repo.sh` — apply the repository's GitHub settings

Everything about `graviaDaemon/Quirpy` that is not a file in the repository — merge settings,
topics, labels, and the rulesets protecting `main` and the `v*` tags — is applied by this script
instead of by clicking through the GitHub UI, so the settings are reviewable and re-runnable.

The script needs a short-lived fine-grained personal access token. Create it, use it, then throw it
away.

> The token is a credential. Never paste it into the repository, into a commit, or into a chat
> transcript. `~/.quirpy-gh-token` lives outside the repository on purpose — a file inside the
> working tree is one `git add -A` away from being published forever.

### 1. Create the token

1. GitHub → your avatar → **Settings** → **Developer settings** → **Personal access tokens** →
   **Fine-grained tokens** → **Generate new token**.
2. **Token name** `quirpy-setup`. **Expiration** 7 days — the shortest practical.
3. **Resource owner** `graviaDaemon`. **Repository access** → *Only select repositories* →
   `Quirpy`.
4. **Repository permissions** — exactly these, nothing more:

   | Permission | Access | Needed for |
   | --- | --- | --- |
   | Administration | Read and write | rulesets, merge settings, Discussions, vulnerability reporting |
   | Issues | Read and write | labels |
   | Metadata | Read-only | mandatory, selected automatically |

   Contents and Pull requests are **not** needed — the script never pushes code.
5. Click **Generate token** and copy it.

### 2. Store it without leaking it

```bash
read -rs token && printf '%s' "$token" > ~/.quirpy-gh-token && chmod 600 ~/.quirpy-gh-token && unset token
```

Press Enter, paste the token (it will not echo), press Enter again. Because the token never appears
on a command line, it never reaches your shell history.

`QUIRPY_GH_TOKEN` in the environment works too and takes precedence over the file.

### 3. Run it

```bash
./scripts/setup-repo.sh --dry-run   # prints every request it intends to make
./scripts/setup-repo.sh             # applies them
```

The script is safe to re-run: existing labels are left alone, and a ruleset that already exists is
updated in place rather than duplicated.

Afterwards, open **Settings → Rules** in the browser and confirm the bypass actor on both rulesets
is **Repository admin**. A wrong bypass actor is the one mistake here that locks you out of your own
`main`.

### 4. Revoke it

```bash
rm ~/.quirpy-gh-token
```

Then GitHub → Settings → Developer settings → Personal access tokens → the `quirpy-setup` token →
**Delete**. The settings are already applied; the token has no further use.
