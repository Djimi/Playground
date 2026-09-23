# Docker Compose

## Worktrees can silently use a different database

Symptom: one branch shows imported playground data, while another shows an
older or empty catalog.

Cause: Compose normally derives its project name from the checkout directory.
Different worktrees then get different Postgres volumes.

Fix: `compose.yaml` pins one project name, so all worktrees share the imported
database and schema. A new/empty shared volume still needs the manual import;
local startup intentionally does not fetch source data. Keep `.env` values the
same across worktrees, especially `POSTGRES_PASSWORD`: changing it does not
rotate the role password already stored in the volume, and the same Compose
services are reconfigured by whichever worktree starts them.

Verify the project name and source record count with:

```bash
docker compose config | sed -n '1,2p'
docker compose exec -T db psql -U playground -d playground \
  -c 'SELECT count(*) FROM source_playgrounds;'
```
