# Multi threaded server

So this server is very limited and will slow down with more requests. It's also massively vulnerable against the [slow loris attack]

[slow loris attack]: https://en.wikipedia.org/wiki/Slowloris_(computer_security)

So let's add some threads. In this step we will spawn a new thread for each request we get.

- Spice up the content we return
  - Either load a user "template" or use a hardcoded default
- Spawn a new thread for each request
  - This uses shared state