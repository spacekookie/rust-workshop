# Threading via a threadpool

Our last server example has a problem: if there are too many requests this will spawn thousands of threads and possibly overload the server completely.

So instead what we want to do is used a threadpool which allocates a fixed amount of resources in the beginning, then assignes jobs to threads as they become available. Easy. For this to happen we need to use a few new concepts:

- Structs to hold state and as objects
- More advanced closures
- Channels as a method for communication