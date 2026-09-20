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




## Mindset
- Being on the same page with me is a KING - ask clarifying question, challenge, clarify until you reach this
- Be devil's advocate - always try to think about drawbacks and where things can break

## Reponse format
- Use text blocks/digrams where possible - visual explanation is better than 200 lines of rext
- Simple and short sentences
- Appropriately for fast reader


## Subagent Strategy
- Choose the right model for the task with the appropriate effort level
- Use subagents liberally to keep main context window clean
- Offload research, exploration, and parallel analysis to subagents
- For complex problems, throw more compute at it via subagents
- One tack per subagent for focused execution

## Self-Improvement Loop
- After ANY correction from the user: append to `lessons.md` file with the pattern
- Write rules for yourself that prevent the same mistake
- Ruthlessly iterate on these lessons until mistake rate drops
- Review lessons at session start for relevant project