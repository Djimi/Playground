# Playground project

## Project overview

That project is for creating user friendly Playgrounds map in Sofia, Bulgaria.

Basic user workflow:
1. you open a map where you can select, etc.
2. You can choose a neighbor, or you can filter - by distance from you, by age, by capabilities, etc. For instance you may want to search for playgrounds with swing, or with 
children's slide, or with both. 
3. You can see exact location + pictures
4. You can see feedback from people (they should be able to authenticate with Google account and more in the future)


Initially that will be webapp which should work find on phones/tables.

## Project info
This is a greenfield project.
Before producing design.md, do not assume the major technology stack.
Ask the user about unresolved architectural choices such as frontend,
backend, persistence and deployment technology.

Discuss every new technology, architectural concept/pattern, communication
protocol, or architectural change with the user first (for example CQRS,
transactional outbox, or optimistic/pessimistic locking). Routine implementation
libraries, such as a PostgreSQL driver, do not require discussion.

## Specifications

`openspec/specs/` is the source of truth for system behavior.

Before changing behavior:
- inspect the relevant specs in `openspec/specs/`
- check `openspec/changes/` for related active changes
- do not contradict existing specs

## Project docs
- Local setup: [LOCAL_SETUP.md](LOCAL_SETUP.md)
- Technology gotchas: [docs/gotchas](docs/gotchas/)
- Test cases: [TEST_CASES.md](TEST_CASES.md)

When a technology causes a bug or subtle behavior, add a short note to its
gotcha file with symptom, cause, fix, and verification. Update existing notes
before creating new files.


## Mindset
- Being on the same page with me is a KING - ask clarifying question, challenge, clarify until you reach this
- Be devil's advocate - always try to think about drawbacks and where things can break

## Reponse format
- Use text blocks/digrams where possible - visual explanation is better than 200 lines of rext
- Simple and short sentences
- Appropriately for fast reader


## Subagent Strategy

Default to delegation rather than doing exploratory work in the main context.

MUST spawn subagents when:
- researching unfamiliar code
- locating relevant files across multiple areas
- comparing 2+ approaches
- investigating independent questions
- reviewing an implementation
- a task can be split into 2+ independent workstreams

Do NOT use a subagent for:
- trivial edits
- one-file lookups
- tasks requiring less than ~5 minutes of reasoning/work

Before substantial work, explicitly ask:
"Which parts of this task should be delegated?"

Prefer cheaper/faster models for exploration and mechanical work.
Reserve the main Sol agent for orchestration, difficult reasoning, and final synthesis.

For independent workstreams, run subagents in parallel.
Each subagent gets one narrowly scoped task.gents
- One tack per subagent for focused execution

## Self-Improvement Loop
- After ANY correction from the user: append to `lessons.md` file with the pattern
- Write rules for yourself that prevent the same mistake
- Ruthlessly iterate on these lessons until mistake rate drops
- Review lessons at session start for relevant project
