# service-nexus
Tfw no money so you fuse your apis and other services into this


## Services fused: 

- [TimetablesV2](https://github.com/rootofminus1atu/timetables-v2) lambda

- [UnboxCat](https://github.com/rootofminus1atu/unboxcat) backend - Had to rewrite an expressjs app for this to work, it was fun though.

- [TheJp2Api](https://github.com/rootofminus1atu/jp2cenzoapi) - only temporarily here. It will have its own place in the future

- Tf2 Subclass Creator backend


## Todos:
<!--unboxcat-->
- [ ] instead of fetching a random name just pick randomly from a list of names (make a random-name crate)
- [ ] maybe optimize /cats/random to do some fetches at the same time

<!--timetablesv2/general-->
- [ ] double check how to clone ClientWithKeys in an Arc-y way

<!--jp2api - to be done in that repo not here-->
- [ ] clean up the supabase struct
- [ ] fix public asset serving (idea: put it in public/jp2 instead of just public)
- [ ] restructure the project with some more mvc and other stuff


## Remarks

Would I use this for more serious projects? Probably not, as version control like this is quite a pain. It's fine to just keep some older, less-popular projects running, but for development it's not as reliable. 
