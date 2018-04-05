# Multi threaded server

So let's add some threads. In this step we will spawn a new thread for each request we get.

- Spice up the content we return
  - Either load a user "template" or use a hardcoded default
- Spawn a new thread for each request
  - This uses shared state