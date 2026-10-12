# Sessions

Which skill starts which kind of session, and when. The owner types the slash command: these skills are user-invoked, so type `/` and pick `mattpocock-skills:<name>`. The agent invokes `tdd`, `diagnosing-bugs`, `research`, `prototype`, `domain-modeling`, `codebase-design`, and `code-review` itself. One ticket per session; `/clear` between tickets.

| Situation | Start with | Output | Next |
|---|---|---|---|
| Build a ticket (most sessions) | `/implement #<n>`, or "Work the frontier per docs/agents/workflow.md Session start" | local `/code-review`, draft PR, ready when CI is green, then CodeRabbit's second opinion | owner merges |
| Decide a map ticket | `/wayfinder #<map>`; name a ticket or take the first frontier one | resolution comment, ticket closed, map updated, ADR PR if hard to reverse | next ticket, new session |
| Research ticket | "Resolve #<n>" (the agent uses `research`) | cited note under `docs/research/`, resolution on the map | none |
| New feature or next milestone | `/grill-with-docs <idea>`, then `/to-spec`, then `/to-tickets`, in one window | CONTEXT.md and ADR PR, spec issue, ticket issues | `/clear`, then build |
| Question that needs running code | `/handoff`, then `/prototype`, then `/handoff` back | prototype branch and the answer | resume grilling |
| Broken or slow | "diagnose: <symptom>" (the agent runs `diagnosing-bugs`) | red loop, regression test at a real seam, fix PR | owner merges |
| Outside report or idea | `/triage` | labelled issue with an agent brief | |
| Milestone boundary | `/improve-codebase-architecture`, then `/grill-with-docs` on one candidate; a literature scan of the next milestone's topics that checks whether new work challenges an existing ADR | HTML report, refactor ticket; "State of the art: …" tickets for the next milestone's technique decisions; an ADR challenge, if any, as a map comment | |
| Unsure | `/ask-matt <situation>` | the flow that fits | |

## Rules of thumb

- Build sessions start from a ticket and nothing else. Grilling and spec work (`/grill-with-docs`, `/to-spec`, `/to-tickets`) stay in their own sessions.
- `/implement` closes with a local `/code-review`: run it before the PR opens. CodeRabbit's review is a second opinion on top of it, treated as suggestions (workflow.md, Review).
- Mark at most two PRs ready within an hour: the plan includes 5 CodeRabbit reviews per rolling hour.
- Do not chart a milestone before the one ahead of it nears exit (workflow.md, The ladder).
- Two build sessions at once need separate worktrees and disjoint modules; open PRs stay at two.
- Status at any time: "Status: frontier, open PRs, review state."
